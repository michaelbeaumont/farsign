#![no_main]
#![no_std]

use crate::hal::{gpio::*, pac, prelude::*};
use cortex_m_rt::entry;
use panic_semihosting as _;
use stm32l0xx_hal as hal;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Configure the clock, interrupts.
    let mut rcc = dp.RCC.freeze(hal::rcc::Config::hsi16());

    // Acquire peripherals
    let gpiob = dp.GPIOB.split(&mut rcc);

    // Setup long/short LEDs
    let mut green_pin = gpiob.pb5.into_push_pull_output();
    green_pin.set_high().unwrap();
    loop {}
}
