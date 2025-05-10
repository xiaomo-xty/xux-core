//! Processor core management for multi-core RISC-V processors.
//!
//! This module provides isolation and synchronization primitives for SMP (Symmetric Multi-Processing)
//! systems, with support for per-Processor task management and interrupt control.

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use alloc::boxed::Box;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use alloc::vec::Vec;


use crate::register::Tp;
use crate::task::{TaskContext, TaskControlBlock};
use crate::{interupt::InterruptState, sync::spin::mutex::Mutex};
use crate::task::scheduler::Scheduler;

/// A unique identifier for a Processor core (hart) in the system.
///
/// This is a newtype wrapper around `usize` that represents the hardware thread ID (hartid)
/// from the RISC-V `mhartid` CSR. It provides type safety when working with Processor identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessorId(usize);

impl From<ProcessorId> for usize {
    fn from(value: ProcessorId) -> Self {
        value.0
    }
}

/// The number of Processor cores supported by this system.
pub const CPU_NUM: usize = 1;

static mut PROCESSORS_LOCAL: [MaybeUninit<ProcessorLocal>; CPU_NUM] = 
    unsafe { MaybeUninit::uninit().assume_init() };


pub fn init_processor(hart_id: usize) {
    unsafe { init_processor_local(hart_id) ;}
}

unsafe fn init_processor_local(
    hart_id: usize,
) {
    PROCESSORS_LOCAL[hart_id].write(ProcessorLocal::new(hart_id));
    let current_processor_lcoal_data_ptr = PROCESSORS_LOCAL[hart_id].assume_init_ref() as *const ProcessorLocal as usize;
    Tp::write(current_processor_lcoal_data_ptr);
}

#[inline]
fn current_processor_local() -> &'static mut ProcessorLocal {
    let current_processor_lcoal_data_ptr = Tp::read();
    unsafe { 
        &mut *(current_processor_lcoal_data_ptr as *mut ProcessorLocal) 
    }

    // let id = current_processor_id().0;
    // &mut PROCESSORS_LOCAL[id] // 或 get_unchecked
}

lazy_static! {
    /// Per-CPU shared data (protected by IRQSpinLock)
    static ref PROCESSORS_SHARED: Vec<Mutex<ProcessorShared>> = {
        log::info!("Initializing {} processors (shared)", CPU_NUM);
        (0..CPU_NUM).map(|_| Mutex::new(ProcessorShared::new())).collect()
    };
}

/// Safe access to current CPU's local data


/// Safe access to current CPU's shared data
#[inline]
fn current_processor_shared() -> &'static Mutex<ProcessorShared> {
    let id = current_processor_id().0;
    &PROCESSORS_SHARED[id]  // 或 get_unchecked
}




pub struct ProcessorShared {
    ipi_pending: AtomicBool,
    wakeup_signal: AtomicBool,
}

impl ProcessorShared {
    pub const fn new() -> Self{
        Self {
            ipi_pending: AtomicBool::new(false),
            wakeup_signal: AtomicBool::new(false)
        }
    }
}


/// Per-Processor core management structure.
///
/// Each Processor core maintains its own task queue, execution context,
/// and interrupt locking state.
/// A core can't visit B core's Processor struct, so I remove the atomic
pub struct ProcessorLocal {
    // cann't be modify
    hart_id: usize,
    // - Task schedule
    /// maybe support multiple core scheduler
    scheduler: MaybeUninit<Box<dyn Scheduler>>,
    current_task: Option<Arc<TaskControlBlock>>,
    // A medium other task return schduler loop
    pub schedule_loop_task_context: TaskContext,
    
    // - Interrupt
    
    /// Nesting counter for interrupt disable operations.
    interrupt_nest_cnt: AtomicUsize,
    /// Saved interrupt state for restoration when unlocking.
    is_enable_interrupt: AtomicBool,
}

impl ProcessorLocal {
    /// Creates a new Processor instance for the given hardware thread.
    ///
    /// # Arguments
    pub fn new(hart_id: usize) -> Self {
        Self {
            hart_id,
            scheduler: MaybeUninit::uninit(),
            current_task: None,
            schedule_loop_task_context: TaskContext::zero_init(),
            interrupt_nest_cnt : AtomicUsize::new(0),
            is_enable_interrupt: AtomicBool::new(true),
        }
    }

    pub fn timer_tick(&self) {

        // log::debug!("timer tick");
        
        self.get_scheduler().yield_current();

        // log::debug!("timer tick handle finish")
    }

    #[inline]
    fn get_scheduler(&self) -> &Box<dyn Scheduler>{
        unsafe { self.scheduler.assume_init_ref() }
    }


    // should be call after memory init
    pub fn init_scheduler(&mut self, schduler: Box<dyn Scheduler>) {
        self.scheduler.write(schduler);
    }


    pub fn get_current_task(&self) -> Option<&Arc<TaskControlBlock>> {
        match &self.current_task {
            Some(task) => {
                Some(task)
            },
            None => {
                None
            }
        }
    }

    pub fn set_current_task(&mut self, task: Arc<TaskControlBlock> ) {
        self.current_task = Some(task);
    }

    pub fn clean_current_task(&mut self) {
        self.current_task = None;
    }

    pub fn get_schedule_loop_context(&mut self) -> &mut TaskContext{
        &mut self.schedule_loop_task_context
    }

    pub fn add_task(&self, task_control_block: Arc<TaskControlBlock>) {
        self.get_scheduler().add_task(task_control_block);
    }

    pub fn fetch_task(&self) -> Option<Arc<TaskControlBlock>> {
        self.get_scheduler().fetch_task()
    }

    pub fn yield_current(&self) {
        self.get_scheduler().yield_current();
    }

    pub fn exit_current(&self, exit_status: i32) {
        self.get_scheduler().exit_current(exit_status);
    }

    // ========== 中断管理接口 ========== //
    pub fn get_saved_interrupt_state(&self) -> InterruptState {
        self.is_enable_interrupt.load(Ordering::Acquire).into()
    }
    
    pub fn set_saved_interrupt_state(&self, state: InterruptState) {
        self.is_enable_interrupt.store(state.into(), Ordering::Release);
    }


    pub fn increment_nest(&self) -> usize {
        self.interrupt_nest_cnt.fetch_add(1, Ordering::Acquire)
    }

    pub fn decrement_nest(&self) -> usize {
        self.interrupt_nest_cnt.fetch_sub(1, Ordering::Release)
    }
}


/// Returns the ID of the current Processor core.
///
/// Reads the RISC-V `mhartid` CSR to determine which core is executing.
#[inline(always)]
pub fn current_processor_id() -> ProcessorId {
    // ProcessorId(hartid::read())
    ProcessorId(0)
}

/// Returns a mutable reference to the specified Processor core's structure.
///
/// # Arguments
/// * `id` - The Processor core identifier
///
/// # Safety
/// Caller must ensure:
/// - No other references to this Processor exist
/// - The ID is valid (0 ≤ id < CPU_NUM)
pub fn get_processor_by_id(id: ProcessorId) -> &'static Mutex<ProcessorShared> {
    let id: usize = id.into();
    log::debug!("return processor[{}]", id);
    &PROCESSORS_SHARED[id]
}


pub fn get_current_processor() -> &'static mut  ProcessorLocal {
    current_processor_local()
}
