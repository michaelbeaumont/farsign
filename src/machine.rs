#[derive(PartialEq, Eq)]
pub enum StateChange {
    LongPress,
    Transmit,
    NewLetter(u8),
}

pub struct MorseMachine {}

impl MorseMachine {
    pub fn new() -> Self {
        MorseMachine {}
    }

    pub fn press(&mut self) {}

    pub fn release(&mut self) {}

    pub fn tick(&mut self) -> Option<StateChange> {
        None
    }
}
