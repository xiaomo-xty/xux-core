use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

use super::mutex::IRQSpinLockGuard;

/// 可挂起的锁，允许跨执行流传递锁的所有权
pub struct PendableLock<T> {
    // 使用 MaybeUninit 避免初始化要求
    slot: UnsafeCell<MaybeUninit<IRQSpinLockGuard<'static, T>>>,
    // 使用原子标志保证线程安全
    occupied: AtomicBool,
}

impl<T> PendableLock<T> {
    /// 创建一个新的 PendableLock
    pub const fn new() -> Self {
        Self {
            slot: UnsafeCell::new(MaybeUninit::uninit()),
            occupied: AtomicBool::new(false),
        }
    }

    /// 存储锁守卫
    /// 安全要求：调用者必须确保 guard 的实际生命周期足够长
    pub fn store_lock(&self, guard: IRQSpinLockGuard<'_, T>) {
        // 检查是否已被占用
        if self.occupied.swap(true, Ordering::Acquire) {
            panic!("PendableLock already occupied");
        }

        unsafe {
            // 写入守卫（延长生命周期到 'static）
            (*self.slot.get()).write(core::mem::transmute(guard));
        }
    }

    /// 取出锁守卫，转移所有权
    pub fn take_lock(&self) -> IRQSpinLockGuard<'_, T> {
        // 检查是否有锁
        if !self.occupied.swap(false, Ordering::Release) {
            panic!("No lock stored in PendableLock");
        }

        unsafe {
            // 读取并转移所有权
            let guard = (*self.slot.get()).as_ptr().read();
            // 标记为未初始化（避免双重释放）
            (*self.slot.get()).assume_init_drop();
            guard
        }
    }
}

// 实现 Sync 因为内部使用原子操作保证线程安全
unsafe impl<T> Sync for PendableLock<T> {}