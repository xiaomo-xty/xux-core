use riscv::register::time;
use crate::{config::CLOCK_FREQ, sbi::set_timer};


mod syscall;
mod intr_req;

const TICKS_PER_SEC: usize = 100;
const MICRO_PER_SEC: usize = 1_000_000;


pub use intr_req::interrupt_request_handler;


pub fn get_time() -> usize {
    time::read()
}


/// Sets the next timer interrupt at a fixed interval (e.g., 10ms).
///
/// The function `set_next_trigger` works by first reading the current value 
/// of the machine timer (`mtime`), then computing the time interval for the 
/// next interrupt (e.g., 10ms) based on the constants `CLOCK_FREQ` and 
/// `TICKS_PER_SEC`. The timer interrupt is scheduled by setting the `mtimecmp` 
/// register to the sum of the current `mtime` value and the computed increment 
/// (which corresponds to the desired time interval).
///
/// The constant `CLOCK_FREQ` represents the clock frequency of the platform 
/// (in Hz), i.e., the number of clock cycles per second. The value of 
/// `CLOCK_FREQ` is typically determined at compile time based on the target 
/// hardware platform. The constant `TICKS_PER_SEC` defines how many timer 
/// interrupts are expected to occur per second. By dividing `CLOCK_FREQ` by 
/// `TICKS_PER_SEC`, we compute the increment value that will correspond to 
/// the desired timer interrupt frequency.
///
/// This process ensures that a timer interrupt is triggered after a fixed 
/// interval (e.g., every 10ms), enabling periodic tasks to be scheduled at 
/// the specified frequency.
///
/// # Constants:
/// - `CLOCK_FREQ`: The platform's clock frequency in Hz (cycles per second).
/// - `TICKS_PER_SEC`: The number of timer ticks per second (e.g., 100).
///
/// # Example:
/// If `CLOCK_FREQ` is 1,000,000 Hz (1 MHz) and `TICKS_PER_SEC` is 100, 
/// the increment value will be 10,000 clock cycles, meaning the next timer 
/// interrupt will occur 10ms after the current time.

pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC);
}

/// Returns the current time **in microseconds (µs)**.
///
/// This function reads the current system time in clock cycles using `time::read()`, 
/// and then converts that value into microseconds by dividing by the number of 
/// clock cycles per microsecond. The conversion factor is calculated by dividing 
/// the system's clock frequency (`CLOCK_FREQ`) by the number of microseconds in a 
/// second (`MICRO_PER_SEC`).
///
/// # Constants:
/// - `CLOCK_FREQ`: The platform's clock frequency in Hz (cycles per second).
/// - `MICRO_PER_SEC`: The number of microseconds per second (1,000,000).
///
/// # Example:
/// If `CLOCK_FREQ` is 1,000,000 (1 MHz), this function will return the current 
/// time in microseconds by converting the clock cycle count returned by `time::read()`.
pub fn get_time_us() -> usize {
    time::read() / (CLOCK_FREQ / MICRO_PER_SEC)
}

