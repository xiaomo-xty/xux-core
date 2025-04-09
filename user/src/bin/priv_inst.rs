#![no_std]
#![no_main]

use core::arch::asm;
use user::println;

#[no_mangle]
unsafe fn main() -> i32 {
    println!("testing priv_inst!");
    println!("Try to execute privileged instruction in U Mode");
    println!("Kernel should kill this application!");
    unsafe {
        asm!("sret");
    }
    println!("testing priv_inst OK!");
    0
}