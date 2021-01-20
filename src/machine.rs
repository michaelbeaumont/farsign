use crate::hal::{pac::TIM2, prelude::*, rcc::Rcc, time::MicroSeconds, timer::Timer};

#[derive(PartialEq, Eq)]
pub enum StateChange {
    LongPress,
    Transmit,
    NewLetter(u8),
}

pub struct MorseMachine {
    timer: Timer<TIM2>,
}

impl MorseMachine {
    pub fn new(timer: TIM2, rcc: &mut Rcc, tick: MicroSeconds) -> Self {
        Self {
            timer: timer.timer(tick, rcc),
        }
    }

    pub fn press(&mut self) {
        self.timer.listen();
    }

    pub fn release(&mut self) {
        // We still need the timer running for the timeout
    }

    pub fn tick(&mut self) -> Option<StateChange> {
        self.timer.clear_irq();
        None
    }
}
