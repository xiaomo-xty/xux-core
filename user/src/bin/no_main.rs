#![no_std]
#![no_main]

use user::println;

#[no_mangle]
fn mian() -> i32{
    println!("test no main");
    println!("hello world");
    0
}