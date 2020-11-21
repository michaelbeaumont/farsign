pub struct StatusLights<R, G, B> {
    red: R,
    green: G,
    blue: B,
}

impl<R, G, B> StatusLights<R, G, B> {
    pub fn new(red: R, green: G, blue: B) -> Self {
        StatusLights { red, green, blue }
    }
}
