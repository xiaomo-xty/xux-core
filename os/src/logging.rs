//! A logging module that configures and outputs log messages with color coding.
//!
//! This module provides a custom logger `OSLogger` that prints log messages in different colors
//! based on their severity level (error, warn, info, debug, trace). It relies on the `log` crate
//! to capture log messages and format them using ANSI escape codes for color output in the Linux console.
//!
//! 

use core::fmt;

// use lazy_static::lazy_static;
use log::{self, Level, LevelFilter, Log, Metadata, Record};
use crate::console::print;

/// # Initialization
/// The logger is initialized using the `init` function, which sets up the logging system based on the
/// `LOG` environment variable. The available log levels are:
/// - "error" -> `LevelFilter::Error`
/// - "warn" -> `LevelFilter::Warn`
/// - "info" -> `LevelFilter::Info`
/// - "debug" -> `LevelFilter::Trace`
/// - Any other value -> `LevelFilter::Off`
///
/// This function **should only be called in the `rust_main` function**, as it sets up the logger
/// for the entire application and configures the log level based on the environment variable.
///
/// # Example
/// To use this module, simply call `init()` from `rust_main` and use the `log!` macros
/// (like `error!`, `warn!`, `info!`, etc.) for logging.
pub fn init() {
    static LOGGER: OSLogger = OSLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(
        match option_env!("LOG") {
            Some("ERROR") => LevelFilter::Error,
            Some("WARN") => LevelFilter::Warn,
            Some("INFO") => LevelFilter::Info,
            Some("DEBUG") => LevelFilter::Trace,
            _ => LevelFilter::Off,
        }
    );
}

/// A custom logger that prints log messages to the console with color coding.
///
/// This logger formats the log message based on its severity level, using ANSI escape sequences
/// for color output. It supports all log levels provided by the `log` crate.
struct OSLogger;

impl Log for OSLogger {
    /// Determines whether the log message should be processed, based on the log level.
    #[warn(unused_variables)]
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    /// Processes the log message and prints it to the console with color formatting.
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        print_in_color(
            format_args!("[KERNEL][{:>5}][0,-] {}\n", record.level(), record.args()),
            level_to_color_code(record.level())
        );
    }

    /// Flushes the log output (no-op in this case).
    fn flush(&self) {}
}

/// Adds escape sequences to the formatted string to print with a specific color.
macro_rules! with_color {
    ($args: ident, $color_code: ident) => {{
        format_args!("\u{1B}[{}m{}\u{1B}[0m", $color_code as u8, $args)
    }};
}

/// Prints the formatted output in the specified color.
///
/// This function takes the formatted string provided by `args` and prints it to
/// the output, but with the specified color applied. The color is determined
/// by the `color_code` parameter, which should be an ANSI color code (e.g.,
/// 31 for red, 32 for green, etc.). The function utilizes the `with_color!` macro
/// to format the string with the appropriate color escape sequences before printing.
///
/// # Parameters:
/// - `args`: The formatted arguments to be printed, typically in the form of `fmt::Arguments`.
/// - `color_code`: The ANSI color code to apply to the text (e.g., 31 for red).
///
/// # Example:
/// ```
/// print_in_color(format_args!("Hello, World!"), 31); // Prints "Hello, World!" in red.
/// ```
///
/// # Note:
/// This function relies on the `print` function to perform the actual output after
/// applying the color formatting.
fn print_in_color(args: fmt::Arguments, color_code: u8) {
    print(with_color!(args, color_code));
}

/// Converts a log level to the corresponding ANSI color code.
///
/// This function maps the log levels (error, warn, info, debug, trace) to the respective
/// color codes to be used in the console output:
/// - `Level::Error` -> Red (31)
/// - `Level::Warn` -> Bright Yellow (93)
/// - `Level::Info` -> Blue (34)
/// - `Level::Debug` -> Green (32)
/// - `Level::Trace` -> Bright Black (90)
fn level_to_color_code(level: Level) -> u8 {
    match level {
        Level::Error => 31, // Red
        Level::Warn => 93,  // BrightYellow
        Level::Info => 34,  // Blue
        Level::Debug => 32, // Green
        Level::Trace => 90, // BrightBlack
    }
}






