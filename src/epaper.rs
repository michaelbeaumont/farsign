use crate::hal::{
    gpio::*,
    pac,
    prelude::*,
    rcc::Rcc,
    spi::{NoMiso, Spi, MODE_0},
};

pub type SPI = Spi<pac::SPI2, (gpiob::PB13<Analog>, NoMiso, gpiob::PB15<Analog>)>;

pub fn init(
    spi2: pac::SPI2,
    sck: gpiob::PB13<Analog>,
    mosi: gpiob::PB15<Analog>,
    cs: gpiob::PB12<Analog>,
    rcc: &mut Rcc,
) -> SPI {
    Spi::spi2(spi2, (sck, NoMiso, mosi), MODE_0, rcc.clocks.sys_clk(), rcc)
}
