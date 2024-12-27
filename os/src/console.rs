/// This module provides printing functionality for formatted output,
/// using `console_putchar` from the SBI (Supervisor Binary Interface) to
/// output individual characters. It includes a custom `print!` and `println!`
/// macro for formatting and printing text similarly to Rust’s standard `print!`
/// and `println!` macros.

use crate::sbi::console_putchar;
use core::fmt::{self, Write};

/// A struct implementing `Write` to send characters to the console via `console_putchar`.
struct Stdout;

impl Write for Stdout {
    /// Implements `write_str` by iterating over each character in the given
    /// string `s` and sending it to `console_putchar`.
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

/// Prints formatted output to the console.
///
/// This function takes formatted arguments and sends them to `Stdout`
/// using `write_fmt`, which invokes `console_putchar` for each character.
///
/// # Parameters
/// - `args`: The formatted arguments to print, created using `format_args!`.
pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

/// Prints formatted text without a newline, similar to `print!` in the standard library.
///
/// This macro uses `format_args!` to handle the provided arguments and calls
/// the `print` function to output them.
///
/// # Usage
/// ```
/// print!("Hello, {}!", "world");
/// ```
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    };
}


/// Prints formatted text followed by a newline, similar to `println!` in the standard library.
///
/// This macro behaves like `print!` but appends a newline character to the output.
///
/// # Usage
/// ```
/// println!("Hello, {}!", "world");
/// ```
#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    };
}

