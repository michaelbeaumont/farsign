pub struct StatusLights<R, G, B> {
    red: R,
    green: G,
    blue: B,
}

impl<R, G, B> StatusLights<R, G, B> {
    pub fn new(red: R, green: G, blue: B) -> Self {
        StatusLights { red, green, blue }
    }

    pub fn on_short(&mut self) {
        self.red.set_low().unwrap();
        self.green.set_low().unwrap();
        self.blue.set_high().unwrap();
    }
}
