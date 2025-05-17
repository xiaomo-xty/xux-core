use os_macros::syscall_register;

use crate::{fs::{open_file, OpenFlags}, mm::page_table::translated_str, task::exit_current};

use super::{current_task, yield_current};

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

// #[syscall_register(SYSCALL_EXEC)]
// pub fn sys_exec(path: *const u8) -> isize {
//     let task = current_task().unwrap().lock();
//     let token = task.user_res
//         .as_ref().unwrap()
//         .memory_set.lock()
//         .token();
//     let path = translated_str(token, path);
//     if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
//         let all_data = app_inode.read_all();
//         let task = current_task().unwrap();
//         task.exec(all_data.as_slice());
//         0
//     } else {
//         -1
//     }
// }