#![no_main]
#![no_std]

use hal::prelude::*;
use heapless::{consts, Vec};
use panic_semihosting as _;
use rtic::app;
use stm32l0xx_hal as hal;
use embedded_time::duration::*;

use crate::hal::{delay, exti::*, gpio::*, pac::TIM2, syscfg, timer::Timer};

use farsign::*;

type PBOut = Pin<Output<PushPull>>;
type Message = Vec<u8, consts::U32>;

const DOT_LENGTH: Microseconds = Microseconds(200_000);

#[app(device = stm32l0::stm32l0x2, peripherals = true)]
const APP: () = {
    struct Resources {
        button: gpiob::PB2<Input<PullUp>>,
        status: status::StatusLights<PBOut, PBOut, PBOut>,
        timer: Timer<TIM2>,
        #[init(machine::MorseTimingMachine::new(DOT_LENGTH))]
        morse: machine::MorseTimingMachine,
        #[init(Vec(heapless::i::Vec::new()))]
        out: Message,
        radio: radio::Lora,
    }

    #[init]
    fn init(init::Context { core, device }: init::Context) -> init::LateResources {
        // Configure the clock at the default speed
        let mut rcc = device.RCC.freeze(hal::rcc::Config::default());
        // Get access to the GPIO ports
        let gpioa = device.GPIOA.split(&mut rcc);
        let gpiob = device.GPIOB.split(&mut rcc);
        let gpioc = device.GPIOC.split(&mut rcc);

        // Setup
        let green_pin = gpiob.pb5.into_push_pull_output();
        let blue_pin = gpiob.pb6.into_push_pull_output();
        let red_pin = gpiob.pb7.into_push_pull_output();
        let status = status::StatusLights::new(
            red_pin.downgrade(),
            green_pin.downgrade(),
            blue_pin.downgrade(),
        );
        // Because SysTick is universal to Cortex-M chips it's provided by the `cortex_m` crate
        let mut syst_delay = delay::Delay::new(core.SYST, rcc.clocks);

        let radio = radio::init_radio(
            device.SPI1,
            gpiob.pb3,
            gpioa.pa6,
            gpioa.pa7,
            gpioa.pa15,
            gpioc.pc0,
            &mut rcc,
            &mut syst_delay,
        );

        let (mut spi, mut epd) = epaper::init(
            device.SPI2,
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
        let mut exti = Exti::new(device.EXTI);
        let mut syscfg = syscfg::SYSCFG::new(device.SYSCFG, &mut rcc);
        let line = GpioLine::from_raw_line(button.pin_number()).unwrap();
        exti.listen_gpio(&mut syscfg, button.port(), line, TriggerEdge::Both);
        let dio0 = gpioa.pa0.into_floating_input();
        let dio_line = GpioLine::from_raw_line(dio0.pin_number()).unwrap();
        exti.listen_gpio(&mut syscfg, dio0.port(), dio_line, TriggerEdge::Rising);

        let timer = Timer::new(device.TIM2, &mut rcc);

        init::LateResources {
            button,
            status,
            timer,
            radio,
        }
    }

    #[task(capacity = 10, resources = [radio])]
    fn event(ctx: event::Context, event: Event<consts::U32>) {
        match event {
            Event::Transmit(ch) => {
                ctx.resources.radio.transmit_payload(&ch).unwrap();
            }
            Event::Receive => {
                // TODO
                // push 'static buf to display task
                //ctx.resources.radio.read_packet_into(&mut rcv).unwrap();
            }
        };
    }

    #[task(binds = TIM2, spawn = [event], resources = [status, timer, morse, out])]
    fn timer(
        timer::Context {
            spawn,
            resources:
                timer::Resources {
                    status,
                    timer,
                    morse,
                    mut out,
                    ..
                },
        }: timer::Context,
    ) {
        if let Some(state_change) = morse.tick(timer) {
            match state_change {
                machine::Transition::Long => status.on_long(),
                machine::Transition::VeryLong => status.busy(),
                machine::Transition::Transmit => {
                    let transmission: Message = core::mem::replace(&mut out, Vec::new());
                    let e = farsign::Event::Transmit(transmission);
                    spawn.event(e).unwrap();
                }
                machine::Transition::Character(ch) => {
                    status.flash_busy(timer);
                    out.push(ch).ok().unwrap();
                }
            }
        } else {
            status.flash_tick(timer);
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

    #[task(binds = EXTI0_1, spawn = [event])]
    fn radio_txrx_done(radio_txrx_done::Context { spawn }: radio_txrx_done::Context) {
        let e = farsign::Event::Receive;
        spawn.event(e).unwrap();
        // TODO
        Exti::unpend(GpioLine::from_raw_line(0).unwrap());
    }

    extern "C" {
        fn USART1();
    }
};
