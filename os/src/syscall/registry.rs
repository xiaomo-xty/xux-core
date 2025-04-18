//! 
//! Module for system call handling infrastructure.
//! Provides the system call table and initialization functionality.

use lazy_static::lazy_static;

use crate::sync::spin::mutex::IRQSpinLock;

// use crate::sync::UPSafeCell;

/// Type alias for system call handler functions.
/// These are unsafe C-ABI functions that take 6 arguments and return an isize.
type SyscallHandler = unsafe extern "C" fn(args: [usize; 6]) -> isize;

type RWLock<T> = IRQSpinLock<T>;

lazy_static! {

    /// Global system call dispatch table (statically initialized).
    /// 
    /// # Safety
    /// - This is mutable static data and requires unsafe access
    /// - Must be properly initialized before use
    /// - Contains 512 entries (0-511) matching standard system call numbers
    #[link_section = ".syscall_table"]
    pub static ref SYSCALL_TABLE: IRQSpinLock<[Option<SyscallHandler>; 512]> = 
            RWLock::new([None; 512]);
}

/// Structure representing a registered system call entry.
/// Used by the linker to collect all system call registrations.
///
/// # Fields
/// - `num`: The system call number
/// - `handler`: Function pointer to the handler implementation
#[repr(C)]
pub struct SyscallRegistry {
    /// The system call number (e.g., SYSCALL_READ)
    pub num: usize,
    /// The handler function for this system call
    pub handler: SyscallHandler,
}

/// Initializes the global system call table by populating it with registered handlers.
///
/// # Safety
/// - Must only be called once during system initialization
/// - Relies on linker-provided symbols for registration data
/// - Modifies mutable static data
///
/// # Panics
/// - If any system call number is out of bounds (>511)
pub unsafe fn init() {
    extern "C" {
        // Linker-provided symbols marking start/end of registration section
        static __syscall_registry_start: SyscallRegistry;
        static __syscall_registry_end: SyscallRegistry;
    }
    
    // Calculate bounds of registration data
    let start = &__syscall_registry_start as *const SyscallRegistry;
    let end = &__syscall_registry_end as *const SyscallRegistry;
    let count = (end as usize - start as usize) / core::mem::size_of::<SyscallRegistry>();

    let mut syscall_table = SYSCALL_TABLE.lock();
    
    // Populate system call table
    for i in 0..count {
        let entry = &*start.add(i);
        syscall_table[entry.num] = Some(entry.handler);
    }
}


/// unuseful
#[allow(unused)]
pub unsafe fn hotpatch(num: usize, new_handler: SyscallHandler) {
    SYSCALL_TABLE.lock()[num] = Some(new_handler)
}