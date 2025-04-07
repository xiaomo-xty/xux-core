mod fs;
mod process;
mod test;
pub mod syscall_num;
pub mod error;

use error::Errno;
use fs::*;
use process::{sys_exit, sys_get_time, sys_yield};
use syscall_num::*;
use test::sys_test;

pub fn syscall_dispatch(syscall_id: usize, args: [usize; 6]) -> isize {
    log::debug!("syscall_id: {}", syscall_id);
    log::debug!("paramete: {:?}", args);

    let syscall = match Syscall::from_repr(syscall_id) {
        Some(s) => s,
        None => return -(Errno::ENOSYS as isize), // Return ENOSYS for unknown calls
    };

    match syscall {
        Syscall::Write =>  { 
            sys_write(args[0], args[1] as *const u8, args[2]) 
        },
        Syscall::Exit => sys_exit(args[0] as i32),
        Syscall::Yield => sys_yield(),
        Syscall::GetTime => sys_get_time(),
        Syscall::Test => { 
            sys_test(
                args[0],
                args[1],
                args[2],
                args[3],
                args[4],
                args[5]
            );
            0
        },
        // Syscall::Fork => sys_fork(),
        // Syscall::Exec => sys_exec(),
        // Syscall::WaitPid => sys_waitpid(),
        // Syscall::Read => sys_read(),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
