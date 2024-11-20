#![no_std]
#![no_main]

use user::println;

#[no_mangle]
fn mian() -> i32{
    println!("hello world");
    0
}