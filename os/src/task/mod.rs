mod context;
mod switch;
mod task;
mod syscall;

use alloc::vec::Vec;
pub use context::TaskContext;
use crate::{config::*, loader::{get_app_data, get_num_app}, trap::TrapContext};
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use lazy_static::lazy_static;

use crate::sync::UPSafeCell;


pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,
}   


lazy_static!{
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        log::info!("total {} app", num_app);
        
        let mut tasks: Vec<TaskControlBlock> = Vec::new();



        for app_id in 0..num_app {
            let app_data = get_app_data(app_id);
            tasks.push(TaskControlBlock::new(app_data, app_id));
            log::info!("push {}th app", app_id);
        }
        TaskManager {
            num_app,
            inner: unsafe { 
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
        
    };
}



impl TaskManager {
    fn run_first_task(&self) -> ! {
        log::debug!("Run the first task");
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        // println!("The tasks[0]: {}", task0.task_cx);

        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as * const TaskContext;
        drop(inner);

        
        let mut _unused = TaskContext::zero_init();
        unsafe {
            // Passing `_unused` as a `*mut TaskContext` 
            // to the `__switch` function 
            // means that the current task's context 
            // will be saved into `_unused`.
            log::debug!("switch");
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
            log::debug!("switch out from the first task");
        }
        unreachable!()
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }

    fn get_current_trap_ctx(&self) -> &'static mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_trap_cx()
    }

    fn run_next_task(&self) {
        log::debug!("Run next task.\n");
        if let Some(next) = self.find_next_task() {
            log::debug!("next app id : {}", next);
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;

            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as * const TaskContext;
            drop(inner);
            unsafe {
                __switch(
                    current_task_cx_ptr,
                    next_task_cx_ptr
                );
            }
        } else {
            panic!("All applications completed!");
        }
    }

    fn find_next_task(&self) -> Option<usize> {
        log::debug!("find the next task");
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        log::debug!("current app_id : {}", current);
        ((current + 1)..(current + self.num_app + 1))
            .map(|id| id % self.num_app)
            .find(|id| {
                inner.tasks[*id].task_status == TaskStatus::Ready
            })
    }
}


pub fn run_first_task() {
    log::info!("run the first task");
    TASK_MANAGER.run_first_task();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// Get the current 'Running' task's token.
pub fn current_user_token() -> usize{
    TASK_MANAGER.get_current_token()
}

/// Get the current 'Running' task's trap contexts
pub fn current_trap_ctx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_ctx()
}