//! Processor core management for multi-core RISC-V processors.
//!
//! This module provides isolation and synchronization primitives for SMP (Symmetric Multi-Processing)
//! systems, with support for per-Processor task management and interrupt control.

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use alloc::boxed::Box;
use alloc::sync::Arc;
use lazy_static::lazy_static;
use alloc::vec::Vec;


use crate::{interupt::InterruptState, sync::spin::mutex::SpinMutex};
use crate::task::scheduler::{FiFoScheduler, Scheduler};

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



lazy_static! {
    /// Per-CPU local data (lock-free access)
    static ref PROCESSORS_LOCAL: Vec<ProcessorLocal> = {
        log::info!("Initializing {} processors (local)", CPU_NUM);
        (0..CPU_NUM).map(|_| ProcessorLocal::new()).collect()
    };

    /// Per-CPU shared data (protected by SpinMutex)
    static ref PROCESSORS_SHARED: Vec<SpinMutex<ProcessorShared>> = {
        log::info!("Initializing {} processors (shared)", CPU_NUM);
        (0..CPU_NUM).map(|_| SpinMutex::new(ProcessorShared::new())).collect()
    };
}

/// Safe access to current CPU's local data
#[inline]
fn current_processor_local() -> &'static ProcessorLocal {
    let id = current_processor_id().0;
    &PROCESSORS_LOCAL[id]  // 或 get_unchecked
}

/// Safe access to current CPU's shared data
#[inline]
fn current_processor_shared() -> &'static SpinMutex<ProcessorShared> {
    let id = current_processor_id().0;
    &PROCESSORS_SHARED[id]  // 或 get_unchecked
}


// temp state
type RWLock<T> = SpinMutex<T>;


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

    // - Task schedule
    /// maybe support multiple core scheduler
    pub scheduler: SpinMutex<Box<dyn Scheduler>>,
    
    // - Interrupt
    
    /// Nesting counter for interrupt disable operations.
    pub interrupt_nest_cnt: AtomicUsize,
    /// Saved interrupt state for restoration when unlocking.
    pub is_enable_interrupt: AtomicBool,
}

impl ProcessorLocal {
    /// Creates a new Processor instance for the given hardware thread.
    ///
    /// # Arguments
    pub fn new() -> Self {
        Self {
            scheduler: SpinMutex::new(Box::new(FiFoScheduler::new(1))),
            interrupt_nest_cnt : AtomicUsize::new(0),
            is_enable_interrupt: AtomicBool::new(true),
        }
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
pub fn get_processor_by_id(id: ProcessorId) -> &'static SpinMutex<ProcessorShared> {
    let id: usize = id.into();
    log::debug!("return processor[{}]", id);
    &PROCESSORS_SHARED[id]
}


pub fn get_current_processor() -> &'static ProcessorLocal {
    current_processor_local()
}
