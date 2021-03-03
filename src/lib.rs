#![cfg_attr(not(test), no_std)]
use heapless::{consts, spsc, ArrayLength, Vec};
use stm32l0xx_hal as hal;

pub mod epaper;
pub mod machine;
pub mod morse;
pub mod radio;
pub mod status;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event<N: ArrayLength<u8>> {
    Transmit(Vec<u8, N>),
    Receive,
}

pub type EventQueue = spsc::Queue<Event<consts::U32>, heapless::consts::U6>;
pub type EventConsumer = spsc::Consumer<'static, Event<consts::U32>, heapless::consts::U6>;
pub type EventProducer = spsc::Producer<'static, Event<consts::U32>, heapless::consts::U6>;
