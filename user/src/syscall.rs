use core::arch::asm;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;

const SYSCALL_TEST: usize = 114514;

fn syscall(id: usize, args: [usize; 6]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}



pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len(), 0, 0, 0])
}



pub fn sys_exit(exit_code: i32) -> ! {
    let args =  { 
        let mut a = [0; 6];
        a[0] = exit_code as usize;
        a
    };
    syscall(SYSCALL_EXIT, args);
    unreachable!()
}

pub fn sys_yield() -> isize {
    let args = [0; 6];
    syscall(SYSCALL_YIELD, args)
}

pub fn sys_get_time() -> isize {
    let args = [0; 6];
    syscall(SYSCALL_GET_TIME, args)
}

pub fn sys_test(
    great_cross_page_ptr: usize,
    great_len: usize, 
    // arg2: usize, 
    // arg3: usize, 
    // arg4: usize, 
    // arg5: usize
)  -> isize {
    syscall(SYSCALL_TEST,
        [
        great_cross_page_ptr,
        great_len,
        0, 0, 0, 0]
    )

}