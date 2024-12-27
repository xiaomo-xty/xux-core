pub const MAX_APP_NUM: usize = 8;                // Maximum number of applications
pub const APP_BASE_ADDRESS: usize = 0x80400000;   // Base address where applications are loaded
pub const APP_SIZE_LIMIT: usize = 0x20000;        // Limit for the size of each application
pub const USER_STACK_SIZE: usize = 4096;      // Size of the user stack (8 KiB)
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;    // Size of the kernel stack (8 KiB)

/*
#[cfg(feature = "board_k210")]
pub const CLOCK_FREQ: usize = 403000000 / 62;

#[cfg(feature = "board_qemu")]
pub const CLOCK_FREQ: usize = 12500000;
*/
pub use crate::board::CLOCK_FREQ;
