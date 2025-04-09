use os_macros::syscall_register;
use super::get_time_us;

#[syscall_register(SYSCALL_GET_TIME)]
pub fn sys_get_time() -> isize {
    get_time_us() as isize
}