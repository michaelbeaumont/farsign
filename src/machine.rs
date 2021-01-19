pub struct MorseMachine {}

pub enum Transition {}

impl MorseMachine {
    fn new() -> Self;
    fn press(&mut self);
    fn release(&mut self);
    fn tick(&mut self) -> Option<Transition>;
}
