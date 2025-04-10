//! A logging module that configures and outputs log messages with color coding.
//!
//! This module provides a custom logger `OSLogger` that prints log messages in different colors
//! based on their severity level (error, warn, info, debug, trace). It relies on the `log` crate
//! to capture log messages and format them using ANSI escape codes for color output in the Linux console.
//!
//! 


// use lazy_static::lazy_static;
use log::{self, Level, LevelFilter, Log, Metadata, Record};

use crate::color_println;

use super::console::Color;

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

        let color = level_to_color(record.level());

        color_println!(
            color,"[KERNEL][{:>5}][0,-] {}\n", record.level(), record.args(),
        );
    }

    /// Flushes the log output (no-op in this case).
    fn flush(&self) {}
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
fn level_to_color(level: Level) -> Color {
    match level {
        Level::Error => Color::Red, // Red
        Level::Warn => Color::BrightYellow,  // BrightYellow
        Level::Info => Color::Blue,  // Blue
        Level::Debug => Color::Green, // Green
        Level::Trace => Color::BrightBlack, // BrightBlack
    }
}






