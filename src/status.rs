use crate::hal::prelude::*;
use crate::hal::{pac::TIM2, timer::Timer};
use core::convert::Infallible;

pub struct StatusLights<R, G, B> {
    red: R,
    green: G,
    blue: B,
    flasher: Flasher,
}

impl<R: OutputPin<Error = Infallible>, G: OutputPin<Error = Infallible>, B: OutputPin<Error = Infallible>>
    StatusLights<R, G, B>
{
    pub fn new(red: R, green: G, blue: B) -> Self {
        StatusLights {
            red,
            green,
            blue,
            flasher: Flasher::new(),
        }
    }

    pub fn busy(&mut self) {
        self.red.set_high().unwrap();
        self.green.set_low().unwrap();
        self.blue.set_low().unwrap();
    }

    pub fn on_short(&mut self) {
        self.red.set_low().unwrap();
        self.green.set_high().unwrap();
        self.blue.set_low().unwrap();
    }

    pub fn on_long(&mut self) {
        self.red.set_low().unwrap();
        self.green.set_low().unwrap();
        self.blue.set_high().unwrap();
    }

    pub fn off(&mut self) {
        self.red.set_low().unwrap();
        self.green.set_low().unwrap();
        self.blue.set_low().unwrap();
    }

    pub fn flash_busy(&mut self, timer: &mut Timer<TIM2>) {
        self.busy();
        self.flasher.flash(timer);
    }

    pub fn flash_tick(&mut self, timer: &mut Timer<TIM2>) {
        if self.flasher.tick(timer) {
            self.off();
        }
    }
}

struct Flasher {
    flashing: bool,
}

impl Flasher {
    const fn new() -> Self {
        Flasher { flashing: false }
    }

    fn flash(&mut self, timer: &mut Timer<TIM2>) {
        self.flashing = true;
        timer.start(100_u32.Hz());
        timer.listen();
    }

    fn tick(&mut self, timer: &mut Timer<TIM2>) -> bool {
        let turn_off = self.flashing;
        if self.flashing {
            timer.unlisten();
            self.flashing = false;
        }
        turn_off
    }
}
