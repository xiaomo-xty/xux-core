//! A readers-writer lock implementation based on atomic operations.
//!
//! This provides concurrent read access and exclusive write access to protected data,
//! using spin-waiting for synchronization. Suitable for read-heavy workloads in
//! no_std environments.

use core::sync::atomic::{AtomicU32, Ordering};
use lock_api::GuardSend;

/// The raw implementation of a readers-writer lock.
///
/// Uses an atomic u32 to track state:
/// - High 16 bits: reader count (supports up to 65535 concurrent readers)
/// - Low 16 bits: writer flag (0 = no writer, 1 = writer present)
///
/// # Safety
/// - Must ensure proper memory ordering (Acquire/Release) for synchronization
pub struct RawRwLock(AtomicU32);

/// A readers-writer lock type providing shared read access and exclusive write access.
///
/// This is the user-facing wrapper around [`RawRwLock`]. It implements Rust's
/// standard locking APIs through the `lock_api` crate.
///
/// # Example
/// ```
/// let lock = RwLock::new(0);
/// {
///     let read_guard = lock.read(); // Multiple readers allowed
///     println!("Value: {}", *read_guard);
/// }
/// {
///     let mut write_guard = lock.write(); // Exclusive write access
///     *write_guard += 1;
/// }
/// ```
pub type RWLock<T> = lock_api::RwLock<RawRwLock, T>;

/// A guard that provides shared read access to the data protected by [`RwLock`].
///
/// Multiple read guards can exist simultaneously. When the last read guard is
/// dropped, the read lock is released.
pub type RwLockReadGuard<'a, T> = lock_api::RwLockReadGuard<'a, RawRwLock, T>;

/// A guard that provides exclusive write access to the data protected by [`RwLock`].
///
/// Only one write guard can exist at a time. When the write guard is dropped,
/// the write lock is released.
pub type RwLockWriteGuard<'a, T> = lock_api::RwLockWriteGuard<'a, RawRwLock, T>;

unsafe impl lock_api::RawRwLock for RawRwLock {
    const INIT: RawRwLock = RawRwLock(AtomicU32::new(0));
    type GuardMarker = GuardSend;

    /// Acquires shared read access, spinning until available.
    ///
    /// This will:
    /// 1. Spin while a writer holds the lock
    /// 2. Atomically increment the reader count
    /// 3. Use Acquire ordering to ensure subsequent reads see the protected data
    fn lock_shared(&self) {
        let mut readers;
        loop {
            readers = self.0.load(Ordering::Relaxed);
            // Wait if a writer holds the lock (low 16 bits != 0)
            if readers & 0xFFFF != 0 {
                core::hint::spin_loop();
                continue;
            }
            // Attempt to increment reader count (high 16 bits +1)
            match self.0.compare_exchange_weak(
                readers,
                readers + (1 << 16),
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(_) => continue,
            }
        }
    }

    /// Attempts to acquire shared read access without blocking.
    ///
    /// Returns `true` if read access was granted, `false` otherwise.
    fn try_lock_shared(&self) -> bool {
        let readers = self.0.load(Ordering::Relaxed);
        if readers & 0xFFFF == 0 {
            self.0.compare_exchange(
                readers,
                readers + (1 << 16),
                Ordering::Acquire,
                Ordering::Relaxed,
            ).is_ok()
        } else {
            false
        }
    }

    /// Acquires exclusive write access, spinning until available.
    ///
    /// This will:
    /// 1. Spin while readers or another writer hold the lock
    /// 2. Set the writer flag (low 16 bits = 1)
    /// 3. Use Acquire ordering to ensure subsequent reads/writes see the protected data
    fn lock_exclusive(&self) {
        while !self.try_lock_exclusive() {
            core::hint::spin_loop();
        }
    }

    /// Attempts to acquire exclusive write access without blocking.
    ///
    /// Returns `true` if write access was granted, `false` otherwise.
    fn try_lock_exclusive(&self) -> bool {
        self.0.compare_exchange(
            0,
            1,
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_ok()
    }

    /// Releases shared read access.
    ///
    /// # Safety
    /// - Must only be called when the lock is held for reading
    unsafe fn unlock_shared(&self) {
        self.0.fetch_sub(1 << 16, Ordering::Release);
    }

    /// Releases exclusive write access.
    ///
    /// # Safety
    /// - Must only be called when the lock is held for writing
    unsafe fn unlock_exclusive(&self) {
        self.0.store(0, Ordering::Release);
    }
}