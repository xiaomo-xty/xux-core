use crate::processor::get_current_processor;

use super::{current_task, task::TaskControlBlockInner, TaskControlBlock};

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Signal {
    /// 1 - 终端挂断或控制进程终止 (可捕获)
    SIGHUP = 1,
    /// 2 - 键盘中断 (Ctrl+C) (可捕获)
    SIGINT = 2,
    /// 3 - 键盘退出 (Ctrl+\) (生成核心转储)
    SIGQUIT = 3,
    /// 6 - 进程异常终止 (如 assert 失败)
    SIGABRT = 6,
    /// 9 - 立即强制终止进程 (不可屏蔽!)
    SIGKILL = 9,
    /// 11 - 非法内存访问 (段错误)
    SIGSEGV = 11,
    /// 13 - 向无读端的管道写入
    SIGPIPE = 13,
    /// 14 - 定时器超时 (alarm/setitimer)
    SIGALRM = 14,
    /// 15 - 优雅终止请求 (kill 默认信号)
    SIGTERM = 15,
    /// 17 - 子进程状态变更 (退出/停止)
    SIGCHLD = 17,
    /// 19 - 暂停进程执行 (不可屏蔽!)
    SIGSTOP = 19,
    /// 20 - 终端暂停请求 (Ctrl+Z, 可捕获)
    SIGTSTP = 20,
}

// ===== 扩展方法 =====
impl Signal {
    /// 判断信号是否不可屏蔽 (SIGKILL/SIGSTOP)
    pub fn is_unmaskable(&self) -> bool {
        matches!(self, Signal::SIGKILL | Signal::SIGSTOP)
    }

    /// 判断信号是否会导致进程终止
    pub fn is_fatal(&self) -> bool {
        match self {
            Signal::SIGCHLD | Signal::SIGTSTP => false,
            _ => true,
        }
    }

    /// 获取信号描述 (兼容 strsignal(3))
    pub fn description(&self) -> &'static str {
        match self {
            Signal::SIGHUP => "Hangup",
            Signal::SIGINT => "Interrupt",
            Signal::SIGQUIT => "Quit (core dumped)",
            Signal::SIGABRT => "Aborted",
            Signal::SIGKILL => "Killed",
            Signal::SIGSEGV => "Segmentation fault",
            Signal::SIGPIPE => "Broken pipe",
            Signal::SIGALRM => "Alarm clock",
            Signal::SIGTERM => "Terminated",
            Signal::SIGCHLD => "Child status changed",
            Signal::SIGSTOP => "Stopped (signal)",
            Signal::SIGTSTP => "Stopped (user)",
        }
    }
}

impl TaskControlBlock {
    pub fn handler_signal(&mut self, signal: Signal) {
        match signal {
            Signal::SIGTERM => get_current_processor().exit_current(-1),
            _ => unreachable!()
        }
    }
}