//! XUX-OS Kernel Main Module
//!
//! A teaching-oriented operating system kernel for RISC-V architecture, implementing core OS functionality including:
//! - Multitasking scheduling
//! - Virtual memory management
//! - System call interface
//! - Hardware timer drivers
//! - Logging subsystem
//!
//! # Architectural Overview
//! ```text
//! +-------------------+
//! |    User Space     |
//! +-------------------+
//! |  System Call API  |
//! +-------------------+
//! |  Task Management  |
//! +-------------------+
//! | Memory Management |
//! +-------------------+
//! | Hardware Abstraction
//! +-------------------+
//! ```
//!
//! # Safety Guarantees
//! - All unsafe blocks are properly documented with invariants
//! - Memory management maintains isolation between processes
//! - Interrupt handlers preserve register state
//!
//! # Testing Strategy
//! - Unit tests for core algorithms
//! - Integration tests for subsystem interactions
//! - QEMU-based hardware tests
//! 


// #![deny(missing_docs)]
// #![deny(warnings)]
#![no_main]
#![no_std]
#![feature(alloc_error_handler)]


// Custom test frameworks in bare metal
#![feature(custom_test_frameworks)]
// collect all test_case to the function `test_runner`
#![test_runner(test_framework::test_runner)]
#![reexport_test_harness_main = "test_main"]


// #![feature(panic_info_message)]
mod io;
mod lang_iterms;
mod sbi;
// mod batch;
mod task;
mod processor;
mod sync;
mod trap;
mod interupt;
mod syscall;
mod config;
mod loader;
mod timer;
mod register;
mod test_framework;

extern crate alloc;
mod mm;

#[path = "boards/qemu.rs"]
mod board;

use core::arch::global_asm;

use processor::get_current_processor;

// os entry
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));


/// - Would be called by `entry.asm`.
/// - Don't return.
#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    io::init();
    log::info!("Logger turn on");
    log::debug!("Debug Logger turn on");
    
    mm::init();
    mm::heap_allocator::heap_test();

    mm::memory_set::remap_test();


    trap::init();
    log::info!("Trap initialize: [success]");

    // loader::load_apps();

    syscall::init();

    log::info!("XUX-OS initilize successed!");
    print_info();
    log::debug!("print end");
    
    
    #[cfg(test)]
    test_main();

    task::init_scheduler();

    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    
    log::info!("test successed!Welcom ot xux-os!");
    {
        let processor = get_current_processor();
        let mut scheduler_guard = processor.scheduler.lock();
        scheduler_guard.run();
    }
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

fn print_info() {
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

}


