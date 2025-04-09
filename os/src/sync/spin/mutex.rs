#![allow(unused)]
use core::{
    cell::UnsafeCell,
    hint,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};


/// A spin-based mutual exclusion lock (spinlock)
///
/// This provides mutually exclusive access to the underlying data through
/// a busy-wait loop (spinning) while waiting for the lock to become available.
///
/// # Example
/// ```
/// let lock = SpinMutex::new(42);
/// let mut guard = lock.lock();
/// *guard = 10;
/// ```
pub struct SpinMutex<T> {
    /// Atomic flag indicating whether the lock is held
    locked: AtomicBool,
    /// The protected data wrapped in UnsafeCell for interior mutability
    data: UnsafeCell<T>,
}

/// A guard that provides mutable access to the data protected by a SpinMutex
///
/// When the guard goes out of scope, the lock will be automatically released.
pub struct SpinMutexGuard<'a, T> {
    /// Reference to the parent spinlock
    mutex: &'a SpinMutex<T>,
}

impl<T> SpinMutex<T> {
    /// Creates a new spinlock protecting the provided data.
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Locks the spinlock and returns a guard.
    ///
    /// This function will spin (busy-wait) with optimized backoff behavior
    /// until the lock becomes available.
    pub fn lock(&self) -> SpinMutexGuard<'_, T> {
        while self.locked.swap(true, Ordering::Acquire) {
            // Optimization hint to the processor that we're in a spin-wait loop
            hint::spin_loop();
        }
        SpinMutexGuard { mutex: self }
    }

    /// Attempts to acquire the lock without blocking.
    ///
    /// Returns `Some(guard)` if the lock was acquired, or `None` if another thread
    /// currently holds the lock.
    pub fn try_lock(&self) -> Option<SpinMutexGuard<'_, T>> {
        if !self.locked.swap(true, Ordering::Acquire) {
            Some(SpinMutexGuard { mutex: self })
        } else {
            None
        }
    }
}

impl<T> Deref for SpinMutexGuard<'_, T> {
    type Target = T;

    /// Provides immutable access to the protected data.
    ///
    /// # Safety
    /// This is safe because we hold the lock, ensuring no other thread can
    /// concurrently access the data.
    fn deref(&self) -> &Self::Target {
        unsafe { 
            &*self.mutex.data.get() 
        }
    }
}

impl<T> DerefMut for SpinMutexGuard<'_, T> {
    /// Provides mutable access to the protected data.
    ///
    /// # Safety
    /// This is safe because we hold the lock exclusively, ensuring no other
    /// references to the data exist.
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { 
            &mut *self.mutex.data.get() 
        }
    }
}

impl<T> Drop for SpinMutexGuard<'_, T> {
    /// Releases the lock when the guard goes out of scope.
    ///
    /// This uses Release ordering to ensure all operations within the critical
    /// section complete before the lock is released.
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}