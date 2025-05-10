#![no_std]
#![no_main]

use user::println;


#[no_mangle]
fn main() -> i32{
    println!("dead3 loop");
    let mut x = 1;
    loop {
        x = x + 1;
        if x % 180 == 0 {
            // println!("dead loop 2: x = {}", x)
        }
    }
    0
}