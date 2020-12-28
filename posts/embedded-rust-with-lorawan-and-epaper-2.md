---
title: Embedded Rust with LoRa and ePaper - Display
date: "Dec. 12, 2020"
repo: "https://github.com/michaelbeaumont/farsign"
series: 2
issue: 2
...

[previous]: embedded-rust-with-lorawan-and-epaper-1
[discovery-board]: https://www.st.com/en/evaluation-tools/b-l072z-lrwan1.html "Discovery board"
[repo]: https://github.com/michaelbeaumont/farsign
[discovery-manual]: https://www.st.com/resource/en/user_manual/dm00329995-discovery-kit-for-lorawan-sigfox-and-lpwan-protocols-with-stm32l0-stmicroelectronics.pdf "discovery board manual"
[uc-datasheet]: https://www.st.com/resource/en/datasheet/stm32l072v8.pdf "Microcontroller datasheet"
[uc-reference]: https://www.st.com/resource/en/reference_manual/dm00108281-ultralowpower-stm32l0x2-advanced-armbased-32bit-mcus-stmicroelectronics.pdf "Microcontroller reference manual"
[embedded-book]: https://rust-embedded.github.io/book
[epaper-page]: https://www.waveshare.com/product/displays/e-paper/epaper-2/2.9inch-e-paper-module-b.htm
[epaper-wiki]: https://www.waveshare.com/wiki/2.9inch_e-Paper_Module_(B)
[epaper-datasheet]: https://www.waveshare.com/wiki/File:2.9inch-e-paper-b-specification.pdf
[epd-waveshare]: https://github.com/caemor/epd-waveshare
[module-datasheet]: https://wireless.murata.com/pub/RFM/data/type_abz.pdf

In the [previous post][previous], we got our discovery board lighting up on button presses using GPIO interrupts.
Next I want to introduce our first peripheral device, our ePaper display.

I chose the [2.9 inch three color ePaper module from Waveshare][epaper-page] for this project,
mostly because a color ePaper display is pretty cool and I wanted to try it out.
Be aware, they are very slow.

In order to communiate with the display, we'll use
[**SPI**](https://en.wikipedia.org/wiki/Serial_Peripheral_Interface), a _synchronous_ (synchronized to a clock signal),
_serial_ (one bit at a time) communication protocol running on four wires.
SPI is extremely common protocol on embedded systems for communicating with
peripherals. The radio transceiver on our discovery board is connected to our
microcontroller via SPI as well.

SPI designates two lines, one for each device, for transmitting data (`MOSI` and
`MISO`). It uses one line for clock synchronization (`SCK`) and one for
notifying the device we're going to send data (`CS`) to it.
Because we have a _display_ with one way communication, we
only use one of the data lines (`MOSI`) and simply set `CS` to active when we
want to transmit data.

Waveshare provides [a wiki page][epaper-wiki] for each display that
gives a summary of the communication protocol specific to that display.
In addition to the SPI lines, the display uses three more pins:

- `RST` - for resetting the device
- `BUSY` (our only input pin) - indicates we need to wait because the device is
  busy
- `DC` - indicates to the display whether we're sending data or a command

![EPD pins](media/epd_pins.jpg){id=epd_pins}

## Initialization

Let's look at how we connect and initialize our device. The first thing to note is that
we won't be directly manipulating the SPI data lines ourselves.
[Our MCU datasheet][uc-datasheet] tells us that it provides 2 SPI peripherals that
handle the SPI communication for us.
The gritty details are again in our [reference manual][uc-reference], chapter 30.

Lucky for us, `embedded-hal` and the HAL for our board are going to handle those details for us by
[initializating the peripherals (setting the clock, etc.)](https://docs.rs/stm32l0xx-hal/0.6.2/src/stm32l0xx_hal/spi.rs.html#177)
and giving us [a convenient interface for sending and receiving data](https://docs.rs/embedded-hal/0.2.4/embedded_hal/spi/trait.FullDuplex.html).

Let's work on getting our SPI peripheral initialized.
_Spoiler alert_ for later: [the datasheet for our module, page 4][module-datasheet]
tells us that the radio transceiver is connected to `SPI1`.
And so we'll be using `SPI2` for our display!

### SPI pins

The SPI peripheral in our microcontroller is only capable of using specific pins
for the SPI lines so let's get those sorted.

If we look at our [board manual][discovery-manual], we find that chapter
9.2/table 7 tells us which pins are exposed on our board and which function they
have. As part of the `CN3` connector we find `SPI2_SCK: PB13`, `SPI2_MOSI: PB15`,
and `SPI2_NSS: PB12` (_NSS_ is another name for the `CS` line).

A more complete description of the pins and their functions can be found in
[table 16 of our MCU datasheet][uc-datasheet].

### HAL

The HAL crate and clever types also gives us some hints as to which pins we need to provide.
If we look at [the signature of the `Spi::spi2`
function](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/spi/struct.Spi.html#method.spi2)
we see we need to pass an argument `pins: PINS` with the bound `PINS: Pins<SPI2>`. The [docs
for the `Pins<SPI>` `Trait`](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/spi/trait.Pins.html#implementors)
link to the `PinSck`, `PinMiso`, and `PinMosi` traits.
[`PinMosi<SPI2>`](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/spi/trait.PinMosi.html) for
example, is only implemented by `PB15`.
You may also have noticed the `NoMiso` struct, which implements `PinMiso<SPI2>`.
We'll pass that instead of passing in a real pin and leaving it unused.

The final arguments of our
[`Spi::spi2` function](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/spi/struct.Spi.html#method.spi2)
are:

- the raw peripheral `SPI2`, which comes from our `Peripherals` struct ([`dp` from part 1][dp-part-1])
- the speed of the SPI peripheral, which we set equal to the system clock speed ([in `main` we set it to the default][clock-part-1])
- the `mode: spi::Mode`

[dp-part-1]: embedded-rust-with-lorawan-and-epaper-1.html#cb1-11
[clock-part-1]: embedded-rust-with-lorawan-and-epaper-1.html#cb1-14

#### SPI Mode

SPI requires setting the [**clock polarity** and **phase**](https://en.wikipedia.org/wiki/Serial_Peripheral_Interface#Clock_polarity_and_phase),
which determine timing and polarity of the clock pulses with respect to the data signal.
Reading through [the display wiki][epaper-wiki] or
by [interpreting the diagrams in the datasheet, chapter _MCU Serial Interface (4-wire SPI)_][epaper-datasheet],
we see that we need
[SPI mode 0](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/spi/constant.MODE_0.html).

Finally we have something like:

```rust
let spi = Spi::spi2(
    dp.SPI2,
    (gpiob.pb12, NoMiso, gpio.pb15),
    MODE_0,
    rcc.clocks.sys_clk(),
    &mut rcc
);
```

### Additional pins

We've also got to connect `BUSY`, `DC` and `RST`. We can arbitrarily choose
any GPIO pins for this. I chose `PA2`, `PA10` and `PA8`.

![EPD pin connections (compare colors with [pins](#epd_pins))](media/epd_mcu_pins.jpg){width=25%}

## Display driver crate

We now have our SPI peripheral connected and setup and can start up the display and send data.
How exactly this is done is specified, albeit sometimes vaguely and confusingly, [in the
datasheet][epaper-datasheet]. It lists the commands we can send as well
as how to send the pixel data.

We'll spare ourselves translating those commands into Rust and use the
amazing [epd-waveshare][] library. Let's start a new module for handling our display:

```{uri="src/epaper.rs" ref=v2_spi_init .rust}

```

Most waveshare ePaper displays are supported by `epd-waveshare` and
in the docs we find [`EPD2in9bc::new`](https://docs.rs/epd-waveshare/0.4.0/epd_waveshare/epd2in9bc/struct.EPD2in9bc.html#method.new)
for our device (the protocol for the red and yellow versions of our display is
identical).

We'll need our initialized SPI struct as well as `CS`, which we haven't used
yet. The `SPI` struct doesn't own the `CS` pin like it does the other SPI pins.
Retaining ownership of the pin gives us more flexibility, especially because if
we have multiple SPI devices, we need one `CS` pin per device.
We also need the 3 other Waveshare specific pins, `RST`, `BUSY` and `DC`.

The final piece of the puzzle here is `delay: &mut DELAY where DELAY: DelayMs<u8>`,
which we're going to require as an argument of our `init` function and cover in
just a second:

```{uri="src/epaper.rs" diff=v2_spi_init ref=v2_epd_init .rust}

```

## Calling `init`

Let's call our new `init` function from `main`.

```{uri="src/main.rs" diff=v2_epd_init ref=v2_epd_init_call_no_delay a=35 b=38 .rust}

```

```{uri="src/main.rs" diff=v2_epd_init ref=v2_epd_init_call_no_delay a=53 b=63 .rust}

```

### Delay

Sometimes with embedded software we need to wait. In
our case `epd-waveshare` waits some number of milliseconds after
powering on our device.

One way to do this in general is to have the CPU loop:

```rust
for i in 1..10_000 {
    cortex_m::asm::noop(); // prevent the compiler from optimizing our loop away
}
```

The disadvantage of this strategy is that it's very difficult to wait a specific
number of milliseconds. For example, the wall execution time of one `noop` instruction varies
based on the configured clock speed and interrupts can preempt our loop.

[`DelayMs<T>`](https://docs.rs/embedded-hal/0.2.4/embedded_hal/blocking/delay/trait.DelayMs.html)
is an `embedded-hal` trait that provides a `delay_ms` function
for waiting some number of milliseconds. Most `embedded-hal`-implementing HAL crates provide
at least one `Delay`-implementing struct, typically on top of _timers_.

### Timers

Timers are configurable counters.
In general, a timer is configured by selecting some input signal as a clock and
then selecting a _prescaler_, which can scale the frequency down by some factor.
We can then start our timer, which updates a register with the elapsed clock
time.

Our timers, as well as all peripherals on our microcontroller, are connected to
what's called a _bus_. A _bus_ is an electrical interconnect that allows
components to communicate amongst each other.
[The microcontroller datasheet][uc-datasheet] shows us in _Figure 1_ the
specifics of which peripherals are connected to which buses.
Every timer on our STM chip is connected to the bus' _internal clock_.
The general purpose timers can also use external signals as clocks.

The our SPI peripheral we set up earlier is connected to a bus, too.
[The implementation of `spi2`](https://docs.rs/stm32l0xx-hal/0.6.2/src/stm32l0xx_hal/spi.rs.html#197-209)
sets the clock frequency by finding the relationship between [the given frequency](#cb1) and the clock of its bus
(`APB1`, a peripheral bus) in order to set the prescaler.

[Figure 2 of the MCU datasheet][uc-datasheet] shows the relationships between
all the clocks, prescalers and buses in the _clock tree_.

`stm32l0xx_hal` provides the `Delay` struct, which implements `DelayMs` and `DelayUs`
with the `SysTick` peripheral. `SysTick` is provided by every Cortex-based microcontroller.
Our MCU provides additional, ST-specific timers as well (`TIM2`, `TIM3`).

```{uri="src/main.rs" diff=v2_epd_init_call_no_delay ref=v2_epd_init_call_delay a=7 b=8 .rust}

```

```{uri="src/main.rs" diff=v2_epd_init_call_no_delay ref=v2_epd_init_call_delay a=30 b=32 .rust}

```

```{uri="src/main.rs" diff=v2_epd_init_call_no_delay ref=v2_epd_init_call_delay a=54 b=66 .rust}

```

## Drawing

`epd-waveshare` uses `embedded-graphics`, which provides primitives like
shapes and text. Let's explore these libraries by displaying a splash on device startup.
The general idea is that we maintain an abstract display to which we draw our
abstract objects and then render that display to a bitmap, either for black pixels
or for red/yellow pixels.

```{uri="src/epaper.rs" diff=v2_epd_init_call_delay ref=v2_epd_display_startup a=8 b=20 .rust}

```

```{uri="src/epaper.rs" diff=v2_epd_init_call_delay ref=v2_epd_display_startup a=61 .rust}

```

![Display startup (initial startup time of ~15 seconds trimmed)](media/epaper_startup.webm){width=100%}

That's a wrap for our second post! Next I'm going to either explore translating our
button presses to Morse code or using our radio.
