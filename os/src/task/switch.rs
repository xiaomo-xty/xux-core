
use super::TaskContext;
use core::arch::global_asm;

global_asm!(include_str!("switch.S"));

extern "C" {
    /// You can find it in [switch.S](https://github.com/xiaomo-xty/xux-core/blob/main/os/src/task/switch.S)
    /// Performs a context switch between the current task and the next task.
    ///
    /// This function saves the context of the currently running task (e.g., stack pointer, return address,
    /// and register values) to the memory location pointed to by `current_task_cx_ptr`. It then restores
    /// the context of the next task (e.g., stack pointer, return address, and register values) from the memory
    /// location pointed to by `next_task_cx_ptr`. This allows the system to pause the execution of the current
    /// task and resume the execution of the next task at the correct point.
    ///
    /// # Parameters
    /// - `current_task_cx_ptr`: A mutable pointer to the `TaskContext` of the currently running task. This will
    ///   store the context of the current task, including the stack pointer (`sp`), return address (`ra`), and
    ///   register values (`s0` to `s11`).
    /// - `next_task_cx_ptr`: A constant pointer to the `TaskContext` of the next task to be scheduled. This will
    ///   provide the context to restore for the next task, including the stack pointer (`sp`), return address (`ra`),
    ///   and register values (`s0` to `s11`).
    ///
    /// # Safety
    /// This function is unsafe because it involves raw pointer dereferencing and manual context manipulation.
    /// Care must be taken to ensure that the task contexts provided are valid and properly aligned.
    ///
    /// 
    /// # Example
    /// ```rust
    /// let current_task_context: *mut TaskContext = ...;
    /// let next_task_context: *const TaskContext = ...;
    /// unsafe {
    ///     
    ///     __switch(current_task_context, next_task_context);
    /// }
    /// ```
    fn __switch(
        save_context_to  :   *mut TaskContext,
        load_context_from: *const TaskContext,
    );
}


/// package unsafe __switch, support more safety interface
#[inline(always)]
pub fn switch(
    save_context_to: &mut TaskContext,
    load_context_from: &TaskContext,
    // _guard: SpinMutexGuard<'_,()>
) {
    unsafe {
        __switch(
            save_context_to as *mut TaskContext, 
            load_context_from as *const TaskContext
        );
    }
}