/// This module defines the panic handler and other language items required for the runtime of the program.
///
/// The `lang_items` module is responsible for implementing the necessary functions and handlers
/// that are part of Rust's language items, such as the panic handler. It allows you to define custom
/// behavior for the program when a panic occurs, ensuring the program can shut down gracefully
/// or perform other custom operations when an error occurs.

use core::panic::PanicInfo;
use alloc::vec::Vec;

use crate::{println, sbi::shutdown, tools::backtrace::trace};

/// Custom panic handler that is triggered when the program encounters a panic.
///
/// The `#[panic_handler]` attribute tells the Rust compiler that this function should be called
/// when a panic occurs. This implementation customizes how panics are handled by printing relevant
/// panic information (such as the file and line number) and then invoking a system shutdown.
/// This is useful for systems without a standard library (`no_std` environment), where you need
/// a custom mechanism to handle panics gracefully.
///
/// # Parameters
/// - `info`: A reference to a `PanicInfo` structure that contains details about the panic.
///   This includes the message, file, and line number where the panic occurred.
///
/// # Behavior
/// - If the panic contains location information (i.e., file and line), it is printed.
/// - If no location is available, only the panic message is printed.
/// - The system is then shut down by calling the `shutdown` function.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {

    
    // Check if panic has a location (file and line number) information
    if let Some(location) = info.location() {
        // If panic occurred in a specific location, print the file, line, and the message
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        // If no location information is available, just print the message
        println!("Panicked: {}", info.message());
    }

    // 收集栈回溯
    let backtrace = trace(7);

    // 打印回溯信息
    println!("Backtrace ({} frames):", backtrace.len());
    for (i, frame) in backtrace.iter().enumerate() {
        println!("  #{:02} fp={:#x} ra={:#x}", i, frame.fp, frame.ra);
    }

    // Call shutdown function from the SBI to halt the system
    // The argument `true` indicates that the shutdown should be initiated due to a panic
    shutdown(true)
}




