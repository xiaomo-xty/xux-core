//! Constants used in rCore for qemu


/// `CLOCK_FREQ` represents the frequency of the system clock that drives the `mtime`
/// register. It is a constant value that determines how often the system timer (or
/// `mtime`) increments, typically in clock cycles per second.
///
/// This frequency is used for timekeeping in the system, and is a stable, fixed value
/// that is independent of the CPU's actual operating frequency. While the CPU's clock
/// speed (or core frequency) may vary depending on load or power-saving modes, the
/// system clock frequency is constant and used for scheduling tasks, triggering timer
/// interrupts, and other time-related operations.
///
/// # Example:
/// If `CLOCK_FREQ` is 1,000,000 (1 MHz), it means that the system clock increments
/// every second by 1,000,000 cycles, which serves as the base for time calculations
/// and interrupt triggers for scheduling and task management.
pub const CLOCK_FREQ: usize = 12_500_000;

pub type BlockDeviceImpl = crate::drivers::block::VirtIOBlock;


/// [start, size]
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x1000_1000, 0x00_1000), // Virtio Block in virt machine
];