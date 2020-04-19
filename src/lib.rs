/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Appache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-gpio/0.5.0")]
#![cfg_attr(not(any(test, doctest)), no_std)]
#![feature(asm, const_fn, const_in_array_repeat_expressions)]
//! # GPIO implementation for Raspberry Pi
//!
//! This crates implements the GPIO HAL for the Raspberry Pi and thus provides access to the same on this hardware.
//! While this implementation ensures with an [AtomicBool] as gate keeper that there will be only one instance of the 
//! [GpioRpi] existing it is the responsibility of the crate user to ensure wrapping of the instance into a proper and
//! safe type to enable cross thread and cross core access to the peripheral. A valid approach would be to wrap it into 
//! a [Singleton](https://https://docs.rs/ruspiro-singleton) and if necessary wrap this further into an [Arc].
//! 
//! # Usage
//!
//! ```no_run
//! use ruspiro_gpio::*;
//!
//! fn doc() {
//!     let gpio = Gpio::new().unwrap();
//! 
//!     let pin = gpio.use_pin(17).unwrap(); // assuming we can always get this pin as it is not in use already
//!     pin.into_output().high(); // set this pin to high - this may lit a connected LED :)
//! }
//! ```
//!
//! # Features
//!
//! - ``ruspiro_pi2`` Ensures the proper MMIO base memory address is used for Raspberry Pi 2, 2B
//! - ``ruspiro_pi3`` Ensures the proper MMIO base memory address is used for Raspberry Pi 3, 3B+
//! - ``ruspiro_pi4`` Ensures the proper MMIO base memory address is used for Raspberry Pi 3, 3B+
//!

/// re-export the HAL trait definitions as they need to be in scope when using the specific
/// Gpio implementation
pub use ruspiro_gpio_hal as hal;

extern crate alloc;
use alloc::{boxed::Box, format};
use ruspiro_error::*;
use core::sync::atomic::{AtomicBool, Ordering};

mod pin;
mod interface;

/// this atomic bool gates the instantiation of the Gpio to happen only once
static GPIO_ONCE: AtomicBool = AtomicBool::new(false);

/// The Raspberry Pi representation of the GPIO peripheral device
pub struct GpioRpi {
    /// for the sake of simplicity we only track which of the max. 40 pins of the Raspberry Pi are already in use
    used_pins: [bool; 54],
}

impl GpioRpi {
    /// Get a new instance of the Gpio peripheral representation. This should be called only once as it assumes no
    /// [GpioPin] is in use. If this has been called already, it returns an [Err].
    pub fn new() -> Result<Self, ()> {
        if !GPIO_ONCE.compare_and_swap(false, true, Ordering::SeqCst) {
            Ok(Self {
                used_pins: [false; 54],
            })
        } else {
            Err(())
        }
    }
}

/// If the actual Gpio instance used is dropped we would be allowed to create another one
/// so stor this fact inside the atomic bool 
impl Drop for GpioRpi {
    fn drop(&mut self) {
        GPIO_ONCE.store(false, Ordering::SeqCst);
    }
}

impl hal::Gpio for GpioRpi {
    fn use_pin(&mut self, id: u32) -> Result<Box<dyn hal::GpioPin>, BoxError> {
        if id > 53 {
            return Err(
                GenericError::with_message("Raspberry Pi supports only up to 54 GPIO Pins")
                    .into()
            );
        }
        
        if self.used_pins[id as usize] {
            Err(
                GenericError::with_message(
                    format!("Pin {} already in use", id).as_str()
                ).into()
            )
        } else {
            Ok(Box::new(pin::GpioPin::new(id)))
        }
    }

    fn release_pin(&mut self, id: u32) -> Result<(), BoxError> {
        if id > 53 {
            return Err(
                GenericError::with_message("Raspberry Pi supports only up to 54 GPIO Pins")
                    .into()
            );
        }
        
        if !self.used_pins[id as usize] {
            Err(
                GenericError::with_message(
                    format!("Pin {} was not in use", id).as_str()
                ).into()
            )
        } else {
            Ok(())
        }
    }

    fn register_event_handler_always(
        &mut self,
        gpio_pin: &dyn hal::GpioPinInput,
        event: hal::GpioEvent,
        handler: Box<dyn FnMut() + 'static + Send>,
    ) {
    }

    fn register_event_handler_onetime(
        &mut self,
        gpio_pin: &dyn hal::GpioPinInput,
        event: hal::GpioEvent,
        handler: Box<dyn FnOnce() + 'static + Send>,
    ) {
    }

    fn unregister_event_handler(&mut self, gpio_pin: &dyn hal::GpioPin, event: hal::GpioEvent) {}
}

/*

extern crate alloc;
use alloc::boxed::Box;
use ruspiro_interrupt::*;
use ruspiro_singleton::Singleton;
use ruspiro_core::*;

mod interface;
use interface::*;
mod pin;
pub use self::pin::*;

pub mod debug;

/// Static ``Singleton`` accessor to the GPIO peripheral. The ``Singleton`` ensures cross core mutual
/// exclusive access.
pub static GPIO: Singleton<Gpio> = Singleton::<Gpio>::new(Gpio::new());

/// GPIO peripheral representation
pub struct Gpio {
    used_pins: [bool; 40],
}

impl ruspiro_talents::capability::Capability for Gpio {}

impl Gpio {
    /// Get a new intance of the GPIO peripheral and do some initialization to ensure a valid state of all
    /// pins uppon initialization
    pub const fn new() -> Self {
        Gpio {
            used_pins: [false; 40],
        }
    }

    /// Get a new pin for further usage, the function of the pin is initially undefined/unknown
    /// Returns an Err(str) if the pin is already in use, otherwise an Ok(Pin)
    /// # Example
    /// ```no_run
    /// # use ruspiro_gpio::GPIO;
    /// # fn doc() {
    /// if let Ok(pin) = GPIO.take_for(|gpio| gpio.get_pin(17) ) {
    ///   // do something with the pin
    /// }
    /// # }
    /// ```
    pub fn get_pin(&mut self, num: u32) -> Result<Pin<function::Unknown, pud::Unknown>, GpioError> {
        if self.used_pins[num as usize] {
            Err(GpioError)
        } else {
            self.used_pins[num as usize] = true;
            Ok(Pin::<function::Unknown, pud::Unknown>::new(num))
        }
    }

    /// Release an used pin to allow re-usage for example with different configuration
    /// The pin's state after release is considered unknown
    /// # Example
    /// ```no_run
    /// # use ruspiro_gpio::GPIO;
    /// # fn doc() {
    /// GPIO.take_for(|gpio| gpio.free_pin(17) );
    /// # }
    /// ```
    pub fn free_pin(&mut self, num: u32) {
        // release the used pin
        // TODO: reset also pin function or other settings?
        if self.used_pins[num as usize] {
            self.used_pins[num as usize] = false;
        };
    }

    /// Register an event handler to be executed whenever the event occurs on the GPIO [Pin] specified.
    /// Event handler can only be registered for a ``Pin<Input,_>``.
    /// The function/closure provided might be called several times. It's allowed to move mutable
    /// context into the closure used.
    /// **HINT*: Interrupts need to be globaly enabled.
    /// # Example
    /// ```no_run
    /// # use ruspiro_gpio::*;
    /// # fn doc() {
    /// GPIO.take_for(|gpio| {
    ///     let pin = gpio.get_pin(12).unwrap().into_input();
    ///     let mut counter: u32 = 0;
    ///     gpio.register_recurring_event_handler(
    ///         &pin,
    ///         GpioEvent::RisingEdge,
    ///         move || {
    ///             counter += 1;
    ///             println!("GPIO Event raised {} time(s)", counter);
    ///         }
    ///     );
    /// });
    /// # }
    /// ```
    pub fn register_recurring_event_handler<F: FnMut() + 'static + Send, PUD>(
        &mut self,
        pin: &Pin<function::Input, PUD>,
        event: GpioEvent,
        function: F,
    ) {
        let slot = (pin.num & 31) as usize;
        let bank = pin.num / 32;

        match bank {
            0 => {
                // access to the static array is safe as it happens only in the GPIO which has mutual
                // exclusive access guarentees or inside the interrupt handler which is only active
                // when there is no lock on the GPIO singleton.
                unsafe {
                    BANK0_HANDLER_MC[slot].replace(Box::new(function));
                    // setting multi call clears single call
                    let _ = BANK0_HANDLER_SC[slot].take();
                };
                IRQ_MANAGER.take_for(|irq| irq.activate(Interrupt::GpioBank0));
            }
            1 => {
                // access to the static array is safe as it happens only in the GPIO which has mutual
                // exclusive access guarentees or inside the interrupt handler which is only active
                // when there is no lock on the GPIO singleton.
                unsafe {
                    BANK1_HANDLER_MC[slot].replace(Box::new(function));
                    // setting multi call clears single call
                    let _ = BANK1_HANDLER_SC[slot].take();
                };
                IRQ_MANAGER.take_for(|irq| irq.activate(Interrupt::GpioBank1));
            }
            _ => (),
        };
        activate_detect_event(pin.num, event);
    }

    /// Register an event handler to be executed at the first occurence of the specified event on
    /// the given GPIO [Pin]. The event handler can only be registered for a ``Pin<Input,_>``.
    /// The function/closure provided will be called only once.
    /// **HINT*: Interrupts need to be globaly enabled.
    /// # Example
    /// ```no_run
    /// # use ruspiro_gpio::*;
    /// # fn doc() {
    /// GPIO.take_for(|gpio| {
    ///     let pin = gpio.get_pin(12).unwrap().into_input();
    ///     gpio.register_oneshot_event_handler(
    ///         &pin,
    ///         GpioEvent::RisingEdge,
    ///         move || {
    ///             println!("GPIO Event raised");
    ///         }
    ///     );
    /// });
    /// # }
    /// ```
    pub fn register_oneshot_event_handler<F: FnOnce() + 'static + Send, PUD>(
        &mut self,
        pin: &Pin<function::Input, PUD>,
        event: GpioEvent,
        function: F,
    ) {
        let slot = (pin.num & 31) as usize;
        let bank = pin.num / 32;

        match bank {
            0 => {
                // access to the static array is safe as it happens only in the GPIO which has mutual
                // exclusive access guarentees or inside the interrupt handler which is only active
                // when there is no lock on the GPIO singleton.
                unsafe {
                    BANK0_HANDLER_SC[slot].replace(Box::new(function));
                    // setting single call clears multi call
                    let _ = BANK0_HANDLER_MC[slot].take();
                };
                IRQ_MANAGER.take_for(|irq| irq.activate(Interrupt::GpioBank0));
            }
            1 => {
                // access to the static array is safe as it happens only in the GPIO which has mutual
                // exclusive access guarentees or inside the interrupt handler which is only active
                // when there is no lock on the GPIO singleton.
                unsafe {
                    BANK1_HANDLER_SC[slot].replace(Box::new(function));
                    // setting single call clears multi call
                    let _ = BANK1_HANDLER_MC[slot].take();
                };
                IRQ_MANAGER.take_for(|irq| irq.activate(Interrupt::GpioBank1));
            }
            _ => (),
        };

        activate_detect_event(pin.num, event);
    }

    /// Remove the event handler and deactivate any event detection for the GPIO [Pin] specified.
    /// Removing event handler is only available on a ``Pin<Input,_>``.
    /// # Example
    /// ```no_run
    /// # use ruspiro_gpio::*;
    /// # fn doc() {
    /// GPIO.take_for(|gpio| {
    ///     let pin = gpio.get_pin(12).unwrap().into_input();
    ///     gpio.remove_event_handler(&pin);
    /// });
    /// # }
    /// ```
    pub fn remove_event_handler<PUD>(&mut self, pin: &Pin<function::Input, PUD>) {
        let slot = (pin.num & 31) as usize;
        let bank = pin.num / 32;

        match bank {
            0 => {
                unsafe {
                    let _ = BANK0_HANDLER_SC[slot].take();
                    let _ = BANK0_HANDLER_MC[slot].take();
                };
            }
            1 => {
                unsafe {
                    let _ = BANK1_HANDLER_SC[slot].take();
                    let _ = BANK1_HANDLER_MC[slot].take();
                };
            }
            _ => (),
        };

        deactivate_all_detect_events(pin.num);
    }
}

/// The different GPIO detect events, an event handler can be registered for
pub enum GpioEvent {
    /// Event triggered when the level changes from low to high
    RisingEdge,
    /// Event triggered when the level changes from high to low
    FallingEdge,
    /// Event triggerd when the level changes from low to high or high to low
    BothEdges,
    /// Event riggered as long as the pin level is high
    High,
    /// Event riggered as long as the pin level is low
    Low,
    /// Event triggered when the level changes from low to high, but the detection is not bound
    /// to the GPIO clock rate and allows for faster detections
    AsyncRisingEdge,
    /// Event triggered when the level changes from high to low, but the detection is not bound
    /// to the GPIO clock rate and allows for faster detections
    AsyncFallingEdge,
    /// Event triggered when the level changes from high to low or low to high, but the detection is
    /// not bound to the GPIO clock rate and allows for faster detections
    AsyncBothEdges,
}

/// The error type that will be returned on issues with accessing the GPIO peripheral
pub struct GpioError;

impl core::fmt::Display for GpioError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "An error occured while accessing the GPIO.")
    }
}

impl core::fmt::Debug for GpioError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // debug output the same as display
        <GpioError as core::fmt::Display>::fmt(self, f)
    }
}

/// recurring/multi call interrupt handler for GPIO 0-31 at bank 0
static mut BANK0_HANDLER_MC: [Option<Box<dyn FnMut() + 'static + Send>>; 32] = [None; 32];
/// oneshot/single call interrupt handler for GPIO 0-31 at bank 0
static mut BANK0_HANDLER_SC: [Option<Box<dyn FnOnce() + 'static + Send>>; 32] = [None; 32];

/// recurring/multi callinterrupt handler for GPIO 32-53 at bank 1
static mut BANK1_HANDLER_MC: [Option<Box<dyn FnMut() + 'static + Send>>; 22] = [None; 22];
/// oneshot/single call interrupt handler for GPIO 32-53 at bank 1
static mut BANK1_HANDLER_SC: [Option<Box<dyn FnOnce() + 'static + Send>>; 22] = [None; 22];

/// Implement interrupt handler for GPIO driven interrupts from bank 0 (GPIO 0..31)
/// # Safety
/// As this handler is only called once at a time for the GPIO bank 0 we can safely access the
/// static handler array. The only second place is from within the [Gpio] ``Singleton`` accessor, that when
/// accessed has the interrupts disabled.
#[IrqHandler(GpioBank0)]
fn handle_gpio_bank0() {
    // get the events that raised this interrupt
    let mut trigger_gpios = get_detected_events(GpioBank::Bank0);
    // acknowledge all the events triggered
    acknowledge_detected_events(trigger_gpios, GpioBank::Bank0);

    // for each triggered GPIO pin call the registered handler if any
    let mut pin = 0;
    while trigger_gpios != 0 {
        // take the single call handler if any and call it once
        if let Some(function) = BANK0_HANDLER_SC[pin].take() {
            (function)()
        };
        // if multi call handler is set call it, leaving the handler in place
        if let Some(ref mut function) = &mut BANK0_HANDLER_MC[pin] {
            (function)()
        };
        trigger_gpios >>= 1;
        pin += 1;
    }
}

/// Implement interrupt handler for GPIO driven interrupts from bank 1 (GPIO 32..53)
/// # Safety
/// As this handler is only called once at a time for the GPIO bank 1 we can safely access the
/// static handler array. The only second place is from within the [Gpio] ``Singleton`` accessor, that when
/// accessed has the interrupts disabled.
#[IrqHandler(GpioBank1)]
fn handle_gpio_bank1() {
    // get the events that raised this interrupt
    let mut trigger_gpios = get_detected_events(GpioBank::Bank1);
    // acknowledge all the events triggered
    acknowledge_detected_events(trigger_gpios, GpioBank::Bank1);

    // for each triggered GPIO pin call the registered handler if any
    let mut pin = 0;
    while trigger_gpios != 0 {
        // take the single call handler if any and call it once
        if let Some(function) = BANK1_HANDLER_SC[pin].take() {
            (function)()
        };
        // if multi call handler is set call it, leaving the handler in place
        if let Some(ref mut function) = &mut BANK1_HANDLER_MC[pin] {
            (function)()
        };
        trigger_gpios >>= 1;
        pin += 1;
    }
}

*/
