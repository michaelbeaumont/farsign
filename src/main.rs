#![no_main]
#![no_std]

mod status;

use crate::hal::{
    gpio::*,
    pac::{self, interrupt},
    prelude::*,
};
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
    let mut blue_pin = gpiob.pb6.into_push_pull_output();
    let mut red_pin = gpiob.pb7.into_push_pull_output();
    green_pin.set_high().unwrap();
    blue_pin.set_high().unwrap();
    red_pin.set_high().unwrap();
    let status = status::StatusLights::new(red_pin, green_pin, blue_pin);
    loop {}
}

#[allow(non_snake_case)]
#[interrupt]
fn EXTI2_3() {
    // disable LEDs
}
