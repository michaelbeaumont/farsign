#![no_main]
#![no_std]

mod epaper;
mod status;

use crate::hal::{
    delay,
    exti::{Exti, ExtiLine, GpioLine, TriggerEdge},
    gpio::*,
    pac::{self, interrupt},
    prelude::*,
    syscfg,
};
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use panic_semihosting as _;
use stm32l0xx_hal as hal;

type PBOut = gpiob::PB<Output<PushPull>>;

static STATUS: Mutex<RefCell<Option<status::StatusLights<PBOut, PBOut, PBOut>>>> =
    Mutex::new(RefCell::new(None));
static BUTTON: Mutex<RefCell<Option<gpiob::PB2<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // Get one-time access to our peripherals
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Configure the clock at the default speed
    let mut rcc = dp.RCC.freeze(hal::rcc::Config::default());

    // Get access to the GPIO A & B ports
    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);

    // Setup and turn on green LED
    let mut green_pin = gpiob.pb5.into_push_pull_output();
    let mut blue_pin = gpiob.pb6.into_push_pull_output();
    let mut red_pin = gpiob.pb7.into_push_pull_output();
    green_pin.set_high().unwrap();
    blue_pin.set_high().unwrap();
    red_pin.set_high().unwrap();
    let status = status::StatusLights::new(
        red_pin.downgrade(),
        green_pin.downgrade(),
        blue_pin.downgrade(),
    );

    // Because SysTick is universal to Cortex-M chips it's provided by the `cortex_m` crate
    let syst_delay = delay::Delay::new(cp.SYST, rcc.clocks);
    let (mut spi, mut epd) = epaper::init(
        dp.SPI2,
        gpiob.pb13,
        gpiob.pb15,
        gpiob.pb12,
        gpioa.pa2.into_floating_input(),
        gpioa.pa10.into_push_pull_output(),
        gpioa.pa8.into_push_pull_output(),
        &mut rcc,
        syst_delay,
    );
    epaper::display_startup(&mut spi, &mut epd);

    let button = gpiob.pb2.into_pull_up_input();
    let mut exti = Exti::new(dp.EXTI);
    let mut syscfg = syscfg::SYSCFG::new(dp.SYSCFG, &mut rcc);
    let line = GpioLine::from_raw_line(button.pin_number()).unwrap();
    exti.listen_gpio(&mut syscfg, button.port(), line, TriggerEdge::Both);

    cortex_m::interrupt::free(|cs| {
        *STATUS.borrow(cs).borrow_mut() = Some(status);
        *BUTTON.borrow(cs).borrow_mut() = Some(button);
    });

    unsafe {
        NVIC::unmask(line.interrupt());
    }

    loop {}
}

#[allow(non_snake_case)]
#[interrupt]
fn EXTI2_3() {
    cortex_m::interrupt::free(|cs| {
        let (pin_number, is_low) = BUTTON
            .borrow(cs)
            .borrow()
            .as_ref()
            .map(|p| (p.pin_number(), p.is_low().unwrap()))
            .unwrap();
        Exti::unpend(GpioLine::from_raw_line(pin_number).unwrap());
        let mut status = STATUS.borrow(cs).borrow_mut();
        if is_low {
            status.as_mut().unwrap().off();
        } else {
            status.as_mut().unwrap().on_short();
        }
    })
}
