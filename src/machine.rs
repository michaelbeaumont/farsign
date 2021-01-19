#[derive(PartialEq, Eq)]
pub enum Transition {
    Long,
    VeryLong,
    Character(u8),
    Transmit,
}

pub struct MorseMachine {}

impl MorseMachine {
    fn new() -> Self;
    fn press(&mut self);
    fn release(&mut self);
    fn tick(&mut self) -> Option<Transition>;
}
