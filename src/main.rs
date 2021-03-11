#![no_main]
#![no_std]

mod epaper;
mod machine;
mod morse;
mod status;

use panic_semihosting as _;
use rtic::app;
use stm32l0xx_hal as hal;

use crate::hal::{
    delay,
    exti::{Exti, ExtiLine, GpioLine, TriggerEdge},
    gpio::*,
    pac::TIM2,
    prelude::*,
    syscfg, time,
    timer::Timer,
};

type PBOut = gpiob::PB<Output<PushPull>>;

const DOT_LENGTH: time::MicroSeconds = time::MicroSeconds(200_000);

#[app(device = stm32l0::stm32l0x2, peripherals = true)]
const APP: () = {
    struct Resources {
        button: gpiob::PB2<Input<PullUp>>,
        status: status::StatusLights<PBOut, PBOut, PBOut>,
        timer: Timer<TIM2>,
        #[init(machine::MorseTimingMachine::new(DOT_LENGTH))]
        morse: machine::MorseTimingMachine,
    }

    #[init]
    fn init(init::Context { core, device }: init::Context) -> init::LateResources {
        // Configure the clock at the default speed
        let mut rcc = device.RCC.freeze(hal::rcc::Config::default());

        // Get access to the GPIO A & B ports
        let gpioa = device.GPIOA.split(&mut rcc);
        let gpiob = device.GPIOB.split(&mut rcc);

        // Setup
        let green_pin = gpiob.pb5.into_push_pull_output();
        let blue_pin = gpiob.pb6.into_push_pull_output();
        let red_pin = gpiob.pb7.into_push_pull_output();
        let status = status::StatusLights::new(
            red_pin.downgrade(),
            green_pin.downgrade(),
            blue_pin.downgrade(),
        );

        let delay = delay::Delay::new(core.SYST, rcc.clocks);
        let (mut spi, mut epd) = epaper::init(
            device.SPI2,
            gpiob.pb13,
            gpiob.pb15,
            gpiob.pb12,
            gpioa.pa2.into_floating_input(),
            gpioa.pa10.into_push_pull_output(),
            gpioa.pa8.into_push_pull_output(),
            &mut rcc,
            delay,
        );
        epaper::display_startup(&mut spi, &mut epd);

        let button = gpiob.pb2.into_pull_up_input();
        let mut exti = Exti::new(device.EXTI);
        let mut syscfg = syscfg::SYSCFG::new(device.SYSCFG, &mut rcc);
        let line = GpioLine::from_raw_line(button.pin_number()).unwrap();
        exti.listen_gpio(&mut syscfg, button.port(), line, TriggerEdge::Both);

        let timer = Timer::new(device.TIM2, &mut rcc);
        init::LateResources {
            button,
            status,
            timer,
        }
    }

    #[task(binds = TIM2, resources = [status, timer, morse])]
    fn timer(
        timer::Context {
            resources:
                timer::Resources {
                    status,
                    timer,
                    morse,
                    ..
                },
        }: timer::Context,
    ) {
        static mut TIMEOUT_FLASH: Option<bool> = None;
        if let Some(state_change) = morse.tick(timer) {
            match state_change {
                machine::Transition::Long => status.on_long(),
                machine::Transition::VeryLong => status.busy(),
                machine::Transition::Transmit => {
                    // send letters
                }
                machine::Transition::Character(ch) => {
                    status.busy();
                    *TIMEOUT_FLASH = Some(true);
                    timer.start(20.ms());
                    timer.listen();
                    hprintln!("{}", ch as char).unwrap();
                    // handle letter
                }
            }
        } else if let Some(flashing) = *TIMEOUT_FLASH {
            if flashing {
                *TIMEOUT_FLASH = Some(false);
            } else {
                status.off();
                *TIMEOUT_FLASH = None;
                timer.unlisten();
            }
        }
    }

    #[task(binds = EXTI2_3, resources = [button, status, timer, morse])]
    fn button(
        button::Context {
            resources:
                button::Resources {
                    button,
                    status,
                    timer,
                    morse,
                    ..
                },
        }: button::Context,
    ) {
        Exti::unpend(GpioLine::from_raw_line(button.pin_number()).unwrap());
        if button.is_low().unwrap() {
            morse.press(timer);
            status.on_short();
        } else {
            morse.release(timer);
            status.off();
        }
    }
};
