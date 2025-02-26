use riscv::register::sstatus::{
    self, Sstatus, SPP
};

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
#[derive(Clone, Copy)]
#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],   // General-purpose registers
    pub sstatus: Sstatus, // Supervisor status register
    pub sepc: usize,      // Special register (could be used for the program counter)
    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub trap_handler: usize,
}

// use crate::batch::TrapContext;
impl TrapContext {
    /// Set the stack pointer (SP) for the current context.
    ///
    /// # Arguments
    /// - `sp`: The stack pointer value to be set.
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp; // x[2] corresponds to the `sp` register in RISC-V.
    }

    /// Initialize a new application context.
    ///
    /// # Arguments
    /// - `entry`: The entry point of the application.
    /// - `sp`: The initial stack pointer value for the application.
    ///
    /// # Returns
    /// - A `TrapContext` initialized for the application with the given entry point and stack pointer.
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        // Read the current sstatus register.
        let mut sstatus = sstatus::read();
        // Set the Supervisor Previous Privilege (SPP) field to User mode.
        sstatus.set_spp(SPP::User);
        
        // Create a new TrapContext with the specified entry point and stack pointer.
        let mut ctx = Self {
            x: [0; 32],     // Initialize all general-purpose registers to 0.
            sstatus,        // Set the sstatus register with updated SPP.
            sepc: entry,    // Set the program counter (PC) to the entry point.
        };
        
        // Set the stack pointer for the context.
        ctx.set_sp(sp);
        ctx
    }
}
