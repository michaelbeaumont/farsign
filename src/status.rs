use crate::hal::prelude::*;
use void::Void;

pub struct StatusLights<R, G, B> {
    red: R,
    green: G,
    blue: B,
}

impl<R: OutputPin<Error = Void>, G: OutputPin<Error = Void>, B: OutputPin<Error = Void>>
    StatusLights<R, G, B>
{
    pub fn new(red: R, green: G, blue: B) -> Self {
        StatusLights { red, green, blue }
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
}
