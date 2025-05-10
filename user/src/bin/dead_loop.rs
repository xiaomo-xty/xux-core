#![no_std]
#![no_main]

use user::println;


#[no_mangle]
fn main() -> i32{
    println!("dead1 loop");
    let mut x = 1;
    loop {
        x = x + 1;
        if x % 100 == 0 {
            println!("dead loop 1: x = {}", x)
        }
    }
    0
}