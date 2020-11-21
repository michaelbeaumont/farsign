#![no_main]
#![no_std]

mod status;

use crate::hal::{gpio::*, pac, prelude::*};
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use panic_semihosting as _;
use stm32l0xx_hal as hal;

static STATUS: Mutex<RefCell<Option<status::StatusLights<?, ?, ?>>>> =
    Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // Get one-time access to our peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure the clock at the default speed
    let mut rcc = dp.RCC.freeze(hal::rcc::Config::default());

    // Get access to the GPIO B port
    let gpiob = dp.GPIOB.split(&mut rcc);

    // Setup and turn on green LED
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
fn INTERRUPT() {
    // handle button press
}
