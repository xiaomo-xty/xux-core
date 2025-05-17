use core::panic;

use os_macros::syscall_register;

use crate::{mm::{page_table::translated_byte_buffer, user_ptr::UserPtr, UserBuffer}, print, task::{current_task, current_user_token}};

use super::{open_file, OpenFlags};

const FD_STDOUT: usize = 1;


#[syscall_register(SYSCALL_WRITE)]
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let task_guard = current_task().unwrap().lock();

    let token = task_guard.user_res.as_ref().unwrap().memory_set.lock().token();

    let fd_table = task_guard.user_res.as_ref().unwrap().fd_table.lock();
    if fd >= fd_table.len() {
        return -1;
    }
    if let Some(file) = &fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(fd_table);
        drop(task_guard);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len).unwrap())) as isize
    } else {
        -1
    }
}

#[syscall_register(SYSCALL_READ)]
pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let task_guard = current_task().unwrap().lock();

    let token = task_guard.user_res.as_ref().unwrap().memory_set.lock().token();

    let fd_table = task_guard.user_res.as_ref().unwrap().fd_table.lock();
    if fd >= fd_table.len() {
        return -1;
    }
    if let Some(file) = &fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len).unwrap())) as isize
    } else {
        -1
    }
}



#[syscall_register(SYSCALL_OPEN)]
pub fn sys_open(file: *const u8, flags: u32) -> isize{
    let current_task = current_task().unwrap();
    let token = current_task.lock().get_user_token();

    let user_file = UserPtr::new(token, file);
    let path = user_file.read_to_string();

    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut task = current_task.lock();
        let user_res = task.user_res.as_mut().unwrap();

        let fd = user_res.alloc_fd();
        user_res.fd_table.lock()[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }

}

#[syscall_register(SYSCALL_CLOSE)]
pub fn sys_close(fd: usize) -> isize{
    let task = current_task().as_ref().unwrap().lock();
    let mut fd_table = task.user_res.as_ref().unwrap().fd_table.lock();
    if fd >= fd_table.len() {
        return -1;
    }
    if fd_table[fd].is_none() {
        return -1;
    }
    fd_table[fd].take();
    0
}
