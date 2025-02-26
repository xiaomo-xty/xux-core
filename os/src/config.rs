pub const MAX_APP_NUM: usize = 8;                // Maximum number of applications
pub const APP_BASE_ADDRESS: usize = 0x80400000;   // Base address where applications are loaded
pub const APP_SIZE_LIMIT: usize = 0x20000;        // Limit for the size of each application
pub const USER_STACK_SIZE: usize = 4096;      // Size of the user stack (8 KiB)
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;    // Size of the kernel stack (8 KiB)

pub const KERNEL_HEAP_SIZE: usize = 4096 * 2;

// Platform related
pub const PA_WIDTH_SV39: usize = 56;
pub const VPN_WIDTH_SV39: usize = 9;
pub const PPN_WIDTH_SV39: usize = PA_WIDTH_SV39 - PAGE_SIZE_BITS;

// Virtual Address
pub const PAGE_SIZE : usize = 4096;
pub const PAGE_SIZE_BITS: usize = 12;
pub const PA_WIDTH: usize = PA_WIDTH_SV39;
pub const VPN_WIDTH: usize = VPN_WIDTH_SV39;
pub const PPN_WIDTH: usize = PA_WIDTH - PAGE_SIZE_BITS;
pub const SATP_ROOT_PPN_BITS: usize = 44;

pub const PA_MASK: usize = (1 << PA_WIDTH) - 1;
pub const PPN_MASK: usize = (1 << PPN_WIDTH) - 1;
pub const VPN_MASK: usize = (1 << VPN_WIDTH) - 1;
pub const OFFSET_MASK: usize = PAGE_SIZE - 1;
pub const SATP_PPN_MASK: usize = (1 << SATP_ROOT_PPN_BITS) - 1;

// The memory size of K210 is 8MiB
pub const MEMORY_END: usize = 0x80800000;

/*
#[cfg(feature = "board_k210")]
pub const CLOCK_FREQ: usize = 403000000 / 62;

#[cfg(feature = "board_qemu")]
pub const CLOCK_FREQ: usize = 12500000;
*/
pub use crate::board::CLOCK_FREQ;


// Return (bootom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(app_id: usize) -> (usize, usize) {
    let top = TRAMPLOLINE - app_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}