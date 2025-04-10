use crate::{color_println, println, sbi::shutdown};
// use crate::io::console::{ Color, Colorize };

/// test_runner
#[allow(unused)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        // 模拟捕获 panic
        let result = test();

        // crate::io::console::color_println!(crate::io::console::Color::Green, "========[Test passed!]========");
        

    }
    color_println!(crate::io::console::Color::Green, "\n      All tests passed!");

    shutdown(true)
}
