use os_macros::syscall_register;

#[syscall_register(SYSCALL_EXIT)]
pub fn sys_exit(xstate: i32) -> ! {
    log::info!(" Application exited with code {}", xstate);
    // exit_current_and_run_next();
    unimplemented!();
    // unreachable!()
}

#[syscall_register(SYSCALL_YIELD)]
pub fn sys_yield() -> isize {
    unimplemented!();
    // suspend_current_and_run_next();
    // 0
}