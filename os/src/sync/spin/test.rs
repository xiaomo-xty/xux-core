#![cfg(feature = "test")]

use crate::sync::spin::mutex::SpinMutex;
use os_macros::kernel_test;


#[kernel_test]
fn basic_lock_unlock() {
    let mutex = SpinMutex::new(0);
    {
        let mut guard = mutex.lock();
        *guard = 42;
    } // 守卫离开作用域，自动解锁
    let guard = mutex.lock();
    assert_eq!(*guard, 42);
}

#[kernel_test]
fn try_lock_fails_when_locked() {
    let mutex = SpinMutex::new(0);
    let guard1 = mutex.lock();
    assert!(mutex.try_lock().is_none());
    drop(guard1); // 显式释放锁
    assert!(mutex.try_lock().is_some());
}


