#![no_std]
#![no_main]

use user::println;


#[no_mangle]
unsafe fn main() -> i32 {
    println!("testing store fault!");
    println!("Into Test store_fault, we will insert an invalid store operation...");
    println!("Kernel should kill this application!");
    unsafe {
        core::ptr::null_mut::<u8>().write_volatile(0);
    }
    println!("testing store fault OK!");
    0
}