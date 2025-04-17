#![allow(unused)]
use core::{
    cell::UnsafeCell,
    hint,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicUsize, Ordering}, u32, usize,
};

use crate::{interupt::{InterruptController, InterruptState, IntrReqGuard}, processor::current_processor_id};

/// A spin-based mutual exclusion lock (spinlock)
///
/// This provides mutually exclusive access to the underlying data through
/// a busy-wait loop (spinning) while waiting for the lock to become available.
///
/// # Features
/// - Optimized spinning with `hint::spin_loop()`
/// - Debug-mode recursion checking
/// - Memory ordering guarantees (Acquire/Release semantics)
///
/// # Example
/// ```
/// let lock = SpinMutex::new(42);
/// let mut guard = lock.lock();
/// *guard = 10;
/// ```
///
/// 
/// # Safety Note
/// - This is a ​**spinlock**, not a sleep-wait lock. Do not hold it for long periods.
/// - Always disable interrupts before locking in kernel mode.
/// - In user mode, consider `std::sync::Mutex` instead.
pub struct SpinMutex<T> {
    /// Atomic flag indicating whether the lock is held
    locked: AtomicBool,
    /// The protected data wrapped in UnsafeCell for interior mutability
    data: UnsafeCell<T>,
    
    #[cfg(debug_assertions)]
    /// Track lock holder for recursion detection (debug only)
    holder_id: AtomicUsize,
}

unsafe impl<T> Sync for SpinMutex<T> where T: Send {}
unsafe impl<T> Send for SpinMutex<T> where T: Send {}

/// A guard that provides mutable access to the data protected by a SpinMutex
///
/// When the guard goes out of scope, the lock will be automatically released.
/// Implements `Deref` and `DerefMut` for transparent access to the inner data.
pub struct SpinMutexGuard<'a, T> {
    /// Reference to the parent spinlock
    mutex: &'a SpinMutex<T>,
    irq_guard: IntrReqGuard,
}


impl<T> SpinMutex<T> {
    /// Sentinel value indicating no holder (debug builds only)
    const NO_HOLDER: AtomicUsize = AtomicUsize::new(usize::MAX);

    /// Creates a new spinlock protecting the provided data
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),

            #[cfg(debug_assertions)]
            holder_id: Self::NO_HOLDER,
        }
    }

    /// Locks the spinlock and returns a guard
    ///
    /// # Behavior
    /// - Spins (with backoff) until lock is acquired
    /// - In debug builds, checks for recursive locking
    /// - Uses Acquire ordering for synchronization
    pub fn lock(&self) -> SpinMutexGuard<'_, T> {
        log::debug!("prepare get lock");
        let irq_guard = InterruptController::intr_disable_nested();

        #[cfg(debug_assertions)]
        self.check_dead_lock();


        log::debug!("start spin for get lock");
        while self.locked.swap(true, Ordering::Acquire) {
            // log::debug!("spin..");
            hint::spin_loop();
        }
        log::debug!("end spin");
        
        #[cfg(debug_assertions)] 
        {
            let cpu_id = current_processor_id();
            self.holder_id.store(cpu_id.into(), Ordering::Relaxed);
        }

        SpinMutexGuard { mutex: self, irq_guard }
    }

    /// Checks for recursive locking (debug builds only)
    ///
    /// # Returns
    /// `true` if current CPU already holds this lock
    #[inline(always)]
    pub fn check_dead_lock(&self) {
        #[cfg(debug_assertions)] {
            let holder = self.holder_id.load(Ordering::Relaxed);
            if holder != usize::MAX && holder == current_processor_id().into() {
                panic!("dead lock occur, holder: {}", holder);
            }
        }
        #[cfg(not(debug_assertions))] {
            false
        }
    }
}

impl<T> Deref for SpinMutexGuard<'_, T> {
    type Target = T;

    /// Immutable access to protected data
    ///
    /// # Safety
    /// Protected by spinlock - no concurrent access possible
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for SpinMutexGuard<'_, T> {
    /// Mutable access to protected data
    ///
    /// # Safety
    /// Protected by exclusive spinlock ownership
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for SpinMutexGuard<'_, T> {
    /// Releases the lock when guard is dropped
    ///
    /// Uses Release ordering to ensure all critical section operations
    /// complete before lock release.
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
        #[cfg(debug_assertions)]
        {
            self.mutex.holder_id.store(usize::MAX, Ordering::Release);
        }
        log::debug!("release lock");
    }
}