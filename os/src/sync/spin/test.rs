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


