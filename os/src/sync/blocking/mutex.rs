use core::{cell::UnsafeCell, sync::atomic::{AtomicU32, Ordering}};

pub struct Mutex<T> {
    state: AtomicU32,
    data: UnsafeCell<T>,
}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}


impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        unimplemented!()
        // while 1 == self.state.swap(1, Ordering::Acquire) {
        //     wait(&self.state, 1);
        // }

        // MutexGuard { mutex: self }
    }
}
