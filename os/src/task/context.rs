//! Implementation of [`TaskContext`]

use crate::trap::trap_return;

/// TaskContext Layout in Memory:
/// ```text
/// ┌───────────────────────────────────────┐
/// │       return address                  │ <- offset 0  (ra)  
/// │ (e.g., __restore in __switch)         │
/// ├───────────────────────────────────────┤
/// │       stack pointer                   │ <- offset 8  (sp)
/// │ (kernel stack pointer of app)         │
/// ├───────────────────────────────────────┤ <- offset 16 (s[0..11]) 
/// │ ┌─────────────────────┐               │        (callee saved registers: s0..s11)
/// │ │   saved register s0 │ <- offset 16  │ 
/// │ ├─────────────────────┤               │
/// │ │   saved register s1 │ <- offset 24  │ 
/// │ ├─────────────────────┤               │
/// │ │         ...         │               │
/// │ ├─────────────────────┤               │
/// │ │  saved register s11 │ <- offset 104 │ 
/// │ └─────────────────────┘               │
/// └───────────────────────────────────────┘
/// ```
#[repr(C)]
pub struct TaskContext {
    /// return address ( e.g. __restore ) of __switch ASM function
    ra: usize,
    /// kernel stack pointer of app
    sp: usize,
    /// callee saved registers:  s 0..11
    s: [usize; 12],
}


impl TaskContext {
    /// `zero_init` method creates a zero-initialized `TaskContext` instance.
    /// All fields, including the return address `ra`, stack pointer `sp`, and saved registers `s`, are initialized to zero.
    ///
    /// # Returns
    /// A zero-initialized `TaskContext` instance.
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    /// set a Task Context
    /// ```rust
    /// TaskContext
    /// {
    ///     __restore ASM funciton: trap_return, 
    ///     sp: kstack_ptr, 
    ///     s: s_0..12
    /// }
    /// ```
    pub fn goto_trap_return(kernel_stack_top: usize)  -> Self {
        Self {
            ra: trap_return as usize,
            sp: kernel_stack_top,
            s: [0; 12],
        }
    }

    
}
