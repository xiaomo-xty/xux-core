use core::{panic, sync::atomic::{AtomicBool, Ordering}};

use alloc::{collections::vec_deque::VecDeque, sync::Arc};

use crate::{
    interupt::{InterruptController, InterruptState}, processor::{self, current_processor_id, get_current_processor}, sync::spin::mutex::{IRQSpinLock, IRQSpinLockGuard}, task::switch::__switch, trap::trap_return
};

use super::{
    current_task, task::{TaskControlBlock, TaskControlBlockInner, TaskState}, yield_current, TaskContext
};

pub trait Scheduler: Send + Sync {
    // drived by timer
    fn schedule(&self, yiled_task_guard: IRQSpinLockGuard<TaskControlBlockInner>);
    fn add_task(&self, task_control_block: Arc<TaskControlBlock>);
    fn fetch_task(&self) -> Option<Arc<TaskControlBlock>>;
    fn yield_current(&self);
    fn exit_current(&self, exit_code: i32);
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

        let schedule_loop_task_context = get_current_processor()
                                                            .get_schedule_loop_context();
        let yield_task_context = &yiled_task_guard.context as *const TaskContext;

        let interrupt_state = get_current_processor().get_saved_interrupt_state();

        let yield_out_task = current_task().unwrap();

        
        unsafe { 
            let name = yield_out_task.get_name();
            log::debug!("pass {} 's guard into scheduler loop in schedule", name);
            yield_out_task.store_lock(yiled_task_guard);

            __switch(yield_task_context as *mut TaskContext, schedule_loop_task_context);
            
            log::debug!("{} switch back to schedule", name);
            let switch_back_task = current_task().unwrap();
            let switch_back_task_gurad = switch_back_task.take_lock();
            log::debug!("release current task lock");
            get_current_processor().set_saved_interrupt_state(interrupt_state);
            drop(switch_back_task_gurad);
        };


    }


    fn add_task(&self, task_control_block: Arc<TaskControlBlock>) {
        log::debug!("task len before add: {}", self.ready_queue.lock().len());
        self.ready_queue.lock().push_back(task_control_block);
        log::debug!("task len after add: {}", self.ready_queue.lock().len());
    }

    fn fetch_task(&self) -> Option<Arc<TaskControlBlock>> {
        log::debug!("task len before fetch: {}", self.ready_queue.lock().len());
        let a = self.ready_queue.lock().pop_front();
        log::debug!("task len after fetch: {}", self.ready_queue.lock().len());
        a
    }

    fn yield_current(&self) {
        log::debug!("yield out current");
        let current_task = current_task();
        if let Some(task) = current_task {
            self.yield_task(task);
        }
        log::debug!("yield in current");
    }



    fn exit_current(&self, exit_code: i32) {
        let current_task = current_task().unwrap();
        current_task.prepare_exit();

        let mut current_task_guard = current_task.lock();
        current_task_guard.set_state(TaskState::Zombie(exit_code));
        
        //child task group, place to init
        current_task_guard.notify_parent(exit_code);
        self.schedule(current_task_guard);
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

    // yield the specified task
    // the task must be current task
    // and the guard mode make sure the lock be accquire.
    // Before yield, the task's state should be `TaskState::Ready`
    // and has been added to ready q
    fn yield_task(&self, task : &Arc<TaskControlBlock>) {

        assert_eq!(task.lock().get_state(), TaskState::Running);

        log::debug!("yield out task {}", task.get_name());
        // let current_task = current_task();

        let mut task_guard = task.lock();
        task_guard.set_state(TaskState::Ready);
        // self.add_task(task.clone());
        self.schedule(task_guard);
        log::debug!("yield in task {}", current_task().unwrap().get_name());
    }

    

    fn setup_timer(_timer_interval: u64) {
        return;
    }

    // fn task_complete(&mut self, task: Arc<TaskControlBlock>);

    // fn task_blocked(&mut self, task: Arc<TaskControlBlock>);

    // fn task_wakeup(&mut self, task: Arc<TaskControlBlock>);
}


pub fn schedule_loop() {
    let processor = get_current_processor();
    loop {
        // Avoid deadlock by ensuring that devices can interrupt.
        // Example: just one process waiting disk, but we wait a `RUNNING` process
        // and need interrrupt to change the process's state to `RUNNING` from `SLEEPING`
        InterruptController::global_enable();

        log::debug!("schedule_loop");
        // should disable_migrate in multiple core
        if let Some(next_task) = processor.fetch_task() {
            log::debug!("prepare switch to {:?}", next_task);
            // accquired by scheduler task from task A
            let mut next_task_guard = next_task.lock();

            let scheduler_context = &processor.schedule_loop_task_context as *const TaskContext;
            
            assert_eq!(next_task_guard.get_state(), TaskState::Ready);
            next_task_guard.state = TaskState::Running;
            processor.set_current_task(next_task.clone());
            
            let next_task_context = &next_task_guard.context as *const TaskContext;
            


            unsafe {
                next_task.store_lock(next_task_guard);
                __switch(scheduler_context as *mut TaskContext, next_task_context);
                log::debug!("switch back to scheduler loop");
                
                let current_task = current_task().unwrap();
                let switch_back_task_gurad = current_task.take_lock();
                
                
                processor.clean_current_task();


                match switch_back_task_gurad.state {
                    TaskState::Ready => { 
                        processor.add_task(next_task);
                    },
                    TaskState::Zombie(exit_code) => {
                        // log::debug!("Zombie task {}, exit code: {}", current_task.get_name(), exit_code);
                        // log::debug!("task arc count: {}", Arc::strong_count(&current_task))
                    },
                    _ => ()
                    
                }
                
                drop(switch_back_task_gurad);
                log::debug!("release switch back task gurad");
            };


            
            
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
        drop(current_task().unwrap().take_lock())
    }

    
    
    log::debug!("new user task start release lock");


    log::debug!("{} prepare return", current_task().unwrap().get_name());

    // return user from here
    trap_return()
}
