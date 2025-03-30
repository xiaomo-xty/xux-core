#![feature(linkage)]
#![no_std]

pub mod console;
mod lang_items;
mod syscall;

use syscall::*;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    // clear_bss();
    exit(main());
    panic!("unreacheable after sys_exit!");
}


#[linkage = "weak"] //need #![feature(linkage)]
#[no_mangle]
fn main() -> i32 {
    //! Would be overwrite, if the main of user program is exist.
    panic!("Cannot find main!");
}

// fn clear_bss() {
//     extern "C" {
//         fn start_bss();
//         fn end_bss();
//     }

//     (start_bss as usize .. end_bss as usize).for_each(|addr| unsafe{
//         (addr as *mut u8).write_volatile(0);
//     });
// }

pub fn write(fd: usize, buf: &[u8]) -> isize {
    sys_write(fd, buf)
}

pub fn exit(exite_code: i32) ->! {
    sys_exit(exite_code)
}

pub fn yield_() -> isize {
    sys_yield()
}

pub fn get_time() -> isize {
    sys_get_time()
}

