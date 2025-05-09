use alloc::vec::Vec;

// src/backtrace.rs
pub struct Frame {
    pub fp: usize,
    pub ra: usize,
}

/// 遍历栈帧并收集返回地址
#[inline(never)]
pub fn trace(max_depth: usize) -> Vec<Frame> {
    let mut frames = Vec::new();
    let mut current_fp: usize;

    // 获取初始帧指针 (RISC-V 使用 s0)
    unsafe { core::arch::asm!("mv {}, s0", out(reg) current_fp) };

    for _ in 0..max_depth {
        // 终止条件：无效帧指针
        if current_fp == 0 || !is_valid_address(current_fp) {
            log::debug!("isn't valid, current_fp: 0x{:x}", current_fp);
            break;
        }

        // 获取返回地址 (RISC-V: fp - 8)
        let ra = unsafe { (current_fp as *const usize).sub(1).read_volatile() };
        frames.push(Frame { fp: current_fp, ra });

        // 上一级帧指针 (RISC-V: fp - 16)
        current_fp = unsafe { (current_fp as *const usize).sub(2).read_volatile() };
    }

    frames
}

extern "C" {
    fn boot_stack_top();
    fn boot_stack_lower_bound();
}

/// 地址有效性检查（示例）
fn is_valid_address(addr: usize) -> bool {
    // 根据具体内存布局设置地址范围
    let STACK_START: usize = boot_stack_lower_bound as usize;
    let STACK_END: usize = boot_stack_lower_bound as usize;
    // (addr >= STACK_START) && (addr <= STACK_END)
    true
}