/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Appache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-gpio/0.0.2")]
#![no_std]
#![feature(asm)]
//! # Raspberry Pi GPIO access abstraction
//! Implementation of a simple and safe API to access Raspberry Pi3 GPIO's. The GPIO configuration requires access to
//! MMIO registers with a specific memory base address. As this might differ between different models the right
//! address is choosen based on the given ``target_family`` while compiling. The value needed for a Raspberry Pi 3 is
//! ``ruspiro-pi3``.
//! 
//! # Usage
//! 
//! The crate provides a singleton accessor to the GPIO peripheral and it's pin to be used in a safe manner like this:
//! ```
//! use ruspiro_gpio::GPIO;
//! 
//! fn demo() {
//!     GPIO.take_for(|gpio| {
//!         let pin = gpio.get_pin(17).unwrap(); // assuming we can always get this pin as it is not in use already
//!         pin.to_output().high(); // set this pin to high - this may lit a connected LED :)
//!     });
//! }
//! ```
//! 

use ruspiro_singleton::Singleton;

pub mod pin;
pub use self::pin::*;

// MMIO peripheral base address based on the target family provided with the custom target config file.
#[cfg(target_family="ruspiro-pi3")]
const PERIPHERAL_BASE: u32 = 0x3F00_0000;

/// Base address for GPIO MMIO registers
const GPIO_BASE: u32 = PERIPHERAL_BASE + 0x0020_0000;

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