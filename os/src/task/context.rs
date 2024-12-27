
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    /// return address ( e.g. __restore ) of __switch ASM function
    ra: usize,
    /// kernel stack pointer of app
    sp: usize,
    /// callee saved registers:  s 0..11
    s: [usize; 12],
}


impl TaskContext {
    /// `goto_restore` method is used to create a `TaskContext` instance, initializing the stack pointer and the return address (RA).
    /// This method takes a stack pointer `kstack_ptr` as input, sets it to the `sp` field, and sets the `ra` field to the address of the `__restore` function.
    /// The `__restore` function is an external C function that will be called when the task is resumed, to restore the task context.
    ///
    /// # Parameters
    /// - `kstack_ptr`: The pointer to the task's kernel stack (typically the stack bottom address) used to restore the task's stack.
    ///
    /// # Returns
    /// A `TaskContext` instance initialized with `ra` set to the address of the `__restore` function, 
    /// `sp` set to the passed `kstack_ptr`, and the `s` register array initialized to zero.
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        extern "C" {fn __restore();}
        Self {
            ra: __restore as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }

    /// `zero_init` method creates a zero-initialized `TaskContext` instance.
    /// All fields, including the return address `ra`, stack pointer `sp`, and saved registers `s`, are initialized to zero.
    ///
    /// # Returns
    /// A zero-initialized `TaskContext` instance.
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
}