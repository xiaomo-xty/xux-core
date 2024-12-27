//! SBI Module
//!
//! This module provides a high-level interface for interacting with the
//! Supervisor Binary Interface (SBI) by encapsulating the `sbi_rt` crate.
//! It offers functions for basic console output and other SBI-related
//! functionalities, simplifying low-level operations for users.
//!
//! The primary purpose of this module is to facilitate communication with
//! the underlying hardware in a RISC-V environment while abstracting away
//! the complexities of direct `sbi_rt` usage. By utilizing this module,
//! developers can easily integrate SBI functionalities into their RISC-V
//! applications, enhancing code clarity and maintainability.
//!
//! # Examples
//!
//! ```rust
//! // Example usage of console_putchar function
//! let s = "Hello world!";
//! for i in s.chars() {
//!    sbi::console_putchar(i as usize);
//! }
//! ```
//! Output:
//! ```bash
//! Hello world!
//! ```



/// Writes a character to the console.
///
/// This function is a wrapper around the deprecated `sbi_rt::legacy::console_putchar`
/// function. It takes a character represented as a `usize` and sends it to the console
/// for output. Note that this function relies on the legacy implementation from the `sbi_rt`
/// crate, which may be deprecated in future versions.
///
/// # Parameters
///
/// - `c`: The character to be written to the console, represented as a `usize`.
///
/// # Example
///
/// ```rust
/// sbi::console_putchar('A' as usize); // Outputs 'A' to the console
/// ```
pub fn console_putchar(c: usize) {
    #[allow(deprecated)]
    sbi_rt::legacy::console_putchar(c);
}


/// Initiates a system shutdown.
///
/// This function performs a system reset, with the option to indicate a failure condition.
/// If `failure` is `false`, the system resets without any reason. If `failure` is `true`, 
/// it signals a system failure during the reset process.
///
/// This function does not return; it will panic if reached due to an error in the system reset.
///
/// # Parameters
///
/// - `failure`: A boolean flag indicating whether the shutdown is due to a failure. If `true`, 
///              it triggers a failure reason for the reset; otherwise, it resets without reason.
///
/// # Example
///
/// ```rust
/// // Perform a normal shutdown
/// sbi::shutdown(false);
///
/// // Perform a shutdown due to failure
/// sbi::shutdown(true);
/// ```
pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}


/// Sets the timer for the next event using the specified absolute time.
///
/// This function schedules the next timer interrupt at the given absolute time 
/// specified by the `timer` parameter. The value of `timer` is typically in 
/// terms of CPU clock cycles. It also ensures that any pending timer interrupt 
/// is cleared as required by the RISC-V SBI specification.
///
/// # Parameters
/// - `timer`: An absolute time value (in CPU clock cycles) at which the next 
///   timer interrupt should occur.
///
/// # Notes
/// - If an infinite delay is desired (i.e., to clear the timer interrupt without 
///   scheduling a new one), the `timer` value can be set to `(usize::MAX)`.
/// - This function internally uses the `set_timer` call from the RISC-V SBI runtime.
///
/// # Example
/// ```
/// set_timer(1_000_000); // Schedule an interrupt after 1,000,000 clock cycles.
/// ```
pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as _);
}

