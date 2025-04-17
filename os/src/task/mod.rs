mod context;
mod switch;
mod task;
mod syscall;
mod allocator;
pub mod scheduler;

use alloc::sync::Arc;
pub use context::TaskContext;
use crate::{loader::{get_app_data, get_num_app}, mm::address::VirtAddr, processor::get_current_processor, trap::TrapContext};
use task::TaskControlBlock;

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
    let scheduler = get_current_processor().scheduler.lock();
    let num_app = get_num_app();

    for app_id in 0..num_app {
        log::info!("load {}th app", app_id);
        let app_data = get_app_data(app_id);
        log::info!("add {}th task", app_id);
        scheduler.add_task(TaskControlBlock::new_from_elf(app_data, app_id));
        log::info!("push {}th app", app_id);
    }
}

pub fn get_current_task() -> Arc<TaskControlBlock>{
    get_current_processor().scheduler.lock().get_current_task().unwrap()
}

pub fn get_current_user_token() -> usize {
    get_current_task().with_user_res(|user_res| {
        user_res.unwrap().memory_set.lock().token()
    })
}

pub fn get_current_user_trap_context_va() -> VirtAddr {
    get_current_task().with_user_res(|user_res| {
        user_res.unwrap().trap_context_vpn().into()
        })
}

pub fn get_current_user_trap_context() -> &'static mut TrapContext {
    get_current_task().with_user_res(|user_res| {
        user_res.unwrap().trap_context_ppn().get_mut()
    })
}


