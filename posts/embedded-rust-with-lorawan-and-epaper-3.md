---
title: Embedded Rust with LoRa and ePaper - State machines
date: "Jan. 21, 2021"
repo: "https://github.com/michaelbeaumont/farsign"
series: 3
issue: 3
...

[previous]: embedded-rust-with-lorawan-and-epaper-2
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
[next]: embedded-rust-with-lorawan-and-epaper-4

In the [previous post][previous], we worked on getting the display powered up
and displaying a splash on startup.

Today we'll start translating into morse code using timers and state machines.
We talked a little about timers in the last post where they were an
implementation detail of `Delay`. Today we'll use them again, except this time we'll
control them directly.

## State machines

Devices that interact with the real world can often be modelled through an
abstraction called a _state machine_. We can usually think of devices
as being in a _state_ at any specific moment. They remain in this state until
some event occurs, like a button press or a change in input signal, at which
point they _transition_ to another state. The behavior of our device can be expressed
as a set of states, each of which has a set of possible outgoing transitions to some
state. A specific "use" of our device can be modeled as a sequence of
transitions.

Let's model the behavior of our device this way:

![Morse code state machine](media/statemachine.svg){id=statemachine}

Enumerating the components of our state and our transitions like this is a
great way to make sure we've covered everything, including edge cases.

The idea is that as we press and hold the button, we move from a short press to a long
press. When we release, we push a new dot/dash onto the list of dots/dashes and when we
haven't held the button in a longer time, the
current sequence of dots/dashes (𝑣) is transformed into a letter which is appended to
the sequence of letters. When we hold a very long press, we transmit the sequence of
letters (𝑤), but only if we don't have any dots/dashes yet.

The state of our device can be described by the LED state, the sequence of letters (𝑤)
as well as the current accumulated sequence of dots/dashes (𝑣).

The transitions in our case are time passing and presses/releases of the button.

## Creating a `MorseMachine`

We already have a type that holds our status lights. Let's create a new `struct`
that we'll use in `src/main.rs` to hold and manipulate the state of the current morse code,
i.e. the sequence of dots/dashes.

```{uri="src/machine.rs" ref=v3_machine_skel .rust}

```

The idea will be to tell the struct when an external event occurs.
We'll call `tick` every time our timer
interrupt fires and call `press`/`release` when our GPIO interrupt fires.

Calling `tick` can cause more than one kind of transition which we're going to
handle in `main` so let's have it return a `Transition`. Notice `press` always
leads to the "short press" state and although `release` modifies the current sequence,
i.e. the _internal_ state of `MorseMachine`, it always leads to a "waiting" state.

Looking again at [our state machine](#statemachine) we can see that a `tick` of
the clock can lead to the following state transitions:

```{uri="src/machine.rs" ref=v3_add_statechange diff=^! chunk=0 .rust}

```

## Hooking interrupts to `MorseMachine`

We'll put a `MorseMachine` into a `Mutex<RefCell<_>>` to share between our
interrupts:

```{uri="src/main.rs" ref=v3_machine_skel_main chunk=1 diff=^! .rust}

```

```{uri="src/main.rs" ref=v3_machine_skel_main chunk=3 diff=^! .rust}

```

Let's hook it up to the `EXTI2_3` interrupt handler and get rid of our
dummy code from [the last chapter][previous]:

```{uri="src/main.rs" ref=v3_machine_skel_main chunk=5 diff=^! .rust}

```

Our [microcontroller reference][uc-reference] describes a general-purpose
timer peripheral called `TIM2` whose interrupt handler we can use to call
`tick`:

```{uri="src/main.rs" ref=v3_machine_skel_main diff=^! chunk=4 .rust}

```

Our HAL crate provides a
[`Timer`](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/timer/struct.Timer.html)
struct to encapsulate the timer which we'll access through a `Mutex<RefCell<_>>`.

```{uri="src/main.rs" ref=v3_machine_led_timer diff=^! chunk=0 .rust}

```

```{uri="src/main.rs" ref=v3_machine_led_timer diff=^! chunk=1 .rust}

```

Let's now hook up the state changes from `press`/`release` as well as `tick`
to our status LEDs and timer. We start the timer with
[`listen`](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/timer/struct.Timer.html#method.listen-1)
whenever we get a button press.

```{uri="src/main.rs" ref=v3_machine_led_timer diff=^! chunk=3 .rust}

```

We always clear the interrupt request with
[`clear_irq`](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/timer/struct.Timer.html#method.clear_irq)
and deactivate the timer with
[`unlisten`](https://docs.rs/stm32l0xx-hal/0.6.2/stm32l0xx_hal/timer/struct.Timer.html#method.unlisten-1)
when we get a new letter in `tick`:

```{uri="src/main.rs" ref=v3_machine_led_timer diff=^! chunk=2 .rust}

```

### Flash on timeout

One UX optimization we can make is to flash our LED when the timeout is
triggered, so the user knows they can start the next letter. It's not really a
part of our morse code state machine so we'll track it in the `TIM2` interrupt
handler. The LED turns on for a few ticks then off again and `unlisten`ing the
timer instead of directly at timeout:

```{uri="src/main.rs" ref=v3_timeout_flash diff=^! chunk=0 .rust}

```

```{uri="src/main.rs" ref=v3_timeout_flash diff=^! chunk=1 .rust}

```

## Finishing up the machine

Let's implement the state tracking and transitions in `MorseMachine`.

### Button state

We're going to maintain the state of our button inside the `MorseMachine` as
well, so let's create an `enum` with the different possible states:

```{uri="src/machine.rs" ref=v3_add_button diff=^! chunk=0 b=13 .rust}

```

Internally, we really only need a count of how many ticks have passed as well
as whether the button is being held. We offer a `state` method to return a
`State` as well as `tick` to advance the tick count.

```{uri="src/machine.rs" ref=v3_add_button diff=^! chunk=0 a=15 .rust}

```

Standard morse code uses a 1:3 ratio of dot:letter-spacing duration so we
set that in `new` and leave the transmit press duration the same as the
letter-spacing:

```{uri="src/machine.rs" ref=v3_init_button_machine diff=^! chunk=0 .rust}

```

### `press`

Looking again at
[our state machine](#statemachine) we can see that [`press`](#cb15-20) is complete because
a press always leads to a short press state and doesn't affect our morse code.

### `release`

When the button is released, our ouput state definitely depends on whether we
were holding a short, long or very long press and we _update_ the current morse
code. Let's sketch out an interface for a morse code value, i.e. a dot/dash
accumulator. I'm going to leave the implementation out for now but we'll briefly
go over it at [the end of this post](#morsecode-implementation).

```{uri="src/morse.rs" ref=v3_morse_code_machine .rust}

```

Import to note here is that we've expanded the alphabet with a sentinel value
to mean "ready to transmit".

In fact, our [`Button::state`](#cb14-29) function from earlier wasn't correct. We only
want to enter a very long press if [we haven't started accumulating dots and
dashes](#statemachine), so let's fix that up:

```{uri="src/machine.rs" ref=v3_morse_code_machine diff=^! chunk=1 .rust}

```

```{uri="src/machine.rs" ref=v3_morse_code_machine diff=^! chunk=2 .rust}

```

We'll track a `MorseCode` in our `MorseMachine`:

```{uri="src/machine.rs" ref=v3_morse_code_machine diff=^! chunk=4 .rust}

```

Now we can complete `release` and update our `MorseCode` according to our
`State`:

```{uri="src/machine.rs" ref=v3_morse_code_machine diff=^! chunk=5 .rust}

```

We make a little optimization to immediately timeout our button when the user
enters a `TRANSMIT` character.

```{uri="src/machine.rs" ref=v3_morse_code_machine diff=^! chunk=3 a=4 b=6 .rust}

```

### `tick`

For `tick` we're going to have to come up with an `Option<Transition>`. The idea
is to tick our `button` and see if there's been a change in `State`, these are
all [the transitions which are triggered by time passing](#statemachine).

The interesting part is the "waiting on dot/dash timeout"
transition where we lookup our accumulated dots/dashes
into a `u8` (an ASCII character).

```{uri="src/machine.rs" ref=v3_morse_machine_tick diff=^! chunk=0 .rust}

```

## `MorseMachine` initialization

Finally we can setup `MorseMachine` from `main` and pick a tick of 10ms with a
dot to dash transition of 20\*10ms, which feels pretty comfortable.

```{uri="src/main.rs" ref=v3_complete_init_mm diff=^! chunk=1 .rust}

```

That concludes everything we need to start speaking morse code, check
it out!

![eso.](media/first_morse_code.webm){width=100% data-external=1}

Hopefully this third post served as an introductory but not completely trivial
example of translating the behavior of an embedded
device into a state machine and then into code.

The [next post][next] covers moving over to RTIC and some refactors.

---

#### `MorseCode` implementation

There are multiple ways to implement the accumulation and lookup of morse code.
I chose to do it by essentially encoding a binary tree of possible morse code values
into an array and keeping a `u8` pointer into the tree, moving left or right down the
branches as new dots/dashes come in.
[Check it out on Github](https://github.com/michaelbeaumont/farsign/tree/v3_morse_code/src/morse.rs).

#### Edits

- Improved `machine::Transition` variants.
