use crate::hal::pac::TIM2;
use crate::hal::{
    time::{MicroSeconds, U32Ext},
    timer::Timer,
};
use crate::morse;
use embedded_hal::timer::CountDown;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PressType {
    Short,
    Long,
    VeryLong,
}

impl PressType {
    pub fn tick(&self) -> Self {
        match self {
            Self::Short => Self::Long,
            Self::Long => Self::VeryLong,
            Self::VeryLong => Self::VeryLong,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Press(PressType),
    WaitingOnPress,
    Idle,
}

impl State {
    fn release(&mut self) -> Self {
        Self::WaitingOnPress
    }

    fn press(&mut self) -> Self {
        Self::Press(PressType::Short)
    }

    fn tick(&self) -> Self {
        match self {
            Self::WaitingOnPress => Self::Idle,
            Self::Press(p) => Self::Press(p.tick()),
            Self::Idle => Self::Idle,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Transition {
    Long,
    VeryLong,
    Character(u8),
    Transmit,
}

struct Button {
    long_press: u32,
    very_long_press: u32,
    timeout: u32,
    count: u32,
    pressed: bool,
}

impl Button {
    const fn new(long_press: u32, very_long_press: u32, timeout: u32) -> Self {
        Self {
            long_press,
            very_long_press,
            timeout,
            count: 0,
            pressed: false,
        }
    }
    fn release(&mut self) {
        self.count = 0;
        self.pressed = false;
    }

    fn press(&mut self) {
        self.count = 0;
        self.pressed = true;
    }

    fn state(&self, code_empty: bool) -> State {
        if !self.pressed {
            if self.count > self.timeout {
                State::Idle
            } else {
                State::WaitingOnPress
            }
        } else {
            State::Press(if self.count > self.very_long_press && code_empty {
                PressType::VeryLong
            } else if self.count > self.long_press {
                PressType::Long
            } else {
                PressType::Short
            })
        }
    }

    fn timeout(&mut self) {
        self.count = self.timeout;
    }

    fn tick(&mut self) {
        self.count += 1;
    }
}

pub struct MorseMachine {
    button: Button,
    current: morse::MorseCode,
}

#[allow(dead_code)]
impl MorseMachine {
    pub const fn new(dot_ticks: u32) -> Self {
        Self {
            button: Button::new(dot_ticks, 3 * dot_ticks, 3 * dot_ticks),
            current: morse::MorseCode::empty(),
        }
    }

    pub fn press(&mut self) {
        self.button.press();
    }

    pub fn release(&mut self) {
        if let State::Press(p) = self.button.state(self.current.is_empty()) {
            self.current = match p {
                PressType::Short => self.current.append_dot(),
                PressType::Long => self.current.append_dash(),
                PressType::VeryLong => morse::TRANSMIT,
            }
        }
        self.button.release();
        if self.current == morse::TRANSMIT {
            self.button.timeout();
        }
    }

    pub fn tick(&mut self) -> Option<Transition> {
        let is_empty = self.current.is_empty();
        let previous_state = self.button.state(is_empty);
        self.button.tick();
        let current_state = self.button.state(is_empty);
        match (previous_state, current_state) {
            (p, n) if p == n => None,
            (_, State::Idle) => {
                let character = self.current;
                self.current = morse::MorseCode::empty();
                Some(if character == morse::TRANSMIT {
                    Transition::Transmit
                } else {
                    Transition::Character(character.lookup())
                })
            }
            (_, State::Press(PressType::VeryLong)) => Some(Transition::VeryLong),
            (_, State::Press(PressType::Long)) => Some(Transition::Long),
            _ => None,
        }
    }
}

struct MorseTimelessMachine {
    state: State,
    current: morse::MorseCode,
}

impl MorseTimelessMachine {
    pub const fn new() -> Self {
        Self {
            state: State::Idle,
            current: morse::MorseCode::empty(),
        }
    }

    pub fn press(&mut self) -> &mut Self {
        self.state = self.state.press();
        self
    }

    pub fn release(&mut self) -> &mut Self {
        if let State::Press(p) = self.state {
            self.current = match p {
                PressType::Short => self.current.append_dot(),
                PressType::Long => self.current.append_dash(),
                PressType::VeryLong => morse::TRANSMIT,
            }
        }
        self.state = self.state.release();
        self
    }

    fn next_state(&self) -> State {
        match (self.current.is_empty(), self.state) {
            (false, State::Press(PressType::Long)) => self.state,
            (_, s) => s.tick(),
        }
    }

    pub fn tick(&mut self) -> Option<Transition> {
        let previous_state = self.state;
        self.state = self.next_state();
        match (previous_state, self.state) {
            (p, n) if p == n => None,
            (_, State::Idle) => {
                let character = self.current;
                self.current = morse::MorseCode::empty();
                Some(if character == morse::TRANSMIT {
                    Transition::Transmit
                } else {
                    Transition::Character(character.lookup())
                })
            }
            (_, State::Press(PressType::VeryLong)) => Some(Transition::VeryLong),
            (_, State::Press(PressType::Long)) => Some(Transition::Long),
            _ => None,
        }
    }
}

pub struct MorseTimingMachine {
    long_press: MicroSeconds,
    very_long_press: MicroSeconds,
    timeout: MicroSeconds,
    machine: MorseTimelessMachine,
}

impl MorseTimingMachine {
    pub const fn new(dot_length: MicroSeconds) -> Self {
        Self {
            long_press: dot_length,
            very_long_press: MicroSeconds(2 * dot_length.0),
            timeout: MicroSeconds(3 * dot_length.0),
            machine: MorseTimelessMachine::new(),
        }
    }

    pub fn press(&mut self, timer: &mut Timer<TIM2>) {
        self.machine.press();
        timer.clear_irq();
        timer.start(self.long_press);
        timer.listen();
    }

    pub fn release(&mut self, timer: &mut Timer<TIM2>) {
        self.machine.release();
        timer.clear_irq();
        let timeout = if self.machine.current == morse::TRANSMIT {
            1.ms()
        } else {
            self.timeout
        };
        timer.start(timeout);
        timer.listen();
    }

    pub fn tick(&mut self, timer: &mut Timer<TIM2>) -> Option<Transition> {
        timer.clear_irq();
        let transition = self.machine.tick();
        if let Some(ref state_change) = transition {
            match state_change {
                Transition::Long => timer.start(self.very_long_press),
                Transition::VeryLong | Transition::Transmit | Transition::Character(_) => {
                    timer.unlisten()
                }
            }
        }
        transition
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_morse_dot() {
        let mut machine = MorseTimelessMachine::new();
        assert_eq!(machine.tick(), None);
        assert_eq!(
            machine.press().release().tick(),
            Some(Transition::Character('e' as u8))
        );
    }
    #[test]
    fn test_morse_dash() {
        let mut machine = MorseTimelessMachine::new();
        machine.press().tick();
        assert_eq!(
            machine.release().tick(),
            Some(Transition::Character('t' as u8))
        );
    }
    #[test]
    fn test_morse_multiple() {
        let mut machine = MorseTimelessMachine::new();
        machine.press().release().press().tick();
        assert_eq!(
            machine.release().tick(),
            Some(Transition::Character('a' as u8))
        );
    }
    #[test]
    fn test_morse_transmit() {
        let mut machine = MorseTimelessMachine::new();
        assert_eq!(machine.press().tick(), Some(Transition::Long));
        assert_eq!(machine.tick(), Some(Transition::VeryLong));
        assert_eq!(machine.release().tick(), Some(Transition::Transmit));
    }
    #[test]
    fn test_morse_no_transmit() {
        let mut machine = MorseTimelessMachine::new();
        machine.press().release();
        assert_eq!(machine.press().tick(), Some(Transition::Long));
        assert_eq!(machine.tick(), None);
        assert_eq!(machine.tick(), None);
        assert_eq!(
            machine.release().tick(),
            Some(Transition::Character('a' as u8))
        );
    }
}
