#![no_main]
#![no_std]

use panic_semihosting as _;
use rtic::app;

use stm32l0xx_hal as hal;

mod epaper;
mod machine;
mod morse;
mod radio;
mod status;

#[app(device = stm32l0::stm32l0x2, peripherals = true, dispatchers = [USART1])]
mod app {
    use crate::{epaper, machine, status, radio};
    use farsign::*;

    type Message = Vec<u8, consts::U32>;
    use embedded_time::duration::*;
    use heapless::{consts, Vec};

    use crate::hal::{
        delay,
        exti::{Exti, ExtiLine, GpioLine, TriggerEdge},
        gpio::*,
        pac::TIM2,
        prelude::*,
        syscfg,
        timer::Timer,
    };

    type PBOut = Pin<Output<PushPull>>;

    const DOT_LENGTH: Microseconds = Microseconds(200_000);

    #[shared]
    struct Resources {
        #[lock_free]
        status: status::StatusLights<PBOut, PBOut, PBOut>,
        #[lock_free]
        timer: Timer<TIM2>,
        #[lock_free]
        morse: machine::MorseTimingMachine,
        #[lock_free]
        out: Message,
        #[lock_free]
        radio: radio::Lora,
    }

    #[local]
    struct Local {
        button: gpiob::PB2<Input<PullUp>>,
    }

    #[init]
    fn init(
        init::Context { core, device, .. }: init::Context,
    ) -> (Resources, Local, init::Monotonics) {
        // Configure the clock at the default speed
        let mut rcc = device.RCC.freeze(crate::hal::rcc::Config::default());

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
        let mut delay = delay::Delay::new(core.SYST, rcc.clocks);

        let radio = radio::init_radio(
            device.SPI1,
            gpiob.pb3,
            gpioa.pa6,
            gpioa.pa7,
            gpioa.pa15,
            gpioc.pc0,
            &mut rcc,
            &mut delay,
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
            &mut delay,
        );
        epaper::display_startup(&mut spi, &mut delay, &mut epd);

        let button = gpiob.pb2.into_pull_up_input();
        let mut exti = Exti::new(device.EXTI);
        let mut syscfg = syscfg::SYSCFG::new(device.SYSCFG, &mut rcc);
        let line = GpioLine::from_raw_line(button.pin_number()).unwrap();
        exti.listen_gpio(&mut syscfg, button.port(), line, TriggerEdge::Both);
        let dio0 = gpioa.pa0.into_floating_input();
        let dio_line = GpioLine::from_raw_line(dio0.pin_number()).unwrap();
        exti.listen_gpio(&mut syscfg, dio0.port(), dio_line, TriggerEdge::Rising);

        let timer = Timer::new(device.TIM2, &mut rcc);
        (
            Resources {
                status,
                timer,
                morse: super::machine::MorseTimingMachine::new(DOT_LENGTH),
                out: Vec(heapless::i::Vec::new()),
                radio,
            },
            Local { button },
            init::Monotonics(),
        )
    }

    #[task(capacity = 10, shared = [radio])]
    fn event(
        event::Context {
            shared:
                event::SharedResources {
                    radio,
                    ..
                },
        }: event::Context,
        event: Event<consts::U32>,
    ) {
        match event {
            Event::Transmit(ch) => {
                radio.transmit_payload(&ch).unwrap();
            }
            Event::Receive => {
                // TODO
                // push 'static buf to display task
                //ctx.resources.radio.read_packet_into(&mut rcv).unwrap();
            }
        };
    }

    #[task(binds = TIM2, shared = [status, timer, morse, out])]
    fn timer(
        timer::Context {
            shared:
                timer::SharedResources {
                    status,
                    mut timer,
                    morse,
                    mut out,
                    ..
                },
        }: timer::Context,
    ) {
        if let Some(state_change) = morse.tick(&mut timer) {
            match state_change {
                crate::machine::Transition::Long => status.on_long(),
                crate::machine::Transition::VeryLong => status.busy(),
                crate::machine::Transition::Transmit => {
                    let transmission: Message = core::mem::replace(&mut out, Vec::new());
                    let e = farsign::Event::Transmit(transmission);
                    event::spawn(e).unwrap();
                }
                super::machine::Transition::Character(ch) => {
                    status.flash_busy(timer);
                    out.push(ch).ok().unwrap();
                }
            }
        } else {
            status.flash_tick(&mut timer);
        }
    }

    #[task(binds = EXTI2_3, local = [button], shared = [status, timer, morse])]
    fn button(
        button::Context {
            local: button::LocalResources { button },
            shared:
                button::SharedResources {
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

    #[task(binds = EXTI0_1)]
    fn radio_txrx_done(_: radio_txrx_done::Context) {
        let e = farsign::Event::Receive;
        event::spawn(e).unwrap();
        // TODO
        Exti::unpend(GpioLine::from_raw_line(0).unwrap());
    }
}
