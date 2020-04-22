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

extern crate alloc;
/// re-export the HAL trait definitions as they need to be in scope when using the specific Gpio implementation
pub use ruspiro_gpio_hal as hal;

mod gpio;
pub use gpio::*;

mod pin;
pub use pin::*;

mod interface;