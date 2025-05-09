#![allow(unused)]

// 读取 tp 寄
pub struct Tp;

impl Tp {
    #[inline]
    pub fn read() -> usize {
        let tp;
        unsafe { 
            core::arch::asm!("mv {}, tp", 
            out(reg) tp)
        };
        tp
    }

    #[inline]
    pub fn write(value: usize) {
        unsafe { 
            core::arch::asm!("mv tp, {}", 
            in(reg) value) 
        };
    }
}

pub struct Sstatus;

impl Sstatus {
    #[inline]
    pub fn write(value: usize) {
        unsafe {
            core::arch::asm!("csrw sstatus, {0}", in(reg) value);
        }
    }
}