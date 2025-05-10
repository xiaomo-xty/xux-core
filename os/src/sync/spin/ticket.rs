use core::sync::atomic::{AtomicUsize, Ordering};

use lock_api::{GuardSend, RawMutex};
use crate::{interupt::InterruptController, processor::current_processor_id};

/// A ticket-based mutex that ensures FIFO ordering for lock acquisition.
///
/// This provides fair synchronization by assigning each thread a "ticket"
/// and only allowing the thread with the matching "now serving" ticket
/// to acquire the lock.
pub type TicketMutex<T> = lock_api::Mutex<RawTicketMutex, T>;

/// An interrupt-safe variant of `TicketMutex` that disables interrupts.
pub type IRQTicketMutex<T> = lock_api::Mutex<RawIrqTicketMutex, T>;

/// Guard types for the mutex variants
pub type TicketMutexGuard<'a, T> = lock_api::MutexGuard<'a, RawTicketMutex, T>;
pub type IRQTicketMutexGuard<'a, T> = lock_api::MutexGuard<'a, RawIrqTicketMutex, T>;

/// Raw implementation of the ticket-based mutex
pub struct RawTicketMutex {
    next_ticket: AtomicUsize,      // Next available ticket number
    now_serving: AtomicUsize,      // Currently allowed ticket number
    // #[cfg(debug_assertions)]
    // holder_id: AtomicUsize,       // Debug: Track lock holder CPU ID
}

impl RawTicketMutex {
    #[cfg(debug_assertions)]
    const NO_HOLDER: usize = usize::MAX;

    /// Check for recursive locking in debug mode
    #[inline]
    fn check_deadlock(&self) {
        // #[cfg(debug_assertions)]
        // {
        //     let holder = self.holder_id.load(Ordering::Relaxed);
        //     if holder != Self::NO_HOLDER && holder == current_processor_id().into() {
        //         panic!("Recursive locking detected on CPU {}", holder);
        //     }
        // }
    }
}

unsafe impl RawMutex for RawTicketMutex {
    const INIT: RawTicketMutex = RawTicketMutex {
        next_ticket: AtomicUsize::new(0),
        now_serving: AtomicUsize::new(0),
        // #[cfg(debug_assertions)]
        // holder_id: AtomicUsize::new(Self::NO_HOLDER),
    };

    type GuardMarker = GuardSend;

    fn lock(&self) {
        log::debug!("ticket lock");
        #[cfg(debug_assertions)]
        self.check_deadlock();

        log::debug!("check_deadlock finish");

        // 1. Get a ticket (FIFO guarantee)
        let my_ticket = self.next_ticket.fetch_add(1, Ordering::Relaxed);

        log::debug!("prepare loop");

        // 2. Spin until it's our turn
        while self.now_serving.load(Ordering::Acquire) != my_ticket {
            core::hint::spin_loop();
        }

        log::debug!("store holder_id");

        // #[cfg(debug_assertions)]
        // self.holder_id.store(current_processor_id().into(), Ordering::Relaxed);

        log::debug!("ticket lock completed");
    }

    fn try_lock(&self) -> bool {
        #[cfg(debug_assertions)]
        self.check_deadlock();

        let next = self.next_ticket.load(Ordering::Relaxed);
        if self.now_serving.load(Ordering::Acquire) == next {
            self.next_ticket.store(next + 1, Ordering::Relaxed);
            // #[cfg(debug_assertions)]
            // self.holder_id.store(current_processor_id().into(), Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    unsafe fn unlock(&self) {
        log::debug!("ticket lock unlock");
        #[cfg(debug_assertions)]
        // self.holder_id.store(Self::NO_HOLDER, Ordering::Relaxed);

        // Advance to next ticket
        self.now_serving.fetch_add(1, Ordering::Release);
        log::debug!("ticket unlock completed");
    }
}



/// Interrupt-disabling version of the ticket mutex
pub struct RawIrqTicketMutex {
    inner: RawTicketMutex,
}

unsafe impl RawMutex for RawIrqTicketMutex {
    const INIT: RawIrqTicketMutex = RawIrqTicketMutex {
        inner: RawTicketMutex::INIT,
    };

    type GuardMarker = GuardSend;

    fn lock(&self) {
        InterruptController::intr_disable_nested();
        self.inner.lock();
    }

    fn try_lock(&self) -> bool {
        if self.inner.try_lock() {
            InterruptController::intr_disable_nested();
            true
        } else {
            false
        }
    }

    unsafe fn unlock(&self) {
        self.inner.unlock();
        InterruptController::intr_enable_nested();
    }
}