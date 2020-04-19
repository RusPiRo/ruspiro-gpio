/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Appache License 2.0
 **********************************************************************************************************************/
//! # Raspberry Pi GPIO Pin abstraction
//!

use crate::hal;
use crate::alloc::boxed::Box;
use crate::{BoxError, GenericError};
use ruspiro_register::{ReadOnly, ReadWrite, RegisterField, WriteOnly};

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

impl hal::GpioPin for GpioPin {
    /// return the id of this [GpioPin]
    fn id(&self) -> u32 {
        self.id
    }

    /// re-configure the [GpioPin] as an Input pin. This is a stateful operation at the hardware layer
    /// so even if the [GpioPin] get's out of scope this setting remains valid
    /// TODO: verify if this is a valid/desired appraoch
    fn into_input(self) -> Box<dyn hal::GpioPinInput> {
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
    fn into_output(self) -> Box<dyn hal::GpioPinOutput> {
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
    fn into_altfunc(self, function: u8) -> Result<Box<dyn hal::GpioPinAltFunc>, BoxError> {
        let alt_func = match function {
            0 => Function::Alt0,
            1 => Function::Alt1,
            2 => Function::Alt2,
            3 => Function::Alt3,
            4 => Function::Alt4,
            5 => Function::Alt5,
            _ => return Err(GenericError::with_message("Raspberry Pi only supports Alt Funtion 0-5").into())
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

impl hal::GpioPinInput for GpioPin {
    fn is_high(&self) -> bool {
        let current = self.config.level_register.get() & self.config.setclr_val;
        if current == 0 { false } else { true }
    }
}

impl hal::GpioPinOutput for GpioPin {
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
impl hal::GpioPinAltFunc for GpioPin {}

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

/*
use ruspiro_register::{ReadOnly, ReadWrite, RegisterField, WriteOnly};
use crate::interface::*;

/// Representation of a GPIO pin that can have specific features. Those features are described with generic arguments to
/// define the pin e.g. as an output pin with disabled PullUp/Down.
#[allow(dead_code)]
pub struct Pin<FUNCTION, PUD> {
    pub(crate) num: u32,
    config: PinConfig,

    #[allow(dead_code)]
    function: FUNCTION,
    pud: PUD,
}

/// Type states for the FUNCTION generic argument of the pin.
pub(crate) mod function {
    pub struct Input;
    pub struct Output;
    pub struct AltFunc0;
    pub struct AltFunc1;
    pub struct AltFunc3;
    pub struct AltFunc2;
    pub struct AltFunc4;
    pub struct AltFunc5;
    pub struct Unknown;
}

/// Type states for the PUD template argument of the pin
pub(crate) mod pud {
    pub struct PullDown;
    pub struct PullUp;
    pub struct Disabled;
    pub struct Unknown;
}

/// Functions available for any kind of pin
impl<FUNC, PUD> Pin<FUNC, PUD> {
    /// Create a new ``Pin`` with an unknown function and PUD settings.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(num: u32) -> Pin<function::Unknown, pud::Unknown> {
        let fsel_num = num / 10;
        let fsel_shift = (num % 10) * 3;
        Pin {
            num,
            config: PinConfig {
                fsel: match fsel_num {
                    0 => GPFSEL0::Register,
                    1 => GPFSEL1::Register,
                    2 => GPFSEL2::Register,
                    3 => GPFSEL3::Register,
                    4 => GPFSEL4::Register,
                    5 => GPFSEL5::Register,
                    _ => panic!("no GPFSEL for the pin {}", num),
                },
                fsel_field: RegisterField::<u32>::new(0x7, fsel_shift),
                set: if num < 32 {
                    GPSET0::Register
                } else {
                    GPSET1::Register
                },
                clear: if num < 32 {
                    GPCLR0::Register
                } else {
                    GPCLR1::Register
                },
                level: if num < 32 {
                    GPLEV0::Register
                } else {
                    GPLEV1::Register
                },
                setclr_val: 1 << (num % 32),
                pudclk: if num < 32 {
                    GPPUDCLK0::Register
                } else {
                    GPPUDCLK1::Register
                },
                pud_val: 1 << (num % 32),
            },
            function: function::Unknown,
            pud: pud::Unknown,
        }
    }

    /// switch any pin into an input pin
    pub fn into_input(self) -> Pin<function::Input, PUD> {
        self.config
            .fsel
            .modify(self.config.fsel_field, Function::Input as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::Input,
            pud: self.pud,
        }
    }

    /// switch any pin into an output pin
    pub fn into_output(self) -> Pin<function::Output, PUD> {
        self.config
            .fsel
            .modify(self.config.fsel_field, Function::Output as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::Output,
            pud: self.pud,
        }
    }

    /// switch any pin into a pin with active alt function 0
    pub fn into_alt_f0(self) -> Pin<function::AltFunc0, PUD> {
        self.config
            .fsel
            .modify(self.config.fsel_field, Function::Alt0 as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::AltFunc0,
            pud: self.pud,
        }
    }

    /// switch any pin into a pin with active alt function 0
    pub fn into_alt_f1(self) -> Pin<function::AltFunc1, PUD> {
        self.config
            .fsel
            .modify(self.config.fsel_field, Function::Alt1 as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::AltFunc1,
            pud: self.pud,
        }
    }

    /// switch any pin into a pin with active alt function 0
    pub fn into_alt_f2(self) -> Pin<function::AltFunc2, PUD> {
        self.config
            .fsel
            .modify(self.config.fsel_field, Function::Alt2 as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::AltFunc2,
            pud: self.pud,
        }
    }

    /// switch any pin into a pin with active alt function 0
    pub fn into_alt_f3(self) -> Pin<function::AltFunc3, PUD> {
        self.config
            .fsel
            .modify(self.config.fsel_field, Function::Alt3 as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::AltFunc3,
            pud: self.pud,
        }
    }

    /// switch any pin into a pin with active alt function 0
    pub fn into_alt_f4(self) -> Pin<function::AltFunc4, PUD> {
        self.config
            .fsel
            .modify(self.config.fsel_field, Function::Alt4 as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::AltFunc4,
            pud: self.pud,
        }
    }

    /// switch any pin into a pin with active alt function 0
    pub fn into_alt_f5(self) -> Pin<function::AltFunc5, PUD> {
        self.config
            .fsel
            .modify(self.config.fsel_field, Function::Alt5 as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::AltFunc5,
            pud: self.pud,
        }
    }

    /// Disable PullUp/Down for the pin
    pub fn into_pud_disabled(self) -> Pin<FUNC, pud::Disabled> {
        self.set_pud(Pud::Disabled);

        Pin {
            num: self.num,
            config: self.config,
            function: self.function,
            pud: pud::Disabled,
        }
    }

    /// Enable PullUp for the pin
    pub fn into_pud_up(self) -> Pin<FUNC, pud::PullUp> {
        self.set_pud(Pud::PullUp);

        Pin {
            num: self.num,
            config: self.config,
            function: self.function,
            pud: pud::PullUp,
        }
    }

    /// Enable PullDown for the pin
    pub fn into_pud_down(self) -> Pin<FUNC, pud::PullDown> {
        self.set_pud(Pud::PullDown);

        Pin {
            num: self.num,
            config: self.config,
            function: self.function,
            pud: pud::PullDown,
        }
    }

    fn set_pud(&self, pud: Pud) {
        // do a pud change cycle:
        // 1. write the desired pud control value to the PUD control register
        GPPUD::Register.modify(GPPUD::PUD, pud as u32);
        // 2. wait 150 cycles
        for _ in 0..150 {
            unsafe { asm!("NOP") }
        }
        // 3. write the pin to upate into the PUDCLCK register
        self.config.pudclk.set(self.config.pud_val);
        // 4. wait 150 cycles to settle the new settings
        for _ in 0..150 {
            unsafe { asm!("NOP") }
        }
        // 5. clear the pud control value in the PUD control register
        GPPUD::Register.set(0x0);
        // 6. write the pin to the PUDCLCK register again to finish the update cycle
        self.config.pudclk.set(self.config.pud_val);
    }
}

/// Functions available only for an Output pin with any PUD setting
impl<PUD> Pin<function::Output, PUD> {
    pub fn high(&self) {
        // write the pin bit to the set register to set the pin to high
        self.config.set.set(self.config.setclr_val);
    }

    pub fn low(&self) {
        // write the pin bit to the clear register to set the pin to low
        self.config.clear.set(self.config.setclr_val);
    }

    pub fn toggle(&self) {
        // get the current level of the pin and toggle it's state
        if (self.config.level.get() & self.config.setclr_val) == 0 {
            self.high();
        } else {
            self.low();
        }
    }
}

#[derive(Clone)]
struct PinConfig {
    pub(crate) fsel: ReadWrite<u32>,
    pub(crate) fsel_field: RegisterField<u32>,
    pub(crate) set: WriteOnly<u32>,
    pub(crate) clear: WriteOnly<u32>,
    pub(crate) level: ReadOnly<u32>,
    pub(crate) setclr_val: u32,
    pub(crate) pudclk: ReadWrite<u32>,
    pub(crate) pud_val: u32,
}

*/
