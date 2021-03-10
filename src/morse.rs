#[rustfmt::skip]
const CODE: [u8; 32] = [
    0, 'e' as u8, 't' as u8,
    'i' as u8, 'a' as u8, 'n' as u8, 'm' as u8,
     's' as u8, 'u' as u8, 'r' as u8, 'w' as u8, 'd' as u8, 'k' as u8, 'g' as u8, 'o' as u8,
    'h' as u8, 'v' as u8, 'f' as u8, 0, 'l' as u8, 0, 'p' as u8, 'j' as u8,
    'b' as u8, 'x' as u8, 'c' as u8, 'y' as u8, 'z' as u8, 'q' as u8, 0, 0, '\0' as u8,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MorseCode(u8);

pub const TRANSMIT: MorseCode = MorseCode(31);

impl MorseCode {
    pub const fn empty() -> Self {
        MorseCode(0)
    }
    pub fn is_empty(&self) -> bool {
        return self.0 == 0;
    }
    fn advance_pointer(&self, is_dash: bool) -> Self {
        let next = (self.0 << 1) + 1 + (is_dash as u8);
        Self(if next > 31 { 0 } else { next })
    }
    pub fn append_dot(&mut self) -> Self {
        self.advance_pointer(false)
    }
    pub fn append_dash(&mut self) -> Self {
        self.advance_pointer(true)
    }
    pub fn lookup(&self) -> u8 {
        CODE[self.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const DASH_DASH_DASH_INDEX: MorseCode = MorseCode(14);
    #[test]
    fn test_advance_pointer() {
        assert_eq!(MorseCode(2), MorseCode(0).append_dash());
        assert_eq!(MorseCode(6), MorseCode(2).append_dash());
        assert_eq!(MorseCode(13), MorseCode(6).append_dot());
        assert_eq!(
            DASH_DASH_DASH_INDEX,
            MorseCode(0).append_dash().append_dash().append_dash(),
        );
    }

    #[test]
    fn test_lookup() {
        assert_eq!(
            's' as u8,
            MorseCode::empty()
                .append_dot()
                .append_dot()
                .append_dot()
                .lookup()
        );
        assert_eq!(
            'o' as u8,
            MorseCode::empty()
                .append_dash()
                .append_dash()
                .append_dash()
                .lookup()
        );
        assert_eq!(
            'l' as u8,
            MorseCode::empty()
                .append_dot()
                .append_dash()
                .append_dot()
                .append_dot()
                .lookup()
        );
        assert_eq!('t' as u8, MorseCode::empty().append_dash().lookup());
    }
}
