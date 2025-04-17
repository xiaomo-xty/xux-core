use core::{panic, sync::atomic::{AtomicBool, Ordering}};

use alloc::{collections::vec_deque::VecDeque, sync::Arc};

use crate::{
    interupt::{InterruptController, InterruptState}, processor::get_current_processor, sync::spin::mutex::SpinMutex
};

use super::{
    switch::switch,
    task::{TaskControlBlock, TaskState},
    TaskContext,
};

pub trait Scheduler: Send + Sync {
    fn run(&mut self);
    // drived by timer
    fn timer_tick(&self);
    fn schedule(&self, yiled_task: Arc<TaskControlBlock>);
    fn add_task(&self, task_control_block: Arc<TaskControlBlock>);
    fn fetch_task(&self) -> Option<Arc<TaskControlBlock>>;
    fn get_current_task(&self) -> Option<Arc<TaskControlBlock>>;
}

pub struct FiFoScheduler {
    ready_queue: SpinMutex<VecDeque<Arc<TaskControlBlock>>>,

    // A medium other task return schduler loop
    schedule_loop_task_context: TaskContext,
    current_task: Option<Arc<TaskControlBlock>>,
    // blocked_tasks: SpinMutex<Vec<Weak<TaskControlBlock>>>,
    time_interval: u64,
    is_running: AtomicBool,
}

impl Scheduler for FiFoScheduler {
    fn run(&mut self) {
        let is_running = self.is_running.fetch_or(true, Ordering::SeqCst);
        if is_running == true {
            panic!("You cann't start a running scheduler")
        }

        self.schedule_loop();


    }

    fn timer_tick(&self) {
        if !self.is_running.load(Ordering::SeqCst) {
            return;
        }
        self.yield_current();
    }

    // switch current task and schduler_task to return schedule_loop
    fn schedule(&self, yiled_task: Arc<TaskControlBlock>) {
        assert_ne!(
            yiled_task.get_state(),
            TaskState::Running,
            "Cannot schedule a Running task."
        );
        assert_ne!(
            InterruptController::get_state(),
            InterruptState::Enabled,
            "Cannot schedule duaring intterupt enable"
        );

        let schedule_loop_task_context = &self.schedule_loop_task_context;
        let yield_task_context = { &mut yiled_task.lock().context };

        let interrupt_state = get_current_processor().get_saved_interrupt_state();
        
        switch(yield_task_context, schedule_loop_task_context);
        InterruptController::set_state(interrupt_state);
    }


    fn add_task(&self, task_control_block: Arc<TaskControlBlock>) {
        self.ready_queue.lock().push_back(task_control_block);
    }

    fn fetch_task(&self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.lock().pop_front()
    }

    fn get_current_task(&self) -> Option<Arc<TaskControlBlock>> {
        self.current_task.clone()
    }
}

impl FiFoScheduler {
    pub fn new(time_interval: u64) -> Self {
        Self {
            ready_queue: SpinMutex::new(VecDeque::new()),
            schedule_loop_task_context: TaskContext::zero_init(),

            current_task: None,
            // blocked_tasks: SpinMutex::new(Vec::new()),
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

    fn schedule_loop(&mut self) {
        loop {
            log::info!("schedule_loop");
            // should disable_migrate in multiple core
            if let Some(next_task) = self.fetch_task() {
                log::debug!("prepare switch to {:?}", next_task);
                // accquired by scheduler task from task A
                let mut next_task_guard = next_task.lock();

                let scheduler_context = &mut self.schedule_loop_task_context;

                next_task_guard.state = TaskState::Running;
                self.current_task = Some(next_task.clone());

                
                switch(scheduler_context, &next_task_guard.context);

                log::debug!("switch back to scheduler loop");
                self.current_task = None

                // released by scheduler task from task B
            } else {
                log::info!("No task avaliable to run")
            }
        }
    }


    fn yield_current(&self) {
        if let Some(current_task) = &self.current_task {
            current_task.set_state(TaskState::Ready);
            self.ready_queue.lock().push_back(current_task.clone());

            self.schedule(current_task.clone());
        } else {
            panic!("It's shouldn't no current task while yield")
        }
    }

    // fn task_complete(&mut self, task: Arc<TaskControlBlock>);

    // fn task_blocked(&mut self, task: Arc<TaskControlBlock>);

    // fn task_wakeup(&mut self, task: Arc<TaskControlBlock>);
}
