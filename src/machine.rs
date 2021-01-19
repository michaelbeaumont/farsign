pub struct MorseMachine {}

pub enum StateChange {}

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
