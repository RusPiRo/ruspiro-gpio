/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Appache License 2.0
 **********************************************************************************************************************/

 //! # Raspberry Pi GPIO peripheral
//! 
//! Implementation of the GPIO HAL for Raspberry Pi
//! 

use alloc::{boxed::Box, format};
use core::sync::atomic::{AtomicBool, Ordering};
use ruspiro_gpio_hal::*;
use ruspiro_error::*;
use ruspiro_interrupt::*;
use super::pin::GpioPin;
use super::interface::*;

/// this atomic bool gates the instantiation of the Gpio to happen only once
static GPIO_ONCE: AtomicBool = AtomicBool::new(false);

/// this two atmic bool's gates the access to the static interrupt handler storage
static GPIO_BANK0_ACCESS: AtomicBool = AtomicBool::new(false);
static GPIO_BANK1_ACCESS: AtomicBool = AtomicBool::new(false);

/// The Raspberry Pi representation of the GPIO peripheral device
pub struct Gpio {
    /// for the sake of simplicity we only track which of the max. 40 pins of the Raspberry Pi are already in use
    used_pins: [bool; 54],
}

impl Gpio {
    /// Get a new instance of the Gpio peripheral representation. This should be called only once as it assumes no
    /// [GpioPin] is in use. If this has been called already, it returns an [Err].
    pub fn new() -> Result<Self, ()> {
        if !GPIO_ONCE.compare_and_swap(false, true, Ordering::SeqCst) {
            Ok(
                Self {
                    used_pins: [false; 54],
                }
            )
        } else {
            Err(())
        }
    }
}

/// If the actual Gpio instance used is dropped we would be allowed to create another one
/// so stor this fact inside the atomic bool 
impl Drop for Gpio {
    fn drop(&mut self) {
        GPIO_ONCE.store(false, Ordering::SeqCst);
    }
}

impl HalGpio for Gpio {
    fn use_pin(&mut self, id: u32) -> Result<Box<dyn HalGpioPin>, BoxError> {
        if id > 53 {
            return Err(Box::new(
                GenericError::with_message("Raspberry Pi supports only up to 54 GPIO Pins")
            ));
        }
        
        if self.used_pins[id as usize] {
            Err(Box::new(
                GenericError::with_message(
                    format!("Pin {} already in use", id).as_str()
                ))
            )
        } else {
            Ok(Box::new(GpioPin::new(id)))
        }
    }

    fn release_pin(&mut self, id: u32) -> Result<(), BoxError> {
        if id > 53 {
            return Err(Box::new(
                GenericError::with_message("Raspberry Pi supports only up to 54 GPIO Pins")
            ));
        }
        
        if !self.used_pins[id as usize] {
            Err(Box::new(
                GenericError::with_message(
                    format!("Pin {} was not in use", id).as_str()
                ))
            )
        } else {
            Ok(())
        }
    }

    fn register_event_handler_always(
        &mut self,
        gpio_pin: &dyn HalGpioPinInput,
        event: GpioEvent,
        handler: Box<dyn FnMut() + 'static + Send>,
    ) {
        // get the event handler slot and the GPIO interrupt bank
        let slot = (gpio_pin.id() & 31) as usize;
        let bank = gpio_pin.id() / 32;

        // store the handler in a way the interrupt is able to call them. This requires them stored in a
        // mutable static variable. This is unsafe as it does not guarantie cross core/cross thread exclusive
        // mutable access. We cannot secure this access with a Mutex like construct as this may lead to deadlocks
        // in the interrupt handler. Therefore we secure the access with a non-blocking [AtomicBool]
        match bank {
            0 => if !GPIO_BANK0_ACCESS.compare_and_swap(false, true, Ordering::SeqCst) {
                unsafe {
                    // BANK0 is safe to update, setting an always caller, clears the single caller
                    BANK0_HANDLER_MC[slot].replace(handler);
                    let _ = BANK0_HANDLER_SC[slot].take();
                    // we are done accessing the BANK0
                    GPIO_BANK0_ACCESS.store(false, Ordering::SeqCst);
                }
            },
            1 => if !GPIO_BANK1_ACCESS.compare_and_swap(false, true, Ordering::SeqCst) {
                unsafe {
                    // BANK1 is safe to update, setting an always caller, clears the single caller
                    BANK1_HANDLER_MC[slot].replace(handler);
                    let _ = BANK1_HANDLER_SC[slot].take();
                    // we are done accessing the BANK1
                    GPIO_BANK1_ACCESS.store(false, Ordering::SeqCst);
                }
            },
            _ => unimplemented!(),
        };

        // once the event handler is stored, activate the event we'd like to detect and get handled
        activate_detect_event(gpio_pin.id(), event);
    }

    fn register_event_handler_onetime(
        &mut self,
        gpio_pin: &dyn HalGpioPinInput,
        event: GpioEvent,
        handler: Box<dyn FnOnce() + 'static + Send>,
    ) {
        // get the event handler slot and the GPIO interrupt bank
        let slot = (gpio_pin.id() & 31) as usize;
        let bank = gpio_pin.id() / 32;

        // store the handler in a way the interrupt is able to call them. This requires them stored in a
        // mutable static variable. This is unsafe as it does not guarantie cross core/cross thread exclusive
        // mutable access. We cannot secure this access with a Mutex like construct as this may lead to deadlocks
        // in the interrupt handler. Therefore we secure the access with a non-blocking [AtomicBool]
        match bank {
            0 => if !GPIO_BANK0_ACCESS.compare_and_swap(false, true, Ordering::SeqCst) {
                unsafe {
                    // BANK0 is safe to update, setting a single caller, clears the always caller
                    BANK0_HANDLER_SC[slot].replace(handler);
                    let _ = BANK0_HANDLER_MC[slot].take();
                    // we are done accessing the BANK0
                    GPIO_BANK0_ACCESS.store(false, Ordering::SeqCst);
                }
            },
            1 => if !GPIO_BANK1_ACCESS.compare_and_swap(false, true, Ordering::SeqCst) {
                unsafe {
                    // BANK1 is safe to update, setting a single caller, clears the always caller
                    BANK1_HANDLER_SC[slot].replace(handler);
                    let _ = BANK1_HANDLER_MC[slot].take();
                    // we are done accessing the BANK1
                    GPIO_BANK1_ACCESS.store(false, Ordering::SeqCst);
                }
            },
            _ => unimplemented!(),
        };

        // once the event handler is stored, activate the event we'd like to detect and get handled
        activate_detect_event(gpio_pin.id(), event);
    }

    fn unregister_event_handler(&mut self, gpio_pin: &dyn HalGpioPin, event: GpioEvent) {
        // get the event handler slot and the GPIO interrupt bank
        let slot = (gpio_pin.id() & 31) as usize;
        let bank = gpio_pin.id() / 32;

        // remove the handler in a way the interrupt will no longer call them. This requires them stored in a
        // mutable static variable. This is unsafe as it does not guarantie cross core/cross thread exclusive
        // mutable access. We cannot secure this access with a Mutex like construct as this may lead to deadlocks
        // in the interrupt handler. Therefore we secure the access with a non-blocking [AtomicBool]
        match bank {
            0 => if !GPIO_BANK0_ACCESS.compare_and_swap(false, true, Ordering::SeqCst) {
                unsafe {
                    // BANK0 is safe to update, setting a single caller, clears the always caller
                    let _ = BANK0_HANDLER_SC[slot].take();
                    let _ = BANK0_HANDLER_MC[slot].take();
                    // we are done accessing the BANK0
                    GPIO_BANK0_ACCESS.store(false, Ordering::SeqCst);
                }
            },
            1 => if !GPIO_BANK1_ACCESS.compare_and_swap(false, true, Ordering::SeqCst) {
                unsafe {
                    // BANK1 is safe to update, setting a single caller, clears the always caller
                    let _ = BANK1_HANDLER_SC[slot].take();
                    let _ = BANK1_HANDLER_MC[slot].take();
                    // we are done accessing the BANK1
                    GPIO_BANK1_ACCESS.store(false, Ordering::SeqCst);
                }
            },
            _ => unimplemented!(),
        };

        // once the event handler is removed, deactivate the event we are no longer interested in
        deactivate_detect_event(gpio_pin.id(), event);
    }
}

/// recurring/always call interrupt handler for GPIO 0-31 at bank 0
static mut BANK0_HANDLER_MC: [Option<Box<dyn FnMut() + 'static + Send>>; 32] = [None; 32];
/// oneshot/single call interrupt handler for GPIO 0-31 at bank 0
static mut BANK0_HANDLER_SC: [Option<Box<dyn FnOnce() + 'static + Send>>; 32] = [None; 32];

/// recurring/always call interrupt handler for GPIO 32-53 at bank 1
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
        // handler calls are only made if the guard does not indicate there is actually a different
        // handler about to be configured.
        if !GPIO_BANK0_ACCESS.compare_and_swap(false, true, Ordering::SeqCst) {
            // take the single call handler if any and call it once
            if let Some(function) = BANK0_HANDLER_SC[pin].take() {
                // release the guard before calling the handler
                GPIO_BANK0_ACCESS.store(false, Ordering::SeqCst);
                (function)()
            };
            // if multi call handler is set call it, leaving the handler in place
            if let Some(ref mut function) = &mut BANK0_HANDLER_MC[pin] {
                // release the guard before calling the handler
                GPIO_BANK0_ACCESS.store(false, Ordering::SeqCst);
                (function)()
            };
        }
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
        // handler calls are only made if the guard does not indicate there is actually a different
        // handler about to be configured.
        if !GPIO_BANK1_ACCESS.compare_and_swap(false, true, Ordering::SeqCst) {
            // take the single call handler if any and call it once
            if let Some(function) = BANK1_HANDLER_SC[pin].take() {
                // release the guard before calling the handler
                GPIO_BANK1_ACCESS.store(false, Ordering::SeqCst);
                (function)()
            };
            // if multi call handler is set call it, leaving the handler in place
            if let Some(ref mut function) = &mut BANK1_HANDLER_MC[pin] {
                // release the guard before calling the handler
                GPIO_BANK1_ACCESS.store(false, Ordering::SeqCst);
                (function)()
            };
        }
        trigger_gpios >>= 1;
        pin += 1;
    }
}