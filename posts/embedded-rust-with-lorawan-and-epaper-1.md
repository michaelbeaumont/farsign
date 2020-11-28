---
title: Embedded Rust with LoRaWAN and ePaper - GPIO & Interrupts
date: "Nov. 29, 2020"
repo: "https://github.com/michaelbeaumont/farsign"
series: 1
issue: 1
...

[next]: embedded-rust-with-lorawan-and-epaper-2.html
[discovery-board]: https://www.st.com/en/evaluation-tools/b-l072z-lrwan1.html "Discovery board"
[repo]: https://github.com/michaelbeaumont/farsign
[discovery-manual]: https://www.st.com/resource/en/user_manual/dm00329995-discovery-kit-for-lorawan-sigfox-and-lpwan-protocols-with-stm32l0-stmicroelectronics.pdf "discovery board manual"
[uc-datasheet]: https://www.st.com/resource/en/datasheet/stm32l072v8.pdf "Microcontroller datasheet"
[uc-reference]: https://www.st.com/resource/en/reference_manual/dm00108281-ultralowpower-stm32l0x2-advanced-armbased-32bit-mcus-stmicroelectronics.pdf "Microcontroller reference manual"
[what-is-lorawan]: https://lora-alliance.org/resource-hub/what-lorawanr
[embedded-book]: https://rust-embedded.github.io/book

Embedded Rust is ready; high-quality, production software is being written on industry-standard
bare metal microcontrollers.
The community has produced so much amazing documentation focusing on the initial stages of using Rust on embedded
hardware. With this series I want to move somewhat beyond that, from getting
started to more immediate embedded Rust.

We're going to use Rust to code up a device that takes advantage of multiple hardware
peripherals including a display and a radio module. It will be able
to broadcast and receive morse code messages using low-power, long range radio
([LoRaWAN][what-is-lorawan]) as well as
display them on an ePaper screen.

The plan is to first build up a prototype running on a discovery board,
then design a custom PCB and finally, a custom case to house everything.

This first few posts will jump right in to initializing and using the peripherals on the discovery
board, using the radio and the display, and taking advantage of interrupts and timers.
We'll be using a [B-L072Z-LRWAN1 discovery board][discovery-board] with a 2.9-inch, three-color ePaper
display from Waveshare.

![SOS](media/send_full.webm){width=100%}

Yes, three-color ePaper really is that slow.

# Goals

This series is written for beginners to embedded development. I want to show how
easy it is to quickly get working results with Rust and microcontrollers.
At the same time, I want to avoid just dumping information and
I want to show how one goes about searching for and finding the details that sometimes seem
so elusive when doing embedded development.

This series is specifically written for beginners _to embedded_ and I do assume
some Rust knowledge. I won't repeat too much that's better explained in the amazing
[Rust embedded Book][embedded-book] however, so I'll periodically
be linking to passages there. I may also link to source code of the
various foundational crates.

We'll be using existing driver crates to interface with our devices, not delving into raw
SPI communication, etc. Still, I ended up making some contributions to these crates
for this project so I may dive into the datasheets here or there where it's
interesting.

# Design

Our device should include some way of inputting morse code, transmitting it and displaying responses
back to the user.

## Input

We're going use the big button on our discovery board
to input Morse code so let's have • be a short button press and − be a long button press.
While it would be possible to transmit every letter as soon as it's entered,
this would mean the _other_ device is constantly drawing to the screen, queueing characters
and waiting for the screen to refresh. Let's instead send strings of characters.
We then need a "transmit" input. Let's make it an _extra_ long press!

## Feedback

While I'm inputting dots or dashes with the button, I want to be able to see if
the press that's currently active is long or short. We do have the ePaper
display but the refresh rate is very slow and wouldn't be usable for the
immediate feedback needed here. Our discovery board includes three LEDs however,
these should be enough to convey the needed information.
Let's have a [•]{style="color:lightblue"} turn on blue, a [− ]{style="color:green"} turn on green
and ["transmit"]{style="color:red"} turn our red light on.

# Kick off

First, a quick overview of our board. We have the **B-L072Z-LRWAN1** discovery board,
which contains our radio module, the **CMWX1ZZABZ-091**.
This module runs on a **STM32L072CZ microcontroller**.
We'll mostly be programming against this microcontroller but
we'll find different information and
levels of detail by looking at the datasheets and manuals for each of these
parts.

We're starting with a `cargo` project initialized in such a way that we can
build and run our application on our B-L072Z-LRWAN1.

```{uri="src/main.rs" ref=v1 .rust}

```

If you want to follow along, make sure you can run [at this point](https://github.com/michaelbeaumont/farsign/tree/v1).

In order to fully understand the code here so far, it may be worth reading the some
chapters of the Embedded Book on
[the _hardware access layer_ system](https://rust-embedded.github.io/book/portability/index.html#what-is-embedded-hal)
and [using HAL crates](https://rust-embedded.github.io/book/start/registers.html#using-a-hal-crate).

We'll mostly be dealing with the HAL crate for our board,
[`stm32l0xx_hal`](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/),
which implements `Trait`s defined by
[the `embedded-hal` crate](https://docs.rs/embedded-hal/0.2.4/embedded_hal/).

Near the end of this code snippet you'll notice the magic identifiers `GPIOB` and `pb5`,
which give us the GPIO pin attached to our green LED.
We know the name of this GPIO port and pin from looking through the [discovery kit schematics][discovery-manual] until we found this diagram:

![Page 33](LEDs.png)

This also tells us how to address our blue and red LEDs:

```{uri="src/main.rs" ref=v1_rgb diff=v1 a=20 .rust}

```

In order to give feedback on the active press, we'll need to turn one LED on and the
others off. We don't want to manage our lights color by color all over our
project.
Our three lights _together_ represent the status of our system, so let's use the type system to
express that. We can create a semantically meaningful interface and disallow meaningless
configurations, i.e. we already know we'll only ever have one LED on.

```{uri="src/status.rs" ref=v1_status .rust}

```

Let's add a function for turning on our blue light for a short press:

```{uri="src/status.rs" ref=v1_status_on_short diff=v1_status_on_short~ a=12 b=16 .rust}

```

If we try to build at this stage, we get an error:

```rust
error[E0599]: no method named `set_low` found for type parameter `R`
in the current scope
  --> src/status.rs:15:18
   |
15 |         self.red.set_low().unwrap();
   |                  ^^^^^^^ method not found in `R`
   |
   = note: the method `set_low` exists but the following trait bounds
were not satisfied:
           `R: stm32l0xx_hal::prelude::_embedded_hal_digital_OutputPin`
           which is required by `R: stm32l0xx_hal::prelude::OutputPin`
```

That makes sense since our `R`, `G` and `B` types could be anything at this point.
We'll need to add that bound to our `StatusLights` `struct`.
If we check the `impl` of that trait for `PB` we also see that the trait has an
associated `Error` type which is `Void` in our case:

```rust
impl<MODE> OutputPin for PB<Output<MODE>> {
    type Error = Void

    fn set_high(&mut self) -> Result<(), Self::Error>

    fn set_low(&mut self) -> Result<(), Self::Error>
}
```

Let's add those bounds to our `impl`:

```{uri="src/status.rs" ref=v1_status_constraints diff=v1_status_constraints~ a=1 b=2 .rust}

```

```{uri="src/status.rs" ref=v1_status_constraints diff=v1_status_constraints~ a=10 b=13 .rust}

```

`Void` represents an uninhabited type, a type of which we can never produce a value,
ergo `stm32l0xx_hal` has infallible GPIO pin writes.

Instead of restricting `type Error` to `Void`, another option would be to let it
be a type argument to our `impl` and add the `Debug` bound, since `unwrap`
requires it. This would give us marginally more portability at the cost of
complicating our type signatures.

## Interrupts

Let's start simple and have our button turn off our the lights when pressed and
then turn the blue LED on again.

We want to take action when we notice our button has been pressed.
What about checking our button in the main loop, something like this:

```rust
let mut leds_on = false;
loop {
    if button_down() && !leds_on {
        leds_on = true;
        // turn off LEDs
    } else if button_up() && leds_on {
        leds_on = false;
        // turn on LEDs
    }
}

```

This is called _polling_, repeatedly checking whether a condition has changed, and it's a
perfectly legitimate way of handling changes.
There's another way though, we can handle events directly as they occur.
If you're not familiar with _interrupts_, the Embedded book goes over [how they
work in Rust](https://rust-embedded.github.io/book/start/interrupts.html) and
[how exceptions (of which interrupts are a subset)
work](https://rust-embedded.github.io/book/start/exceptions.html).

Interrupts have the advantage of immediately triggering an event handler no matter
where in our code we're executing and so allow for a faster response time.
Both approaches would work fine in our button case but I want to show you how to
setup interrupts for GPIO pins with Rust.

Looking at [our discovery manual][discovery-manual]
we see it has one user button that's been wired up to a GPIO port.
Let's see if we can make our light flash off a button press.
Let's create a function that will serve as our interrupt handler:

```{uri="src/main.rs" ref=v1_inter_skel diff=v1_inter_skel~ a=26 .rust}

```

### Concurrency

You may have wondered how we can get access to our LEDs in our handler
considering it's called without arguments. In our entry function we use `pac::Peripherals::take()`
but remember, we can only call it once. The LEDs need to be shared
between our main loop and interrupt handlers.
We have _concurrency_, let's see [what the Embedded book has to say about it](https://rust-embedded.github.io/book/concurrency/).

The TL;DR is that we've got two options. One is to put our data in a
structure to which Rust will allow us to share access, which is the option
we'll be taking. With `Mutex` we can obtain an exclusive lock on the data
contained inside, which allows us to put it in a `static` variable.
`static` requires `Sync` as static variables can be accessed from any point in our application.
Inside the `Mutex` we put our struct wrapped in a `RefCell`.
`RefCell` is necessary to dynamically give us a mutable reference
to a single value, the initial struct we create in `main`.

Another option would be to use [a framework](https://rtic.rs/) to manage
the boilerplate of creating and accessing these mutexes in interrupt handlers.
We should consider this option but I think it's always useful to understand why and whether frameworks are
needed before jumping straight into one. I will likely cover moving our project over to RTIC in a future post.

```{uri="src/main.rs" ref=v1_status_typed_holes diff=v1_status_typed_holes~ a=11 b=17 .rust}

```

The fact that we've bundled our LEDs together into one struct makes dealing with
mutexes much more ergonomic.

Our three generic type parameters need to be instantiated with some types
representing our LEDs. If we try something (clearly false) we get a pretty helpful
error message:

```{uri="src/main.rs" ref=v1_set_status_typed_holes diff=v1_set_status_typed_holes~ a=13 b=14 .rust}

```

```{uri="src/main.rs" ref=v1_set_status_typed_holes diff=v1_set_status_typed_holes~ a=34 b=40 .rust}

```

```rust
error[E0308]: mismatched types
  --> src/main.rs:40:48
   |
40 |         *STATUS.borrow(cs).borrow_mut() = Some(status);
   |                                                ^^^^^^ expected `u64`,
found struct `stm32l0xx_hal::gpio::gpiob::PB7`
   |
   = note: expected struct `StatusLights<u64, u64, u64>`
              found struct `StatusLights<
stm32l0xx_hal::gpio::gpiob::PB7<Output<PushPull>>,
stm32l0xx_hal::gpio::gpiob::PB5<Output<PushPull>>,
stm32l0xx_hal::gpio::gpiob::PB6<Output<PushPull>>
>`
```

So we can see that our HAL crate `stm32l0xx_hal` embeds even the pin number
we're using into our type. This may seem like a strange approach at first but the embedded book elaborates
[on this idea](https://rust-embedded.github.io/book/static-guarantees/design-contracts.html) and
[how to apply it for GPIO](https://rust-embedded.github.io/book/design-patterns/hal/gpio.html).

Writing out these types here isn't really great for flexibility or readability, though. We can improve things
a little bit, our HAL [provides `downgrade` functions on our pins](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/gpio/gpiob/struct.PB5.html#method.downgrade):

```rust
impl<MODE> PB5<Output<MODE>> {
  pub fn downgrade(self) -> PB<Output<MODE>>
```

We can at least downgrade all of our LEDs into a type representing _some_ pin on
the GPIO B port.

```{uri="src/main.rs" ref=v1_status_downgrade diff=v1_status_downgrade~ a=13
b=16 .rust}

```

```{uri="src/main.rs" ref=v1_status_downgrade diff=v1_status_downgrade~ a=36
b=42 .rust}

```

Now we're free to get a reference to the status lights in our interrupt handler:

```{uri="src/status.rs" ref=v1_inter_status_off diff=v1_inter_status_off~ a=22
.rust}

```

```{uri="src/main.rs" ref=v1_inter_status_off diff=v1_inter_status_off~ a=47
.rust}

```

### EXTI

We now have a function that's ready to handle our interrupt.
We're not done yet, because this interrupt isn't yet enabled and wired up to our
button. First of all, we'll need our GPIO pin which for our button is `B2`
(check [our board manual][discovery-manual]),
configured as an input:

```{uri="src/main.rs" ref=v1_gpio_interrupt diff=v1_gpio_interrupt~ a=54 b=54
.rust}

```

If look into our [microcontroller datasheet, chapter 3.7][uc-datasheet],
we read that our we can configure interrupts for changes on GPIO pins.
Changes are always either a falling edge (high to low state) or a rising edge (low to high state).

Check out the [microcontroller reference manual][uc-reference], chapter 13, for a very detailed
explanation of the responsible peripheral,
the **Extended interrupt and Event Controller (EXTI)**.
What we need is a link between our button on `PB2` and the name of an interrupt
handler.
On [page 285 of our
microcontroller reference][uc-reference] we find that `PB2` is mapped to the
interrupt line `EXTI2`.
The idea here is that every pin `x` of each GPIO port is mapped to one
`EXTI` interrupt line, i.e. for `EXTI2` we need to select which of the pins `Px2` should trigger
the interrupt, which [chapter 10.2.4, page 256][uc-reference] describes.

Looking then at [chapter 12.3, page 280][uc-reference], we find a list
of _interrupt vectors_ (interrupt handlers) and find an entry at position 6 for
`EXTI Line2 and 3 interrupts` with the name `EXTI[3:2]`.

Our `stm32l0xx_hal` crate gives us an API to the EXTI peripheral through
the
[`Exti`](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/exti/struct.Exti.html) struct and the [`listen_gpio`](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/exti/struct.Exti.html#method.listen_gpio) function.

Looking at the function signatures, we need the raw peripherals `EXTI` and `SYSCFG`. Both our `Port` (`PB` in this case)
and our [`GpioLine`](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/exti/struct.GpioLine.html) depend on the pin we're using.
We'll create both of these dynamically from our `button`. We want to trigger both on rising and falling edges
(i.e. we want to be notified when the button is either pressed or released).
We use the [`NVIC::unmask` function](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/pac/struct.NVIC.html#method.unmask)
to enable the interrupt handler corresponding to our line, which is an `unsafe` operation.

Put it all together and we have:

```{uri="src/main.rs" ref=v1_gpio_interrupt diff=v1_gpio_interrupt~ a=52 b=63
.rust}

```

Finally, we find the interrupt for our line represented by the
[`interrupt` enum variant `EXTI2_3`](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/pac/enum.interrupt.html#variant.EXTI2_3).
If we name our function `EXTI2_3` and give it the `interrupt` attribute, we set up our function as the handler for `EXTI[3:2]`:

```{uri="src/main.rs" ref=v1_gpio_interrupt diff=v1_gpio_interrupt~ a=65
.rust}

```

If we run our application now, we can see our lights go off and then on in
response to a button press! But... it keeps blinking after releasing the
button...

The way interrupts work requires us to manually clear the interrupt, otherwise
it will keep firing over and over again. We'll add a `Mutex` for our button so
we can get the pin number.

```{uri="src/main.rs" ref=v1_gpio_interrupt_clear diff=v1_gpio_interrupt_clear~ a=22 b=24
.rust}
```

```{uri="src/main.rs" ref=v1_gpio_interrupt_clear diff=v1_gpio_interrupt_clear~ a=50 b=63
.rust}
```

```{uri="src/main.rs" ref=v1_gpio_interrupt_clear diff=v1_gpio_interrupt_clear~ a=74 b=78
.rust}
```

One final feature we can add is checking whether the button is pressed or
released. Right now you'll notice we treat a release the same as a press. Let's
keep our LED off while pressed and turn it back on when released. To do this we
need to check the level of our pin in our interrupt.

```{uri="src/main.rs" ref=v1_gpio_interrupt_off_on diff=v1_gpio_interrupt_off_on~ a=70 b=89
.rust}
```

That may seem like a lot of code for turning off and on an LED but it will serve as
the basis for the rest of our project.

![Blinky button](media/off_on.webm){width=100%}

In [the next installment][next] we're going to look at using our ePaper display.
