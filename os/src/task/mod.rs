mod context;
mod switch;
mod task;
mod syscall;
mod allocator;
pub mod scheduler;

use alloc::{boxed::Box, sync::Arc};
pub use context::TaskContext;
use scheduler::FiFoScheduler;
pub use task::TaskControlBlock;
use crate::{loader::{get_app_data, get_num_app}, mm::address::VirtAddr, processor::get_current_processor, trap::TrapContext};

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
    let num_app = get_num_app();

    for app_id in 0..num_app {
        log::info!("load {}th app", app_id);
        let app_data = get_app_data(app_id);
        log::info!("add {}th task", app_id);
        processor.add_task(TaskControlBlock::new_from_elf(app_data, app_id));
        log::info!("push {}th app", app_id);
    }
}

pub fn current_task() -> Arc<TaskControlBlock> {
    log::debug!("perpare get current task");
    let current_task = get_current_processor().get_current_task();
    log::debug!("get task");
    current_task
}

pub fn current_user_token() -> usize {
    current_task().with_user_res(|user_res| {
        user_res.unwrap().memory_set.lock().token()
    })
}

pub fn current_user_trap_context_va() -> VirtAddr {
    current_task().with_user_res(|user_res| {
        user_res.unwrap().trap_context_vpn().into()
        })
}

pub fn current_user_trap_context() -> &'static mut TrapContext {
    current_task().with_user_res(|user_res| {
        user_res.unwrap().trap_context_ppn().get_mut()
    })
}


pub fn yield_current() {
    get_current_processor().yield_current();
}

