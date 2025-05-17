mod context;
mod switch;
mod task;
mod syscall;
mod allocator;
mod signal;
pub mod scheduler;

use alloc::{boxed::Box, string::{String, ToString}, sync::Arc};
pub use context::TaskContext;
use scheduler::FiFoScheduler;
pub use task::TaskControlBlock;
use crate::{fs::{open_file, OpenFlags}, mm::address::VirtAddr, processor::get_current_processor, trap::TrapContext};

// use crate::sync::UPSafeCell;


// pub struct TaskManager {
//     num_app: usize,
//     inner: UPSafeCell<TaskManagerInner>,
// }

// struct TaskManagerInner {
//     tasks: Vec<TaskControlBlock>,
//     current_task: usize,
// }   


pub fn init_scheduler() {
    log::info!("initialize scheduler");
    let processor = get_current_processor();
    processor.init_scheduler(Box::new(FiFoScheduler::new(1)));

    log::info!("load init_task");

    if let Some(app_inode) = open_file("init_proc", OpenFlags::RDONLY) {
        log::debug!("open file dead_loop2 success");
        let all_data = app_inode.read_all();
        // let task = current_task().unwrap();
        processor.add_task(TaskControlBlock::new_from_elf(
            &all_data.as_slice(), 
            "init_task".to_string(), 
            None)
        );
    }
    else {
        panic!("not found init proc");
    }
}

pub fn current_task() -> Option<&'static Arc<TaskControlBlock>> {
    let current_task = get_current_processor().get_current_task();
    current_task
}

pub fn current_user_token() -> usize {
    // log::debug!("get current user token");
    current_task().unwrap().lock().with_user_res(|user_res| {
        // log::debug!("get user memory set lock");
        user_res.memory_set.lock().token()
    })
    
}

pub fn current_user_trap_context_va() -> VirtAddr {
    current_task().unwrap().lock().with_user_res(|user_res| {
        user_res.trap_context_vpn().into()
        })
}

pub fn current_user_trap_context() -> &'static mut TrapContext {
    current_task().unwrap().lock().with_user_res(|user_res| {
        user_res.trap_context_ppn().get_mut()
    })
}


pub fn yield_current() {
    get_current_processor().yield_current();
}

pub fn exit_current(exit_status: i32) {
    get_current_processor().exit_current(exit_status);
}

