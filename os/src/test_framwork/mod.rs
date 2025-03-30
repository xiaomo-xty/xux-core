use crate::{println, sbi::shutdown};

/// test_runner
#[allow(unused)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        // 模拟捕获 panic
        let result = run_test(test);
        

        // 检查测试是否 panic
        if result.is_err() {
            println!("Test failed!");
        } else {
            println!("Test passed!");
        }
    }
    println!("All tests completed!");

    shutdown(true)
}

// capture panic
fn run_test(test: &dyn Fn()) -> Result<(), ()> {
    struct Guard;

    impl Drop for Guard {
        fn drop(&mut self) {
            println!("Test completed!");
        }
    }

    let _guard = Guard;

    test();
    Ok(())
}