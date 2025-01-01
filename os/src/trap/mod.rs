//! Trap handling module for RISC-V
//! 
//! This module sets up the trap handling mechanism for a RISC-V system, 
//! including initializing the `stvec` register to point to the trap entry 
//! and defining the trap handler logic.

mod context;

use riscv::register::utvec::TrapMode;
use riscv::register::{scause, stval, stvec};
use riscv::register::scause::{Exception, Interrupt};

use crate::syscall::syscall;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::set_next_trigger;
use crate::global_asm;

// Include the trap assembly implementation.
global_asm!(include_str!("trap.S"));

/// Initialize the CSR `stvec` to point to the trap entry `__alltraps`.
pub fn init() {
    extern "C" {
        /// You can find it in [trap.S](https://github.com/xiaomo-xty/xux-core/blob/main/os/src/trap/trap.S)
        fn __alltraps();
    }
    unsafe {
        // Set `stvec` to the address of `__alltraps` with direct mode.
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

/// The main trap handler function.
///
/// Handles various traps (e.g., exceptions and system calls) and performs the appropriate actions.
/// Then return to `trap.S` and continue from `__restore`
/// 
/// # Arguments
/// - `ctx`: A mutable reference to the `TrapContext`, which contains the current context of the application.
///
/// # Returns
/// - Returns a mutable reference to the updated `TrapContext`.
/// - Then return to `trap.S` and continue from `__restore` 
#[no_mangle]
pub fn trap_handler(ctx: &mut TrapContext) -> &mut TrapContext {
    // Read the trap cause and trap value from CSR registers.
    let scause = scause::read();
    let stval = stval::read();

    use scause::Trap;

    match scause.cause() {
        // Handle system calls.
        Trap::Exception(Exception::UserEnvCall) => {
            // Advance the program counter to skip the ecall instruction.
            ctx.sepc += 4;
            // Perform the system call and store the result in `a0`.
            ctx.x[10] = syscall(ctx.x[17], [ctx.x[10], ctx.x[11], ctx.x[12]]) as usize;
        },

        // Handle store-related faults.
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            log::error!("Page Fault in application, kernel killed it."); 
            // Run the next application as the current one is terminated.
            exit_current_and_run_next();
        },

        // Handle illegal instructions.
        Trap::Exception(Exception::IllegalInstruction) => {
            log::error!("Illegal instruction in application, kernel killed it.");
            exit_current_and_run_next();
        },

        // Handle unknown exceptions.
        Trap::Exception(Exception::Unknown) => {
            panic!("Unknown exception encountered!");
        },

        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        },

        // Handle unsupported traps.
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    // Return the updated trap context.
    // And then return to trap.S 
    // and continue from __restore 
    ctx
}

use riscv::register::sie;

pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer();}
}


pub use context::TrapContext;