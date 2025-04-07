use strum_macros::FromRepr;

const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
// const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_TEST: usize = 114514;

#[derive(Debug, FromRepr, PartialEq, Eq)]
#[repr(usize)] // 指定底层类型为 usize
pub enum Syscall {
    Read = SYSCALL_READ,
    Write = SYSCALL_WRITE,
    Exit = SYSCALL_EXIT,
    Yield = SYSCALL_YIELD,
    GetTime = SYSCALL_GET_TIME,
    Fork = SYSCALL_FORK,
    Exec = SYSCALL_EXEC,
    WaitPid = SYSCALL_WAITPID,
    Test = SYSCALL_TEST,
}

