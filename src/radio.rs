use embedded_hal::blocking::delay;
use sx127x_lora as sx;

use crate::hal::{gpio::*, pac::SPI1, prelude::*, rcc, spi};

type PB3 = gpiob::PB3<Analog>;
type PA6 = gpioa::PA6<Analog>;
type PA7 = gpioa::PA7<Analog>;
type SX1276SPIPins = (PB3, PA6, PA7);

pub type Lora = sx::LoRa<
    spi::Spi<SPI1, SX1276SPIPins>,
    gpioa::PA15<Output<PushPull>>,
    gpioc::PC0<Output<PushPull>>,
>;

pub fn init_radio(
    spi1: SPI1,
    pb3: PB3,
    pa6: PA6,
    pa7: PA7,
    pa15: gpioa::PA15<Analog>,
    pc0: gpioc::PC0<Analog>,
    mut rcc: &mut rcc::Rcc,
    delay: &mut dyn delay::DelayMs<u8>,
) -> Lora {
    let (sck, miso, mosi, cs, reset) = (
        pb3,
        pa6,
        pa7,
        pa15.into_push_pull_output(),
        pc0.into_push_pull_output(),
    );
    let spi = spi::Spi::spi1(spi1, (sck, miso, mosi), spi::MODE_0, 1_000_u32.Hz(), &mut rcc);
    let mut lora = sx::LoRa::new(spi, cs, reset, 868, delay).unwrap();
    lora.set_mode(sx::RadioMode::RxContinuous).unwrap();
    lora
}
