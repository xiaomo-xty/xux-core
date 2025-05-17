//! System call handling infrastructure
//!
//! This module provides the core functionality for dispatching system calls
//! to their respective handlers through a system call table.
mod test;
mod registry;

pub mod syscall_num;
pub mod error;

pub use registry::SyscallRegistry;


use error::Errno;
use registry::SYSCALL_TABLE;


/// Dispatches a system call to its registered handler
///
/// # Arguments
/// * `syscall_id` - The numeric identifier of the system call
/// * `args` - Array of 6 arguments passed from userspace (mapped from registers a0-a5)
///
/// # Returns
/// The return value from the system call handler, or a negative error code if:
/// * The system call number is invalid (`-Errno::ENOSYS`)
///
/// # Safety
/// This function is unsafe because:
/// * It accesses the global system call table without synchronization
/// * It executes arbitrary function pointers from the table
/// * System call handlers may perform unsafe operations
pub fn syscall_handler(syscall_id: usize, args: [usize; 6]) -> isize {
    // log::debug!("syscall handler, syscall_id: {}", syscall_id);


    // log::debug!("getting syscall_table");
    let syscall_table = SYSCALL_TABLE.read();
    // log::debug!("getted syscall_table");

    unsafe {
        // Look up the handler in the system call table
        let syscall_wrap = match syscall_table.get(syscall_id).and_then(|f| *f) {
            Some(func) => func,
            None => return -(Errno::ENOSYS as isize),
        };
    
        // Execute the system call handler

        drop(syscall_table);
    
        syscall_wrap(args)
    }
}


/// registry the syscalls
pub fn init() {
    unsafe {
        registry::init();
    }
}