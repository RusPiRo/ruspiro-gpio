/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Appache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-gpio/0.0.1")]
#![no_std]
#![feature(asm)]

//! # Raspberry GPIO access abstraction
//! Implementation of a simple and safe API to access Raspberry Pi3 GPIO's.
//! 
//!
use ruspiro_singleton::Singleton;

pub mod pin;
pub use self::pin::*;

// MMIO peripheral base address need to be provided by user of this crate
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
    /// Returns an Err(()) if the pin is already in use, otherwise an Ok(Pin)
    pub fn get_pin(&mut self, num: u32) -> Result<Pin<function::Unknown, pud::Unknown>, ()> {
        if self.used_pins[num as usize] {
            Err(())
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

    pub fn free_pin2<F, P>(&mut self, pin: Pin<F, P>) {
        if self.used_pins[pin.num as usize] {
            self.used_pins[pin.num as usize] = false;
        }
    }
}