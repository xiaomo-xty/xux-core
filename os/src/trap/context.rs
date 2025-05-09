//! Implementation of [`TrapContext`]

use alloc::{format, vec::Vec};
use riscv::register::sstatus::{
    self, Sstatus, SPP
};

use crate::register;



/// Represents the context of a trap (e.g., during an interrupt or system call).
/// 
/// The layout of `TrapContext` in memory is as follows:
/// 
/// ```text
/// +--------------------------------------------------------+
/// |   TrapContext Address (stack top) = sp                 | <- offset 0x0 (stack top)
/// +--------------------------------------------------------+    (user stack pointer)
/// |                   General-purpose registers            | 
/// | ┌─────────────────────────────────────────────────────┐| <- offset 0x0
/// | │   x[0] (usize)                                      │|
/// | ├─────────────────────────────────────────────────────┤|
/// | │   x[1] (usize)                                      │|
/// | ├─────────────────────────────────────────────────────┤|
/// | │   x[2] (usize)                                      │|
/// | ├─────────────────────────────────────────────────────┤|
/// | │       ...                                           │|
/// | ├─────────────────────────────────────────────────────┤|
/// | │   x[30] (usize)                                     │|
/// | ├─────────────────────────────────────────────────────┤|
/// | │   x[31] (usize)                                     │|
/// | └─────────────────────────────────────────────────────┘|
/// +--------------------------------------------------------+ <- offset 0x100
/// |                       sstatus (Sstatus)                |    
/// | ┌─────────────────────────────────────────────────────┐|
/// | │   sstatus (usize)   (user stack pointer)            │|
/// | └─────────────────────────────────────────────────────┘|
/// |    (sstatus stores the previous stack pointer sp,      |
/// |     saved in sscratch during trap entry)               |
/// +--------------------------------------------------------+ <- offset 0x108
/// |                       spec (usize)                     |    
/// | ┌─────────────────────────────────────────────────────┐|
/// | │   spec (usize)     (return address)                 │|
/// | └─────────────────────────────────────────────────────┘|
/// |   (spec stores the return address for resuming program,|
/// |   it is restored to sepc in `__restore`` for resuming  |
/// |   the program after trap handling)                     |
/// +--------------------------------------------------------+
/// ```
#[repr(C)]
pub struct TrapContext {
    // =====================================+
    // | Save   | when (user  ) -> (kernel) |
    // +--------+---------------------------+
    // | Restore| when (kernel) -> (user)   |
    // =====================================+
    
    /// (0~31) common register
    pub x: [usize; 32],   
    /// (32) CSR sstatus
    pub sstatus: Sstatus, 
    /// (33) CSR sepc
    pub sepc: usize,    

    // =====================================+
    // | Save   | when (kernel) -> (user  ) |
    // +--------+---------------------------+
    // | Restore| when (user  ) -> (kernel) |
    // =====================================+
    /// (34) Addr of Page Table
    pub kernel_satp: usize,
    /// (35) kernel stack
    pub kernel_sp: usize,
    /// (36) kernel tp
    pub kernel_tp: usize,
    /// (37) Addr of trap_handler function.
    pub trap_handler: usize,
}

// use crate::batch::TrapContext;
impl TrapContext {


    /// Set the stack pointer (SP/X2) for the current context.
    ///
    /// # Arguments
    /// - `sp`: The stack pointer value to be set.
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp; // x[2] corresponds to the `sp` register in RISC-V.
    }

    /// Initialize a new application execution context for privilege level switching.
    ///
    /// This function prepares the trap frame when transitioning from kernel mode 
    /// to user mode, typically during process creation or context restoration.
    ///
    /// # Arguments
    /// - `entry`:    User space entry point (virtual address) where the application starts execution.
    /// - `sp`:       Initial user stack pointer (virtual address) for the application.
    /// - `kernel_satp`:  Kernel's SATP register value (page table root PFN + mode bits),
    ///                   used to restore kernel address space on trap entry.
    /// - `kernel_sp`:    Kernel stack pointer (physical address) for trap handling.
    /// - `trap_handler`: Virtual address of the kernel's trap handler function.
    ///
    /// # Returns
    /// - A fully initialized [`TrapContext`] containing:
    ///   - Clean general-purpose registers (x0-x31)
    ///   - Configured `sstatus` with privilege transition metadata
    ///   - Control registers for trap handling setup
    ///
    /// # Registers Configuration
    /// - `sepc`: Set to `entry` to define the return-to-user address after `sret`.
    /// - `sstatus`: 
    ///   - ​**SPP**​ field set to [`SPP::User`] to indicate previous privilege mode.
    ///   - ​**SIE**​ state preserved for global interrupt handling.
    /// - Kernel resources (SATP, stack, handler) stored for transparent access during traps.
    ///
    /// # Safety
    /// - Caller must guarantee the validity of memory addresses (entry, sp, trap_handler).
    /// - Kernel SATP value must point to a valid kernel page table.
    pub fn app_init_context(
        entry: usize, 
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        // 1. Configure privilege mode transition metadata
        let mut sstatus = sstatus::read();
        
        // Set Supervisor Previous Privilege (SPP) to User mode.
        // This ensures that when the application triggers a trap (exception/interrupt),
        // the hardware will set SPP to User, allowing correct return via `sret`.
        sstatus.set_spp(SPP::User);

        let kernel_tp = register::Tp::read();
        // 2. Initialize trap frame with isolation between user/kernel resources
        let mut ctx = Self {
            x: [0; 32],     // Zero-initialize general-purpose registers.
                            // x2 (sp) will be overwritten by ctx.set_sp(sp)
            
            sstatus,        // Carry SIE (Supervisor Interrupt Enable) state,
                            // preserving global interrupt configuration.
            
            sepc: entry,    // User execution starting point. 
                            // This will be loaded into PC via `sret`.
            
            kernel_satp,    // Kernel's page table configuration (SATP).
                            // Used to switch back to kernel address space on traps.
            
            kernel_sp,      // Per-CPU kernel stack for trap handling.
                            // Provides stack isolation between user/kernel modes.
            
            kernel_tp,
            
            trap_handler,   // Entry point of the kernel's trap handling routine.
                            // Stored here for fast access during trap vector setup.
        };

        // 3. Set user stack pointer in the context.
        // This utilizes the dedicated setter method to ensure correct register slot assignment.
        // (Typically writes to x2, as per RISC-V calling convention)
        ctx.set_sp(sp);

        ctx
    }
}




use core::fmt;

impl fmt::Debug for TrapContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 按你的注释分块输出
        f.debug_struct("TrapContext")
            // =====================================
            // User -> Kernel 时保存，Kernel -> User 时恢复
            // =====================================
            .field("\nx (registers)", &{
                // 自定义寄存器输出（避免显示全部32个）
                let mut regs = self.x.iter().enumerate()
                    .filter(|(i, _)| *i == 0 || *i == 1 || *i == 2 || *i >= 28)
                    .map(|(i, v)| format!("x{:02}={:#x}", i, v))
                    .collect::<Vec<_>>();
                regs.insert(3, "...".into());
                regs
            })
            .field("\nsstatus", &self.sstatus)
            .field("\nsepc", &format_args!("{:#x}", self.sepc))
            
            // =====================================
            // Kernel -> User 时保存，User -> Kernel 时恢复
            // =====================================
            .field("\nkernel_satp", &format_args!("{:#x}", self.kernel_satp))
            .field("\nkernel_sp", &format_args!("{:#x}", self.kernel_sp))
            .field("\nkernel_tp", &format_args!("{:#x}", self.kernel_tp))
            .field("\ntrap_handler\n\n", &format_args!("{:#x}", self.trap_handler))
            .finish()
    }
}