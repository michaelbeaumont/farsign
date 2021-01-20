#[derive(PartialEq)]
pub struct MorseCode {}

pub const TRANSMIT: MorseCode;

impl MorseCode {
    pub fn empty() -> Self;
    pub fn is_empty(&self) -> bool;
    pub fn append_dot(&mut self) -> Self;
    pub fn append_dash(&mut self) -> Self;
}
