#![no_std]
#![no_main]

use user::{get_time, println, yield_};

#[no_mangle]
unsafe fn main() -> i32 {
    println!("testing sleep!");
    let current_timer = get_time();
    let wait_for = current_timer + 3000;
    while get_time() < wait_for {
        println!("prepare yield");
        yield_();
    }
    println!("Test sleep OK!");
    0
}