use crate::morse;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PressType {
    Short,
    Long,
    VeryLong,
}

#[derive(PartialEq, Eq)]
enum State {
    Press(PressType),
    WaitingOnPress,
    Idle,
}

struct Button {
    long_press: u32,
    very_long_press: u32,
    timeout: u32,
    count: u32,
    pressed: bool,
}

impl Button {
    fn new(long_press: u32, very_long_press: u32, timeout: u32) -> Self {
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

#[derive(PartialEq, Eq)]
pub enum Transition {
    Long,
    VeryLong,
    Character(u8),
    Transmit,
}

pub struct MorseMachine {
    button: Button,
    current: morse::MorseCode,
}

impl MorseMachine {
    pub fn new(dot_ticks: u32) -> Self {
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
