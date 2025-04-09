#![allow(missing_docs)] 
#![allow(unused)]

// use strum_macros::FromRepr;

pub const SYSCALL_READ: usize = 63;
pub const SYSCALL_WRITE: usize = 64;
pub const SYSCALL_EXIT: usize = 93;
pub const SYSCALL_YIELD: usize = 124;
pub const SYSCALL_GET_TIME: usize = 169;
// pub const SYSCALL_GETPID: usize = 172;

pub const SYSCALL_FORK: usize = 220;
pub const SYSCALL_EXEC: usize = 221;
pub const SYSCALL_WAITPID: usize = 260;
pub const SYSCALL_TEST: usize = 511;

// #[derive(Debug, FromRepr, PartialEq, Eq)]
// #[repr(usize)] // 指定底层类型为 usize
// pub enum Syscall {
//     Read = SYSCALL_READ,
//     Write = SYSCALL_WRITE,
//     Exit = SYSCALL_EXIT,
//     Yield = SYSCALL_YIELD,
//     GetTime = SYSCALL_GET_TIME,
//     Fork = SYSCALL_FORK,
//     Exec = SYSCALL_EXEC,
//     WaitPid = SYSCALL_WAITPID,
//     Test = SYSCALL_TEST,
// }

