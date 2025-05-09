use os_macros::syscall_register;

use crate::task::exit_current;

use super::yield_current;

#[syscall_register(SYSCALL_EXIT)]
pub fn sys_exit(exit_status: i32) -> ! {
    log::warn!(" Application exited with code {}", exit_status);
    exit_current(exit_status);
    unreachable!()
}

#[syscall_register(SYSCALL_YIELD)]
pub fn sys_yield() -> isize {
    yield_current();
    0
}
