// Platform related
#[cfg(feature = "sv39")]
mod arch_config {
    pub const PA_WIDTH: usize = 56;      // Sv39 物理地址宽度
    pub const VA_WIDTH: usize = 39;      // Sv39 虚拟地址宽度
}

#[cfg(feature = "sv48")]
mod arch_config {
    pub const PA_WIDTH: usize = 56;      // Sv48 物理地址宽度
    pub const VA_WIDTH: usize = 48;      // Sv48 虚拟地址宽度
}

// 导出配置
#[allow(unused)]
pub use arch_config::*;


pub const PAGE_SIZE_BITS: usize = 12;
pub const PAGE_SIZE : usize = 1 << PAGE_SIZE_BITS;


// ================================================================================================
pub const HIGH_BITS_WIDTH: usize = usize::BITS as usize - VA_WIDTH;
pub const USER_HIGH_BIT: usize = VA_WIDTH - 1;
pub const KERNEL_HIGH_BIT: usize = VA_WIDTH - 1;


pub const HIGH_BITS_MASK: usize = ((1 << HIGH_BITS_WIDTH) - 1) << VA_WIDTH;
pub const VALID_USER_HIGH_BITS: usize = 0;
pub const VALID_KERNEL_HIGH_BITS: usize = HIGH_BITS_MASK;


pub const USER_STACK_SIZE: usize = 1 * PAGE_SIZE;      // Size of the user stack (8 KiB)
pub const KERNEL_STACK_SIZE: usize = 2 * PAGE_SIZE;    // Size of the kernel stack (8 KiB)

// The half of k210 SRAM
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
// pub const KERNEL_HEAP_SIZE: usize = 0x10_00;

// The memory size of K210 is 8MiB
pub const MEMORY_END: usize = 0x80800000;





// Virtual Address

pub const MAX_VA: usize = (1 << VA_WIDTH) - 1;

pub const VPN_WIDTH: usize = VA_WIDTH - PAGE_SIZE_BITS;
pub const PPN_WIDTH: usize = PA_WIDTH - PAGE_SIZE_BITS;
pub const SATP_ROOT_PPN_BITS: usize = 44;
   
pub const PA_MASK: usize = (1 << PA_WIDTH) - 1;
pub const VA_MASK: usize = (1 << VA_WIDTH) - 1;
pub const PPN_MASK: usize = (1 << PPN_WIDTH) - 1;
pub const VPN_MASK: usize = (1 << VPN_WIDTH) - 1;
pub const OFFSET_MASK: usize = PAGE_SIZE - 1;
pub const SATP_PPN_MASK: usize = (1 << SATP_ROOT_PPN_BITS) - 1;



pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;



// Return (bootom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}




/*
#[cfg(feature = "board_k210")]
pub const CLOCK_FREQ: usize = 403000000 / 62;

#[cfg(feature = "board_qemu")]
pub const CLOCK_FREQ: usize = 12500000;
*/
pub use crate::board::CLOCK_FREQ;



