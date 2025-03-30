//! Trap handling module for RISC-V
//! 
//! This module sets up the trap handling mechanism for a RISC-V system, 
//! including initializing the `stvec` register to point to the trap entry 
//! and defining the trap handler logic.

mod context;

use core::arch::asm;

use riscv::register::utvec::TrapMode;
use riscv::register::{scause, stval, stvec};
use riscv::register::scause::{Exception, Interrupt};

use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::syscall::syscall;
use crate::task::{current_trap_ctx, current_user_token, exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::set_next_trigger;
use crate::global_asm;

use riscv::register::sie;

pub fn enable_timer_interrupt() {
    unsafe { sie::set_stimer();}
}


// Include the trap assembly implementation.
global_asm!(include_str!("trap.S"));

/// Initialize the CSR `stvec` to point to the trap entry `__alltraps`.
pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe  {
        stvec::write(
            trap_from_kernel as usize, 
            TrapMode::Direct
        );
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(
            TRAMPOLINE as usize, 
            TrapMode::Direct
        );
    }
}


/// # Trap Handler Function
///
/// This function is the primary trap handler invoked by the `trap.S` assembly code via the 
/// `TrapContext.trap_handler` entry point. It processes various trap types (exceptions, interrupts, 
/// system calls) and delegates actions based on the trap cause.
///
/// ## Key Responsibilities:
/// 1. ​**Trap Context Initialization**:
///    - Sets the kernel trap entry via `set_kernel_trap_entry()`.
///    - Retrieves the current trap context (`TrapContext`) from the kernel stack.
/// 2. ​**Trap Cause Identification**:
///    - Reads `scause` (trap cause) and `stval` (trap value) CSR registers.
/// 3. ​**Trap Handling**:
///    - Matches the trap cause to predefined categories (system calls, page faults, illegal instructions, etc.).
///    - Executes appropriate actions (e.g., system call dispatch, application termination).
/// 4. ​**Context Return**:
///    - Transfers control back to the assembly trampoline (`trap_return()`) to restore user context.
///
/// ## Safety
/// - This function is marked `#[no_mangle]` to ensure its symbol is preserved for assembly linkage.
/// - Directly manipulates low-level CPU state and context registers. Must only be called by the trap entry assembly.
///
#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let ctx = current_trap_ctx();
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
        Trap::Exception(Exception::StoreFault) 
        | Trap::Exception(Exception::StorePageFault) 
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
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
    trap_return()
}




#[no_mangle]
/// Transition from kernel mode back to user mode with restored context.
///
/// # Key Mechanism: TRAMPOLINE & Virtual Address Consistency
///
/// 1. ​**TRAMPOLINE Design Purpose**:
///    - A fixed virtual address region (e.g., 0xFFFF_FFFF_FFFF_F000) that is mapped to
///      the same physical page in both kernel and all user page tables.
///    - Ensures continuity of code execution after page table switching.
///
/// 2. ​**Offset Calculation Rationale**:
///    - `__alltraps` (trap entry) and `__restore` (restore code) are placed consecutively
///      in a shared physical page. 
///    - `offset = __restore - __alltraps` calculates their relative position in physical memory.
///    - `restore_va = TRAMPOLINE + offset` translates this to the fixed virtual address
///      valid in any page table context.
///
/// # Execution Flow
/// 1. Prepare user page table (SATp) and trap context pointer.
/// 2. Calculate restore address in TRAMPOLINE's virtual space.
/// 3. Jump to TRAMPOLINE-restore code which:
///    - Restores user registers from TrapContext
///    - Switches to user page table
///    - Returns to user mode via `sret`
///
/// # Safety
/// - Requires #[repr(C)] for predictable struct layout matching assembly expectations.
/// - TRAMPOLINE must be identity-mapped in all address spaces.
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();

    extern "C" {
        fn __alltraps();
        fn __restore();
    }

    // Calculate physical offset between trap entry and restore code
    let offset = __restore as usize - __alltraps as usize;
    let alltraps_va = TRAMPOLINE;
    // Translate to fixed virtual address in TRAMPOLINE space
    let restore_va = alltraps_va + offset;
    log::debug!("go to restore (va = 0x{:X})", restore_va);
    unsafe {
        asm!(
            "fence.i",          // Ensure previous memory ops are visible
            "jr {restore_va}",  // Jump to TRAMPOLINE-mapped restore code
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,  // a0: TrapContext pointer
            in("a1") user_satp,    // a1: user SATP value
            options(noreturn)
        );
    }
}


#[no_mangle]
// Unimplement: traps/interrupts/exceptions
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel!")
}


pub use context::TrapContext;