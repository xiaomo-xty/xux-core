//! # Spin Locks Module
//!
//! Provides low-level synchronization primitives for kernel/embedded contexts.
//!
//! ## Implementations
//! ### Spin-based Locks (Non-blocking)
//! - [ ] `TicketLock` - Fair spinlock using ticket algorithm  
//!   ▶ Prevents thread starvation at the cost of slightly higher latency
//! - [x] [`SpinMutex`] - Basic spinlock implementation  
//!     - [x] Core locking functionality (`lock()`, `try_lock()`)
//!     - [ ] Backoff strategy optimization  
//!       ▶ Exponential backoff for high-contention scenarios
//! - [x] CpuSpinLock
//!
//! ### Blocking Locks
//! - [ ] `Mutex` - Thread-blocking mutex with scheduler integration  
//!   ▶ Will be implemented after kernel scheduler is ready  
//!   ▶ Planned features:  
//!     - [ ] Priority inheritance  
//!     - [ ] Deadlock detection hooks
//!
//! ## Usage Guidelines
//! ```rust
//! // Spinlock example (for <100ns critical sections)
//! use kernel_sync::spin::SpinMutex;
//! let lock = SpinMutex::new(0);
//! *lock.lock() = 42;
//!
//! // Future blocking mutex example
//! #[cfg(future)]
//! use kernel_sync::mutex::Mutex; // Will yield CPU when contested
//! ```
//!
//! ## Safety
//! - All locks implement `Send + Sync` for cross-thread use
//! - Spinlocks MUST NOT be held across:
//!   - Scheduling boundaries (use blocking `Mutex` instead)
//!   - Long-running operations (>1µs)
//! - IRQ safety requirements marked with `#[interrupt_safe]`

pub mod mutex;
mod test;