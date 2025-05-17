//! This is the first user proc
//! other user proc start by it

#![no_std]
#![no_main]

use user::println;

#[no_mangle]
fn main() -> i32{
    loop {
        println!("init_proc");
    }
    0
}