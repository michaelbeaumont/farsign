---
title: Farsign
repo: "https://github.com/michaelbeaumont/farsign/tree/"
---

This series is going to be a showcase of embedded development with Rust with a little bit
of SDR thrown in.

I especially want to show how easy it is to get started with Rust and microcontrollers.
At the same time, I want to avoid just dumping information and
I want to show how one goes about searching for and finding the specifics that sometimes seem
so elusive when doing embedded development.

While this series is specifically written for beginners to embedded development, I do assume
some Rust knowledge. I also won't repeat things that are better explained in the amazing
[Rust Embedded Book](https://rust-embedded.github.io/book) so I'll periodically
be linking to passages there. I will also link to source code of the
various foundational crates.

## Goals

The device we're going to build will be pretty simple. It should be able
to broadcast morse code messages using low-power, long range radio as well as
receive and display them with an ePaper screen.

The plan is to first introduce a prototype running on a discovery board,
then design a custom PCB and finally a custom case to house everything.
We'll be using a B-L072Z-LRWAN1 discovery board with a 2.9-inch, three color ePaper
display from Waveshare.

The first post will jump right in to managing the peripherals on the discovery
board, using the radio and the display, and taking advantage of interrupts and timers.
However I'm also including an additional prologue post that goes over
initializing things but a lot is already covered in the Embedded Book,
please read that if you feel like you're missing something.

## Design

Like I mentioned, our device should include some way of:

- inputting morse code
  - the board has a configurable button which certainly suffices for morse code.
- giving _immediate_ user feedback
  - the ePaper display is relatively slow and using it for feedback would
    drastically decrease the speed with which the user can input letters.

Our discovery board includes three LEDs however, these should be enough to convey the needed information.

### Input

Obviously, we want a dot to be a short button press and a dash to be a long button press.
While it would be possible to transmit every letter as soon as it's entered,
this would mean the _other_ device is constantly drawing to the screen, queueing characters
and waiting for the screen to refresh. Let's instead send strings of characters.
We then need a "transmit" input. Let's make it an _extra_ long press!

### Feedback

While I'm inputting dots or dashes with the button, I want to be able to see if
the press that's currently active is long or short, so let's have blue LED on be
short and green LED on be long. We've got the very long press too, so let's have
that turn the red LED on.

## Kick off

We're starting with a `cargo` project initialized in such a way that we can
build and run our application on our B-L072Z-LRWAN1.

```{uri="src/main.rs" ref=v1 a=2 b=4 .rust}

```

```{uri="src/main.rs" ref=v1 .rust}

```

If you want to follow along, make sure you can `cargo run` at this point.
If you're startng with this post, you'll notice the magic identifier `pb`. We found this from
looking through the [discovery board datasheet]() (`/blue`) until we found this diagram:

[leds.png]

This also tells us how to address our blue and red LEDs;

```{uri="src/main.rs" ref=v1_rgb diff=v1 a=20 .rust}

```

We don't want to manage our lights separately all the time. Our three lights
together represent the status of our system, so let's use the type system to
express that. This way we make also give semantic meaning to our LED status
idea. We can create a readable interface and disallow meaningless 
configurations, e.g. we already know we won't be turning on RG and turning off B.

Let's think about how we're going to eventually handle key presses. We'll
probably want to _act_ when we notice there's been a button down event. Now, we
can definitely busy loop until we notice a key press. What about something
like:

```rust
loop {
    if button_down() {
        // turn on and LED
    }
}

```

This would work, but there's another way.

Introducing interrupts! If you're not familiar, there's [an intro in in the Embedded
book on how to use them in Rust](https://rust-embedded.github.io/book/start/interrupts.html).

Checking [our datasheet](TODO) again, we can see that our microcontroller has
interrupts for GPIO changes. We can also see [our discovery board has one "user
button"](TODO) that's been wired up to a GPIO port. Let's see if we can make our light
flash off a button press.

```{uri="src/main.rs" ref=v1_tim2_skel diff=v1_tim2_skel~ a=25 .rust}

```

You'll notice we need to get our LEDs but we're setting them up in our
entrypoint function. We'll need to share them. We've now got concurrency in our
program so let's see [what the Embedded book has to say about that](https://rust-embedded.github.io/book/concurrency/).
The TL;DR is that we've got two options. One is to put shared data in a
structure where Rust will allow us to share access (a Mutex), which is the option
we'll be taking. Another would be to use [a framework](https://rtic.rs/) to manage
the boilerplate of using Mutexes. For a real project, this option is the right
one but I think it's always useful to understand why and whether frameworks are
needed before jumping straight into one. I may cover moving our project over to
RTIC in a future post.

---

## Setup

Let's jump right in and initialize a new project:

```shell
❯ cargo new farsign
     Created binary (application) `farsign` package
```

We're going to start basic and just try to blink our LEDs! But in order to run
our binary on our microcontroller, we need to take care
of some embedded and device specific things.

Some things are going to be very different to how we normally work with Cargo
and Rust.

## Architecture

Let's look at [the documentation for our discovery board](https://www.st.com/resource/en/data_brief/b-l072z-lrwan1.pdf) a little closer. The heart of our board is a
[LoRa module by Murata](#TODO MWX1ZZABZ-091). We'll talk more about LoRa a little later. Our module is powered by a STM32L072CZ microcontroller which contains an ARM Cortex-M0+ core. Looking at [the ARM docs](https://developer.arm.com/documentation/ddi0484/c/Introduction/Product-documentation--design-flow-and-architecture/Architecture-and-protocol-information?lang=en) tells us these cores implement the ARMv6-M architecture.

Rust compiles code using LLVM and the final byte code that is compiled down to
is defined by a _target_. If you're using Linux for example and have installed
rust using `rustup`, you can run:

```
> rustup target list --installed
x86_64-unknown-linux-gnu
```

which should output something like the above. Since ARM Cortex-M0+ is an
entirely different architecture, we'll need to install a different target, `thumbv6m-none-eabi`:

- `thumbv6m` is the name of the instruction set compatible with ARM Cortex-M0+
- `none` refers to the lack of an operating system
- `eabi` refers to the [ARM embedded application binary interface](https://developer.arm.com/documentation/ihi0036/latest/) which structures the binaries in an interoperable way

We need to tell cargo to target this architecture when building our program:

```.cargo/config
[build]
target = "thumbv6m-none-eabi"
```

#TODO memory.x

But we also need to install this target on our machine! Let's break out `rustup target` again:

```shell
rustup target add thumbv6m-none-eabi
```

Below is all you need for a basic blink program with our board. For more details
about the attributes found here, see
[Program Overview in the Embedded Book](https://rust-embedded.github.io/book/start/qemu.html#program-overview).

```rust
// Disable linking to `std` and instead link to `core`.
// `core` is a library with very limited functionality that
// doesn't assume the existence of a memory allocator for example.
#![no_std]
// Skip emitting the main symbol.
#![no_main]

// This attribute from `cortex-m-rt` is a proc macro
// that sets up the function to be executed by the reset handler,
// which is called when the device is powered on.
// See https://github.com/rust-embedded/cortex-m-rt/blob/96525a64197049d11cfc8cb5cc2c4dc9b5240e42/src/lib.rs#L244
#[entry]
fn main() -> ! {
    // In order to enforce exclusive access to system resources
    // we can only call `.take()` here once, additional calls will fail.
    let cp = pac::CorePeripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Configure the clock.
    let mut rcc = dp.RCC.freeze(hal::rcc::Config::hsi16());
    // Setup the SYSTICK for arbitrary delaying.
    let mut syst_delay = delay::Delay::new(cp.SYST, rcc.clocks);

    // Get exclusive access to the GPIO B port.
    let gpiob = dp.GPIOB.split(&mut rcc);
    // The magic value `pb5` can be found in the [#TODO] discovery board doc.
    let green_pin = gpiob.pb5.into_push_pull_output();

    green_pin.set_high().unwrap();
    // `Delay::delay_ms(x)` loops until the SYSTICK timer
    // has counted down for `x` ms.
    syst_delay.delay_ms(1000);
    green_pin.set_low().unwrap();
}
```

Reference [concurrency](https://rust-embedded.github.io/book/concurrency/index.html)
