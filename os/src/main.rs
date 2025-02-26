//! main mod

#![deny(missing_docs)]
#![deny(warnings)]
#![no_main]
#![no_std]
#![feature(alloc_error_handler)]
// #![feature(panic_info_message)]
mod console;
mod lang_iterms;
mod sbi;
mod logging;
// mod batch;
mod task;
mod sync;
mod trap;
mod syscall;
mod config;
mod loader;
mod timer;

extern crate alloc;
extern crate bitflags;
mod mm;

#[path = "boards/qemu.rs"]
mod board;

use core::arch::global_asm;


global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

/// - Would be called by `entry.asm`.
/// - Don't return.
#[no_mangle]
pub fn rust_main() -> ! {
    #[allow(unused)]
    // Read in linker.ld
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
    }
    clear_bss();
    logging::init();
    log::info!("Hello world!");
    
    log::trace!(
        ".text [{:#x}, {:#x}])",
        stext as usize,
        etext as usize,
    );

    log::debug!(
        ".rodata [{:#x}, {:#x}])",
        srodata as usize, erodata as usize
    );

    log::info!(
        ".data [{:#x}, {:#x}])",
        srodata as usize, erodata as usize
    );

    log::warn!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower_bound as usize
    );

    mm::heap_allocator::init_heap();
    mm::heap_allocator::heap_test();
    mm::page_table::test_PTEFlags();

    trap::init();
    loader::load_apps();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
    unreachable!();
    
}

/// Clears the `.bss` section by setting each byte to zero.
///
/// The `.bss` section is used to store uninitialized global and static variables,
/// which are expected to be zeroed out before the program starts running. This function
/// iterates over the memory range between the `sbss` (start of `.bss`) and `ebss`
/// (end of `.bss`) symbols and sets each byte to zero.
///
/// # Safety
///
/// This function performs unsafe memory operations by writing directly to raw
/// pointers. It is intended to be called at the beginning of the program to
/// ensure that the `.bss` section is correctly initialized
fn clear_bss() {
    extern "C" {
        // sbss and ebss is defined in `entry.asm`
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { 
            (a as *mut u8).write_volatile(0);
        }
    });
}