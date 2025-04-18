use riscv::register::sstatus;

use crate::processor::{self, get_current_processor, ProcessorLocal};


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InterruptState {
    Enabled,
    Disabled,
}

impl From<InterruptState> for bool {
    fn from(value: InterruptState) -> Self {
        match value {
            InterruptState::Enabled => true,
            InterruptState::Disabled => false,
        }
    }
}

impl From<bool> for InterruptState {
    fn from(value: bool) -> Self {
        match value {
            true => InterruptState::Enabled,
            false => InterruptState::Disabled,
        }
    }
}

pub struct InterruptController;

impl InterruptController {
    #[inline]
    pub fn global_enable() {
        log::debug!("enable intterupt");
        unsafe { sstatus::set_sie(); }
    }

    #[inline]
    pub fn global_disable() {
        unsafe { sstatus::clear_sie(); }
        log::debug!("disable intterupt");
    }

    pub fn intr_disable_nested() {
        log::debug!("disable interrrupt nested");

        let processor = get_current_processor();

        let old_intr_state = InterruptController::get_state();
        InterruptController::global_disable();
        let old_nest_cnt = processor.increment_nest();

        log::debug!("old_nest_cnt: {}", old_nest_cnt);

        if old_nest_cnt == 0 {
            processor.set_saved_interrupt_state(old_intr_state);
        }
    }

    pub fn intr_enable_nested() {
        let processor = get_current_processor();
        let old_nest_cnt = processor.decrement_nest();
        if old_nest_cnt == 1 { // Last guard going out of scope
            InterruptController::set_state(processor.get_saved_interrupt_state());
        }

        log::debug!("enable interrrupt nested");
    }

    
    pub fn get_state() -> InterruptState {
        //Turn
        if sstatus::read().sie() {
            InterruptState::Enabled
        } else {
            InterruptState::Disabled
        }
    }

    pub fn set_state(state: InterruptState) {
        match state {
            InterruptState::Enabled => {Self::global_enable();},
            InterruptState::Disabled => {Self::global_disable();}
        }
    }

}
