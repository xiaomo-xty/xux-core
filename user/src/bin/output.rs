#![no_std]
#![no_main]

use user::println;

#[no_mangle]
fn main() -> i32{
    println!("testing output!");

    for i in 0..10 {
        println!("{}", i);
    }
    println!("test output OK!");
    0
}