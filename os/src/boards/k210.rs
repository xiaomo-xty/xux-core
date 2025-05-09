

pub const CLOCK_FREQ: usize = 400_000_000; // 400Mhz


/// [start, size]
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
];