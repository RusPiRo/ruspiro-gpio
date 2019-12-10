/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Appache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-gpio/0.3.0")]
#![no_std]
#![feature(asm)]
//! # Raspberry Pi GPIO access abstraction
//! 
//! This crate provide as simple to use and safe abstraction of the GPIO's available on the Raspberry Pi 3. The GPIO 
//! configuration requires access to MMIO registers with a specific memory base address. As this might differ between
//! different models the right address is choosen based on the given ``ruspiro_pi3`` feature while compiling.
//! 
//! # Usage
//! 
//! The crate provides a singleton accessor to the GPIO peripheral and it's pin to be used in a safe manner like this:
//! ```
//! use ruspiro_gpio::GPIO;
//! 
//! # fn main() {
//!     GPIO.take_for(|gpio| {
//!         let pin = gpio.get_pin(17).unwrap(); // assuming we can always get this pin as it is not in use already
//!         pin.to_output().high(); // set this pin to high - this may lit a connected LED :)
//!     });
//! # }
//! ```
//! 
//! # Features
//! 
//! - ``ruspiro_pi3`` is active by default and ensures the proper MMIO base memory address is used for Raspberry Pi 3
//! 

use ruspiro_singleton::Singleton;

pub mod pin;
pub use self::pin::*;

pub mod debug;

/// Static "singleton" accessor to the GPIO peripheral
pub static GPIO: Singleton<Gpio> = Singleton::<Gpio>::new(Gpio::new());

/// GPIO peripheral representation
pub struct Gpio {
    used_pins: [bool;40],
}

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
    pub fn get_pin(&mut self, num: u32) -> Result<Pin<function::Unknown, pud::Unknown>, &'static str> {
        if self.used_pins[num as usize] {
            Err("requested pin already in use")
        } else {
            self.used_pins[num as usize] = true;
            Ok(Pin::<function::Unknown, pud::Unknown>::new(num))
        }
    }

    pub fn free_pin(&mut self, num: u32) {
        // release the used pin
        // TODO: reset also pin function or other settings?
        if self.used_pins[num as usize] {
            self.used_pins[num as usize] = false;
        };
    }
}