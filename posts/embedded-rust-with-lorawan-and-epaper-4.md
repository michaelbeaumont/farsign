---
title: Embedded Rust with LoRa and ePaper - Refactoring & RTIC
date: "Mar. 12, 2021"
repo: "https://github.com/michaelbeaumont/farsign"
series: 4
issue: 4
...

[previous]: embedded-rust-with-lorawan-and-epaper-3
[discovery-board]: https://www.st.com/en/evaluation-tools/b-l072z-lrwan1.html "Discovery board"
[discovery-manual]: https://www.st.com/resource/en/user_manual/dm00329995-discovery-kit-for-lorawan-sigfox-and-lpwan-protocols-with-stm32l0-stmicroelectronics.pdf "discovery board manual"
[uc-datasheet]: https://www.st.com/resource/en/datasheet/stm32l072v8.pdf "Microcontroller datasheet"
[uc-reference]: https://www.st.com/resource/en/reference_manual/dm00108281-ultralowpower-stm32l0x2-advanced-armbased-32bit-mcus-stmicroelectronics.pdf "Microcontroller reference manual"
[embedded-book]: https://rust-embedded.github.io/book
[epd-waveshare]: https://github.com/caemor/epd-waveshare
[module-datasheet]: https://wireless.murata.com/pub/RFM/data/type_abz.pdf
[rtic]: https://rtic.rs/
[rtic-book]: https://rtic.rs/0.5/book/en/by-example.html
[rtic-tasks]: https://docs.rs/cortex-m-rtic/0.5.6/rtic/attr.app.html#c-task
[rtic-init]: https://docs.rs/cortex-m-rtic/0.5.6/rtic/attr.app.html#a-init

In the [previous post][previous], we created a state machine and implemented it
in Rust as a combination of interrupt handlers and structs.

In this post, I want to cover converting over to [RTIC][rtic], which is a sort of
lightweight real-time OS/framework that focuses on making it easier to
handle concurrency on `cortex-m` devices.
The main thing RTIC is going to give us is better
abstractions so that we can focus more on the "business logic" of our device.

I'll also touch on some refactoring to improve our interrupt usage and
`Timer` handling.

## RTIC

Before moving up an abstraction level, I like making sure I understand things
well enough at the previous level so I can generally understand what the next
level is giving us and how it does so.
Now that we've seen the basic abstractions provided to us by the `cortex-m` crate,
it's a good time to move "up" to RTIC.

Migrating over was a _very_ straightforward process since the way we
structured everything maps really well to RTIC's abstractions.
By the way, there's a lot more details available in [the RTIC book][rtic-book]
so I won't go into everything here.

The main improvement we'll see is RTIC managing the tedious
`static Mutex<RefCell<Option<T>>>`s for us as well as avoiding
both the noisy critical sections and the `borrow`, `as_ref`, `unwrap` chains
necessary to use these shared values.

The idea behind RTIC is that we have a set of `Resources` that are initialized
and then shared between `Tasks`. We declare all of these things inside an item
with the `[#app]` attribute (a `const` module-level item, for reasons, but
just imagine a `mod` or crate-level attribute).
We also provide the PAC crate we're
using and ask for access to the device-specific peripherals.

```{uri="src/main.rs" ref=v4_rtic diff=^! a=50 b=51 .rust}

```

We can convert our shared resources:

```{uri="src/main.rs" ref=v4_rtic diff=^! a=33 b=37 .rust}

```

into the following `Resources` struct:

```{uri="src/main.rs" ref=v4_rtic diff=^! a=52 b=58 .rust}

```

`#[init(...)]` allows us to statically initialize a variable, which we can do
with `const fn machine::MorseMachine::new` (I updated `new` in between this and the last post
to be `const`, it could now also be used directly `Mutex<RefCell<T>>` to avoid
the `Option`).

### `init`

The [`init`][rtic-init] function is where we initialize whatever parts of our `Resources`
weren't statically initialized. In our case, it will be identical to our `main`
function except we have direct access to the `core` peripherals and the `device`
peripherals. Finally, we return `init::LateResources` which is made up of exactly the
values needed to fill in our `Resources` struct. The magic of code generation!

```{uri="src/main.rs" ref=v4_rtic diff=^! a=63 b=66 .rust}

```

```{uri="src/main.rs" ref=v4_rtic diff=^! a=140 b=144 .rust}

```

### `task`

We mark each interrupt handler as a
[`task`][rtic-tasks] and tell `RTIC` which resources we
need:

```{uri="src/main.rs" ref=v4_rtic diff=^! a=161 b=162 .rust}

```

Our function is then called with those resources passed in:

```{uri="src/main.rs" ref=v4_rtic diff=^! a=162 b=172 .rust}

```

From then on our code remains the same except we have _direct_ access to the
resources and don't have to go through `Mutex`es and `RefCell`s (there
are cases where we need to use a `lock` to get access to the resource, but
`RTIC` will enforce this if and only if necessary).

```{uri="src/main.rs" ref=v4_rtic diff=^! a=152 b=160 .rust}

```

```{uri="src/main.rs" ref=v4_rtic diff=^! a=173 b=182 .rust}

```

Our `button` handler is improved similarly.

## Fewer interrupts

In the previous post we implemented our morse code machine by counting timer "ticks",
to avoid having to deal with setting different timers. The downside of
this is that we run an interrupt every tick and most of the time nothing
happens.

### Timeless morse machine

Instead of counting, we can set up our
`Timer` to countdown to exactly the time we need, depending on our current
state, so that we only tick once per transition.
I.e. a tick moves us directly from short to long, long to very long, or waiting
to idle.

The new `MorseTimelessMachine` now no longer uses `Button` to count but
instead uses `State` directly. We also wrap the new machine with a `MorseTimingMachine`
that handles manipulating our timer peripheral and dealing with interrupts for us.

```{uri="src/machine.rs" ref=v4_machine_timer diff=^! chunk=0 a=18 b=25 .rust}

```

```{uri="src/machine.rs" ref=v4_machine_timer diff=^! chunk=0 a=35 b=50 .rust}

```

In `MorseTimelessMachine`, use `State::tick` in combination with the `current`
`MorseCode`:

```{uri="src/machine.rs" ref=v4_machine_timer diff=^! chunk=2 a=35 b=40 .rust}

```

Finally, we move all `Timer` handling into `MorseTimingMachine`, which is the
`struct` we expose for `main.rs`:

```{uri="src/machine.rs" ref=v4_machine_timer diff=^! chunk=2 a=70 b=112 .rust}

```

The upside of the `Timeless`/`Timing` distinction is that testing is significantly easier
and as a state machine, `Timeless` is much more similar to the abstract machine from [last
post][previous].

```{uri="src/machine.rs" ref=v4_machine_timer diff=^! chunk=2 a=135 b=143 .rust}

```

The less testable `Timer` handling is left up to `MorseTimingMachine`.

### Flashing lights

Instead of having a `static mut` local to our `timer` task, we can put the flash
functionality into `StatusLights`. Similar to the new `Machine`, we let the
`Flasher` handle our `Timer`.

```{uri="src/status.rs" ref=v4_status_flash diff=^! chunk=1 .rust}

```

## Improvements

The tasks in `src/main.rs` are now significantly easier to read, due to making
our various state machines much more explicit. We can immediately see each
transition as well as what triggers them just from looking at the interrupt
handlers/tasks.

```{uri="src/main.rs" ref=v4_status_flash diff=v4_rtic chunk=4 .rust}

```

```{uri="src/main.rs" ref=v4_status_flash diff=v4_rtic chunk=5 .rust}

```

This post was focused on refactoring in embedded Rust.
Hopefully it's interesting to see an example of using RTIC for a real device as well as
how to best design interactions between state machines and hardware.

The next post will (really) cover keeping a stack of inputted characters and handling the
`TRANSMIT` value to broadcast over the radio.
