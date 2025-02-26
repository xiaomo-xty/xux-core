use crate::{config::*, trap::TrapContext};
use core::arch::asm;

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Clone, Copy)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}


static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE]
}; MAX_APP_NUM];

static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack {
    data: [0; USER_STACK_SIZE]
}; MAX_APP_NUM];


/// Represents the kernel stack used for saving task context.
impl KernelStack {
    
    /// Returns the current stack pointer of the kernel stack.
    /// This is calculated by adding the kernel stack base address to the kernel stack size.
    /// 
    /// # Returns
    /// * `usize`: The current stack pointer (i.e., the top of the kernel stack).
    fn get_sp(&self) -> usize {
        // Calculate the current stack pointer based on the stack base and size
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    /// Pushes a `TrapContext` onto the kernel stack.
    /// The method writes the provided `TrapContext` into the space at the current stack pointer 
    /// and returns the address of the pushed context.
    ///
    /// # Arguments
    /// * `trap_cx`: The `TrapContext` to be pushed onto the stack.
    ///
    /// # Returns
    /// * `usize`: The address of the `TrapContext` in the kernel stack.
    pub fn push_context(&self, trap_cx: TrapContext) -> usize {
        // Get the address for the TrapContext to be pushed
        let trap_cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        
        // Use unsafe block to directly modify the memory at the pointer
        unsafe  {
            // Write the TrapContext into the kernel stack at the computed address
            *trap_cx_ptr = trap_cx;
        }
        
        // Return the address of the pushed context as a usize
        trap_cx_ptr as usize
    }
}


impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

fn get_base_i(app_id : usize) -> usize{
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

pub fn get_num_app() -> usize {
    extern "C" { fn _num_app();}
    unsafe { 
        let num_app_ptr = _num_app as usize as *const usize;
        num_app_ptr.read_volatile()
    }
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    extern "C" {fn _num_app();}
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    assert!(app_id < num_app);
    let app_start = unsafe {
        core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
    };

    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id]   
        )
    }
}

pub fn load_apps() {
    extern "C" { fn _num_app();}
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    log::info!("Load app total {}.", num_app);
    let app_start = unsafe {
        core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
    };
    
    for i in 0..num_app {
        let base_i = get_base_i(i);
        log::info!("{}th app starting from {:#X}", i,  base_i);
        // clear region
        (base_i..base_i + APP_SIZE_LIMIT)
            .for_each(|addr| unsafe {
                (addr as *mut u8).write_volatile(0)
        });
        // load app from data section to memory
        let src = unsafe {
            core::slice::from_raw_parts(
                app_start[i] as *const u8, 
                app_start[i + 1] - app_start[i]
            )
        };

        let dst = unsafe {
            core::slice::from_raw_parts_mut(base_i as *mut u8, src.len())
        };
        
        dst.copy_from_slice(src);

    }
    unsafe {
        asm!("fence.i");
    }
}



/// Initializes the application context for the given application ID.
/// This function sets up the trap context (initializing registers, stack pointer, etc.)
/// for the application and pushes it onto the kernel stack.
///
/// # Parameters
/// - `app_id`: The unique identifier of the application. It is used to retrieve
///   the entry point and stack pointer associated with the application.
///
/// # Returns
/// The function returns the address of the newly pushed `TrapContext` on the kernel stack.
/// This address is used for the application's context switching during trap handling.
///
/// # Details
/// - The application entry point and stack pointer are retrieved using the `app_id`.
/// - The `TrapContext` for the application is initialized using `app_init_context`.
/// - The `TrapContext` is pushed onto the kernel stack for future context switching.
///
/// # Logs
/// This function logs the application ID, entry point, and stack pointer for debugging purposes.
pub fn init_app_cx(app_id : usize) -> usize {
    log::debug!("App id: {}  , Entry : {:#X}, Stack Point: {:#X}",
         app_id, get_base_i(app_id), USER_STACK[app_id].get_sp()
    );

    KERNEL_STACK[app_id].push_context(TrapContext::app_init_context(
        get_base_i(app_id),
        USER_STACK[app_id].get_sp(),
    ))
}