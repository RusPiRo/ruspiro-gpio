/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Appache License 2.0
 **********************************************************************************************************************/
//! # Raspberry Pi GPIO Pin abstraction
//!

use alloc::boxed::Box;
use ruspiro_register::{ReadOnly, ReadWrite, RegisterField, WriteOnly};
use ruspiro_gpio_hal::*;
use ruspiro_error::*;
use super::interface::*;

pub struct GpioPin {
    id: u32,
    config: PinConfig,
}

impl GpioPin {
    /// create a new Gpio Pin
    pub(crate) fn new(id: u32) -> Self {
        // create a configuration for this pin. This includes the setting of the correct
        // registers to be used for this pin, based on it's id
        // the rgister number used for function selection
        let fsel_register_num = id / 10;
        // thi bit-shift for the value to be set in the function selection register
        let fsel_value_shift = (id % 10) * 3;
        let config = PinConfig {
            fsel_register: match fsel_register_num {
                0 => &GPFSEL0::Register,
                1 => &GPFSEL1::Register,
                2 => &GPFSEL2::Register,
                3 => &GPFSEL3::Register,
                4 => &GPFSEL4::Register,
                5 => &GPFSEL5::Register,
                // as the creation of a pin with an id > 53 is prevented in the Gpio implementation
                // this should never happen that the register value is > 5
                _ => unreachable!(),
            },
            fsel_field: RegisterField::<u32>::new(0x7, fsel_value_shift),
            set_register: if id < 32 { &GPSET0::Register } else { &GPSET1::Register },
            clear_register: if id < 32 { &GPCLR0::Register } else { &GPCLR1::Register },
            level_register: if id < 32 { &GPLEV0::Register } else { &GPLEV1::Register },
            setclr_val: 1 << (id % 32),
            pudclk_register: if id < 32 { &GPPUDCLK0::Register } else { &GPPUDCLK1::Register },
            pud_val: 1 << (id % 32),
        };
        Self { 
            id,
            config,
        }
    }

    /// configure this pin with a specific pull-up/down setting requires a specific flow
    fn configure_pud(&self, pud: Pud) {
        // 1. write the desired pud control value to the PUD control register
        GPPUD::Register.modify(GPPUD::PUD, pud as u32);
        // 2. wait 150 cycles
        for _ in 0..150 {
            unsafe { asm!("NOP") }
        }
        // 3. write the pin to upate into the PUDCLCK register
        self.config.pudclk_register.set(self.config.pud_val);
        // 4. wait 150 cycles to settle the new settings
        for _ in 0..150 {
            unsafe { asm!("NOP") }
        }
        // 5. clear the pud control value in the PUD control register
        GPPUD::Register.set(0x0);
        // 6. write the pin to the PUDCLCK register again to finish the update cycle
        self.config.pudclk_register.set(self.config.pud_val);
    }
}

impl HalGpioPin for GpioPin {
    /// return the id of this [GpioPin]
    fn id(&self) -> u32 {
        self.id
    }

    /// re-configure the [GpioPin] as an Input pin. This is a stateful operation at the hardware layer
    /// so even if the [GpioPin] get's out of scope this setting remains valid
    /// TODO: verify if this is a valid/desired appraoch
    fn into_input(self: Box<Self>) -> Box<dyn HalGpioPinInput> {
        // configure a GpioPin as input requires to configure it's FSEL register
        // accoringly
        self.config
            .fsel_register
            .modify(self.config.fsel_field, Function::Input as u32);

        Box::new(GpioPin { id: self.id, config: self.config })
    }

    /// re-configure the [GpioPin] as an Output pin. This is a stateful operation at the hardware layer
    /// so even if the [GpioPin] get's out of scope this setting remains valid
    /// TODO: verify if this is a valid/desired appraoch
    fn into_output(self: Box<Self>) -> Box<dyn HalGpioPinOutput> {
        // configure a GpioPin as input requires to configure it's FSEL register
        // accoringly
        self.config
            .fsel_register
            .modify(self.config.fsel_field, Function::Output as u32);

        Box::new(GpioPin { id: self.id, config: self.config })
    }

    /// re-configure the [GpioPin] with an alternative function. This is a stateful operation at the hardware layer
    /// so even if the [GpioPin] get's out of scope this setting remains valid.
    /// If a specific hardware dow not support the requested alternative function it shall return an [Err]
    /// TODO: verify if this is a valid/desired appraoch
    fn into_altfunc(self: Box<Self>, function: u8) -> Result<Box<dyn HalGpioPinAltFunc>, BoxError> {
        let alt_func = match function {
            0 => Function::Alt0,
            1 => Function::Alt1,
            2 => Function::Alt2,
            3 => Function::Alt3,
            4 => Function::Alt4,
            5 => Function::Alt5,
            _ => return Err(
                Box::new(
                    GenericError::with_message("Raspberry Pi only supports Alt Funtion 0-5")
                )
            )
        };

        // configure a GpioPin as alternative function x requires to configure it's FSEL register
        // accoringly
        self.config
            .fsel_register
            .modify(self.config.fsel_field, alt_func as u32);

        Ok(
            Box::new(GpioPin { id: self.id, config: self.config })
        )
    }

    /// Diable the pull-up/down settings for this [GpioPin]
    fn disable_pud(&self) {
        self.configure_pud(Pud::Disabled);
    }
    
    /// Enable the pull-up settings for this [GpioPin].
    fn enable_pud_up(&self) {
        self.configure_pud(Pud::PullUp);
    }

    /// Enable the pull-down settings for this [GpioPin]
    fn enable_pud_down(&self) {
        self.configure_pud(Pud::PullDown);
    }
}

impl HalGpioPinInput for GpioPin {
    fn is_high(&self) -> bool {
        let current = self.config.level_register.get() & self.config.setclr_val;
        if current == 0 { false } else { true }
    }
}

impl HalGpioPinOutput for GpioPin {
    fn high(&self) {
        // write 1 for the GPIO Pin bit to the SET register will cause the GPIOPin to
        // produce a high output
        self.config.set_register.set(self.config.setclr_val);
    }

    fn low(&self) {
        // write 1 for the GPIO Pin bit to the CLEAR register will cause the GPIOPin to
        // produce a low output
        self.config.clear_register.set(self.config.setclr_val);
    }

    fn toggle(&self) {
        // toggle the GPIO based on it's current state
        let current = self.config.level_register.get() & self.config.setclr_val;
        if current == 0 { self.high() } else { self.low() }
    }
}
impl HalGpioPinAltFunc for GpioPin {}

//#[derive(Copy, Clone)]
struct PinConfig {
    fsel_register: &'static ReadWrite<u32>,
    fsel_field: RegisterField<u32>,
    set_register: &'static WriteOnly<u32>,
    clear_register: &'static WriteOnly<u32>,
    level_register: &'static ReadOnly<u32>,
    setclr_val: u32,
    pudclk_register: &'static ReadWrite<u32>,
    pud_val: u32,
}
