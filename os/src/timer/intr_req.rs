use crate::processor::get_current_processor;
use super::set_next_trigger;

/// Handles timer interrupt requests.
///
/// This function is called when a timer interrupt occurs. It performs two main tasks:
/// 1. Sets up the next timer interrupt by calling `set_next_trigger()`
/// 2. Notifies the scheduler about the timer tick, allowing it to perform time-related
///    scheduling operations such as:
///    - Updating process/thread time quanta
///    - Checking for timeouts
///    - Potentially triggering scheduling decisions
///
/// # Safety
/// This function should only be called from an interrupt context. It assumes:
/// - The interrupt is properly masked/disabled where necessary
/// - The scheduler and processor structures are in a valid state
///
/// # Panics
/// - If the current processor cannot be accessed
/// - If the scheduler cannot be accessed
/// - If any internal invariants are violated during the timer tick processing
///
/// # Example
/// ```no_run
/// // Typically called from an interrupt handler:
/// interrupt_request_handler();
/// ```
pub fn interrupt_request_handler() {
    log::debug!("Handle timer interrupt");
    // Set up the next timer interrupt
    set_next_trigger();
    
    // Notify the scheduler about the timer tick
    get_current_processor().scheduler.lock().timer_tick();
}