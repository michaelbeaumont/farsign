---
title: Embedded Rust with LoRa and ePaper - LoRa
date: "Jan. 21, 2021"
repo: "https://github.com/michaelbeaumont/farsign"
series: 5
issue: 5
...

[previous]: embedded-rust-with-lorawan-and-epaper-3
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
[sx1276-datasheet]: https://www.mouser.com/datasheet/2/761/sx1276-1278113.pdf

In the [previous post][previous], we worked on getting the morse code logic
working in our program, so that we can translate presses into morse code and
into letters.

In this post, I'll introduce the radio functionality. The first step will focus
on the "high-level" interfacing of our program with the radio and how we
accumulate letters and deal with transmitting them. We'll assume a driver
library that we can use to communicate with the module.

Cracking open both the [the SX1276 datasheet][sx1276-datashet] and the [xXXX] 
tells us that we can monitor the `DIO0` pin to tell us when we're done receiving
or transmitting a packet. Let's connect that `DIO0` pin on our discovery
board to the `PA0` pin so we can use `EXTI` to run an interrupt handler.

We'll also assume we have `fn transmit(&[u8])` and `fn read(&mut [u8]) -> &[u8]` 
with which we start to transmit or receive a byte string over our radio.

When we get the `machine::Transition::Transmit` transition we can start

### LoRa over SPI

Let's take another look at the [datasheet for our LoRa module][module-datasheet]
(the component our microcontroller lives inside of).

It doesn't tell us too much, but it does tell us that the radio module is
connected to the microcontroller over `SPI1`. Unfortunately, it doesn't mention
over which pins it's connected and the `STM32l0xx` family offers multiple
options for `SPI1_{MISO,MOSI,...}` pins TODO. Altogether, we're left with a
large number of possibilities. Luckily, some internet sleuthing tells us that
others have had success with the combination "XXXX".
>>>>>>> 23f4200 (WIP)

