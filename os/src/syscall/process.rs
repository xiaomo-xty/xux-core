use crate::{task::{exit_current_and_run_next, suspend_current_and_run_next}, timer::get_time_us};

/// task exits and submit an exit code
/// 注意这里我们并没有检查传入参数的安全性，
/// 即使会在出错严重的时候 panic，还是会存在安全隐患。
/// 这里我们出于实现方便暂且不做修补。
pub fn sys_exit(xstate: i32) -> ! {
    log::info!(" Application exited with code {}", xstate);
    exit_current_and_run_next();
    unreachable!()
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_us() as isize
}
