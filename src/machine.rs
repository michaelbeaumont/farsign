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

    fn state(&self) -> State {
        if !self.pressed {
            if self.count > self.timeout {
                State::Idle
            } else {
                State::WaitingOnPress
            }
        } else {
            State::Press(if self.count > self.very_long_press {
                PressType::VeryLong
            } else if self.count > self.long_press {
                PressType::Long
            } else {
                PressType::Short
            })
        }
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
}

impl MorseMachine {
    pub fn new(dot_ticks: u32) -> Self {
        Self {
            button: Button::new(dot_ticks, 3 * dot_ticks, 3 * dot_ticks),
        }
    }

    pub fn press(&mut self) {
        self.button.press();
    }

    pub fn release(&mut self) {
        self.button.release();
    }

    pub fn tick(&mut self) -> Option<Transition> {
        self.button.tick();
        None
    }
}
