//! Synchronization primitives based on spinning, including interrupt-safe variants.
//!
//! This module provides two main spinlock implementations:
//! - [`SpinLock`]: A basic spinlock for thread synchronization
//! - [`IRQSpinLock`]: An interrupt-disabling spinlock for kernel contexts

use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use lock_api::{GuardSend, RawMutex};
use crate::{interupt::InterruptController, processor::current_processor_id};

/// A mutual exclusion lock based on spinning (busy-waiting)
///
/// This is a basic spinlock that provides mutually exclusive access to data
/// between threads. It does not disable interrupts and should not be used
/// in interrupt contexts.
///
/// # Example
/// ```
/// let lock = SpinLock::new(0);RawMutexRawSpinLock
/// let mut guard = lock.lock();
/// *guard += 1;
/// ```
pub type SpinLock<T> = lock_api::Mutex<RawSpinLock, T>;

/// A guard that provides mutable access to the data protected by [`SpinLock`]
///
/// When the guard goes out of scope, the lock will be automatically released.
pub type SpinLockGuard<'a, T> = lock_api::MutexGuard<'a, RawSpinLock, T>;

/// An interrupt-safe spinlock that disables interrupts while held
///
/// This variant automatically disables interrupts when acquired and
/// restores them when released, making it safe for use in interrupt
/// contexts and kernel code.
///
/// # Safety
/// - Must not be held across sleep operations
/// - Interrupts remain disabled while the lock is held
pub type IRQSpinLock<T> = lock_api::Mutex<RawIrqSpinlock, T>;

/// A guard that provides mutable access to the data protected by [`IRQSpinLock`]
///
/// When dropped, this guard will release the lock and restore interrupt state.
pub type IRQSpinLockGuard<'a, T> = lock_api::MutexGuard<'a, RawIrqSpinlock, T>;

/// The raw implementation of a basic spinlock
///
/// This provides the low-level synchronization primitive that [`SpinLock`]
/// builds upon. It uses an atomic boolean to track lock state and includes
/// optional debug checks for recursion detection.
pub struct RawSpinLock {
    /// Atomic flag indicating whether the lock is held
    locked: AtomicBool,
    
    #[cfg(debug_assertions)]
    /// Track lock holder for recursion detection (debug only)
    holder_id: AtomicUsize,
}

impl RawSpinLock {
    /// Sentinel value indicating no current holder
    const NO_HOLDER: AtomicUsize = AtomicUsize::new(usize::MAX);

    #[cfg(debug_assertions)]
    /// Check for potential deadlock situations in debug mode
    ///
    /// This will panic if the current CPU already holds this lock,
    /// indicating a recursive locking attempt that could deadlock.
    fn check_dead_lock(&self) {
        let holder = self.holder_id.load(Ordering::Relaxed);
        if holder != usize::MAX && holder == current_processor_id().into() {
            panic!("dead lock occur, holder: {}", holder);
        }
    }
}

unsafe impl RawMutex for RawSpinLock {
    const INIT: RawSpinLock = RawSpinLock { 
        locked: AtomicBool::new(false),
        #[cfg(debug_assertions)]
        holder_id: Self::NO_HOLDER,
    };
    
    type GuardMarker = GuardSend;

    /// Acquire the spinlock, spinning until available
    ///
    /// This will busy-wait while the lock is held by another thread.
    /// In debug builds, it also checks for recursive locking attempts.
    fn lock(&self) {
        log::debug!("accquire lock");
        #[cfg(debug_assertions)]
        self.check_dead_lock();

        while !self.try_lock() {
            core::hint::spin_loop()
        }

        #[cfg(debug_assertions)] 
        {
            let cpu_id = current_processor_id();
            self.holder_id.store(cpu_id.into(), Ordering::Relaxed);
        }
    }

    /// Attempt to acquire the lock without spinning
    ///
    /// Returns `true` if the lock was acquired, `false` otherwise.
    fn try_lock(&self) -> bool {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    /// Release the lock
    ///
    /// # Safety
    /// - Must only be called when the lock is held by the current thread
    unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
        #[cfg(debug_assertions)]
        {
            self.holder_id.store(usize::MAX, Ordering::Release);
        }
        log::debug!("release lock");
    }
}

/// The raw implementation of an interrupt-disabling spinlock
///
/// This wraps a [`RawSpinLock`] and adds interrupt state management,
/// making it safe for use in interrupt contexts.
pub struct RawIrqSpinlock {
    /// The underlying spinlock implementation
    inner: RawSpinLock,
}

unsafe impl RawMutex for RawIrqSpinlock {
    const INIT: RawIrqSpinlock = RawIrqSpinlock { 
        inner: RawSpinLock::INIT
    };
    
    type GuardMarker = GuardSend;

    /// Acquire the lock while disabling interrupts
    ///
    /// This will:
    /// 1. Disable interrupts
    /// 2. Spin until the lock is acquired
    fn lock(&self) {
        InterruptController::intr_disable_nested();
        self.inner.lock();
    }

    /// Attempt to acquire the lock without spinning
    ///
    /// Returns `true` if the lock was acquired, `false` otherwise.
    /// Does not modify interrupt state for failed attempts.
    fn try_lock(&self) -> bool {
        self.inner.try_lock()
    }

    /// Release the lock and restore interrupts
    ///
    /// # Safety
    /// - Must only be called when the lock is held by the current thread
    unsafe fn unlock(&self) {
        self.inner.unlock();
        InterruptController::intr_enable_nested();
    }
}