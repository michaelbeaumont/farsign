use crate::hal::{
    gpio::*,
    pac,
    prelude::*,
    rcc::Rcc,
    spi::{NoMiso, Spi},
};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    text::{Text, TextStyleBuilder},
    pixelcolor::BinaryColor::On as Black,
    prelude::*,
};
use embedded_hal::blocking::delay::*;
use epd_waveshare::{
    epd2in9bc::{Display2in9bc, Epd2in9bc},
    prelude::*,
    SPI_MODE,
};

pub type SPI = Spi<pac::SPI2, (gpiob::PB13<Analog>, NoMiso, gpiob::PB15<Analog>)>;

pub fn init<DELAY, BUSY, DC, RST>(
    spi2: pac::SPI2,
    sck: gpiob::PB13<Analog>,
    mosi: gpiob::PB15<Analog>,
    cs: gpiob::PB12<Analog>,
    busy: BUSY,
    dc: DC,
    rst: RST,
    rcc: &mut Rcc,
    delay: &mut DELAY,
) -> (
    SPI,
    Epd2in9bc<SPI, gpiob::PB12<Output<PushPull>>, BUSY, DC, RST, DELAY>,
)
where
    DELAY: DelayMs<u8>,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    // `epd-waveshare` conveniently exports the SPI mode for the waveshare devices
    let mut spi = Spi::spi2(
        spi2,
        (sck, NoMiso, mosi),
        SPI_MODE,
        rcc.clocks.sys_clk(),
        rcc,
    );
    let epd = Epd2in9bc::new(
        &mut spi,
        cs.into_push_pull_output(),
        busy,
        dc,
        rst,
        delay,
    )
    .unwrap();
    (spi, epd)
}

pub type Epd<DELAY> = Epd2in9bc<
    SPI,
    gpiob::PB12<Output<PushPull>>,
    gpioa::PA2<Input<Floating>>,
    gpioa::PA10<Output<PushPull>>,
    gpioa::PA8<Output<PushPull>>,
    DELAY,
>;

pub fn display_startup<DELAY: DelayMs<u8>>(spi: &mut SPI, delay: &mut DELAY, epd: &mut Epd<DELAY>) {
    let mut display = Display2in9bc::default();
    // the rotation is used when rendering our text
    // and shapes into a bitmap
    display.set_rotation(DisplayRotation::Rotate90);
    // send a uniform chromatic and achromatic frame
    epd.clear_frame(spi, delay).expect("clear frame failed");
    let font = MonoTextStyle::new(&FONT_6X10, Black);
    let style = TextStyleBuilder::new().build();
    let _ = Text::with_text_style("Farsign", Point::new(50, 35), font, style)
        .draw(&mut display);
    // render our display to a buffer and set it as
    // our chromatic frame
    epd.update_chromatic_frame(spi, display.buffer())
        .expect("send text failed");
    epd.display_frame(spi, delay).expect("display startup failed");
}
