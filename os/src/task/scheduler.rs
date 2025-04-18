use core::{panic, sync::atomic::{AtomicBool, Ordering}};

use alloc::{collections::vec_deque::VecDeque, sync::Arc};
use lock_api::MutexGuard;

use crate::{
    interupt::{InterruptController, InterruptState}, processor::{self, current_processor_id, get_current_processor}, sync::spin::mutex::{self, IRQSpinLock, IRQSpinLockGuard}, task::switch::__switch, trap::trap_return
};

use super::{
    current_task, task::{TaskControlBlock, TaskControlBlockInner, TaskState}, TaskContext
};

pub trait Scheduler: Send + Sync {
    // drived by timer
    fn schedule(&self, yiled_task_guard: IRQSpinLockGuard<TaskControlBlockInner>);
    fn add_task(&self, task_control_block: Arc<TaskControlBlock>);
    fn fetch_task(&self) -> Option<Arc<TaskControlBlock>>;
    fn yield_current(&self);
    // fn exit_current(&self);
}

pub struct FiFoScheduler {
    ready_queue: IRQSpinLock<VecDeque<Arc<TaskControlBlock>>>,
    
    // blocked_tasks: IRQSpinLock<Vec<Weak<TaskControlBlock>>>,
    time_interval: u64,
    is_running: AtomicBool,
}

impl Scheduler for FiFoScheduler {

    // switch current task and schduler_task to return schedule_loop
    fn schedule(&self, yiled_task_guard: IRQSpinLockGuard<TaskControlBlockInner>) {
        assert_ne!(
            yiled_task_guard.get_state(),
            TaskState::Running,
            "Cannot schedule a Running task."
        );
        assert_ne!(
            InterruptController::get_state(),
            InterruptState::Enabled,
            "Cannot schedule duaring intterupt enable"
        );

        let schedule_loop_task_context = get_current_processor().get_schedule_loop_context();
        let yield_task_context = &yiled_task_guard.context as *const TaskContext;

        let interrupt_state = get_current_processor().get_saved_interrupt_state();

        let current_task = current_task();
        
        unsafe { 
            let name = yiled_task_guard.name.clone();
            log::debug!("pass {}'s guard into scheduler loop in schedule", name);
            current_task.transfer_lock(yiled_task_guard);
            __switch(yield_task_context as *mut TaskContext, schedule_loop_task_context);
            log::debug!("{} switch back to schedule", name);
        };

        get_current_processor().set_saved_interrupt_state(interrupt_state);

    }


    fn add_task(&self, task_control_block: Arc<TaskControlBlock>) {
        self.ready_queue.lock().push_back(task_control_block);
    }

    fn fetch_task(&self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.lock().pop_front()
    }


    fn yield_current(&self) {
        log::debug!("yiled_current: Yiled current task");
        let current_task = current_task();
        let mut pass_into_guard = current_task.lock();
        log::debug!("accquire {}'s guard in yield_current",pass_into_guard.name);

        log::debug!("switch current task state to Ready");
        pass_into_guard.set_state(TaskState::Ready);
        self.ready_queue.lock().push_back(current_task.clone());
        self.schedule(pass_into_guard);

        unsafe {
            let pass_out_guard = current_task.take_lock(); 
            log::debug!("release {}'s guard in yield_current",pass_out_guard.name);
        }
    }


}

impl FiFoScheduler {
    pub fn new(time_interval: u64) -> Self {
        Self {
            ready_queue: IRQSpinLock::new(VecDeque::new()),
            // blocked_tasks: IRQSpinLock::new(Vec::new()),
            time_interval,
            is_running: AtomicBool::new(false),
        }
    }

    pub fn init(&self) {
        self.is_running.store(true, Ordering::SeqCst);
        Self::setup_timer(self.time_interval);
    }

    

    fn setup_timer(_timer_interval: u64) {
        return;
    }

    // fn task_complete(&mut self, task: Arc<TaskControlBlock>);

    // fn task_blocked(&mut self, task: Arc<TaskControlBlock>);

    // fn task_wakeup(&mut self, task: Arc<TaskControlBlock>);
}


pub fn schedule_loop() {
    loop {
        log::debug!("schedule_loop");
        let processor = get_current_processor();
        // should disable_migrate in multiple core
        if let Some(next_task) = processor.fetch_task() {
            log::debug!("prepare switch to {:?}", next_task);
            // accquired by scheduler task from task A
            let mut next_task_guard = next_task.lock();

            let scheduler_context = &processor.schedule_loop_task_context as *const TaskContext;
            
            next_task_guard.state = TaskState::Running;
            processor.set_current_task(next_task.clone());
            
            let next_task_context = &next_task_guard.context as *const TaskContext;
            unsafe {
                next_task.transfer_lock(next_task_guard);
                __switch(scheduler_context as *mut TaskContext, next_task_context);
                log::debug!("switch back to scheduler loop");
                drop(next_task.take_lock());
            }
            

            log::debug!("switch back to scheduler loop");
            processor.clean_current_task();

            // released by scheduler task from task B
        } else {
            log::info!("No task avaliable to run")
        }
    }
}


#[allow(unused)]
pub fn new_user_task_start() {
    log::debug!("new user task start");
    unsafe { 
        drop(current_task().take_lock())
    }

    log::debug!("new user task start release lock");

    log::debug!("{} prepare return", current_task().lock().name);

    // return user from here
    trap_return()
}
