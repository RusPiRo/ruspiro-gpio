/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Appache License 2.0
 **********************************************************************************************************************/

//! # Title goes here
//! Implementation of a GPIO Pin and its functions. The purpose and current state of each pin is encapsulated with a
//! zero-sizes-type generics argument to ensure compile time safety when using a pin that has specific requirements
//! 
use ruspiro_register::{define_registers, ReadWrite, WriteOnly, RegisterField};
//use rubots_lib::timer::sleepcycles;

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
    pub fn new(num: u32) -> Pin<function::Unknown, pud::Unknown> {
        let fsel_num = num / 10;
        let fsel_shift = (num % 10) * 3;
        Pin {
            num: num,
            config: PinConfig {
                fsel: match fsel_num {
                    0 => GPFSEL0::Register,
                    1 => GPFSEL1::Register,
                    2 => GPFSEL2::Register,
                    3 => GPFSEL3::Register,
                    4 => GPFSEL4::Register,
                    5 => GPFSEL5::Register,
                    _ => panic!("no GPFSEL for the pin {}", num)
                },
                fsel_field: RegisterField::<u32>::new(0x7, fsel_shift),
                set: if num < 32 { GPSET0::Register } else { GPSET1::Register },
                clear: if num < 32 { GPCLR0::Register } else { GPCLR1::Register },
                setclr_val: 1 << (num % 32),
                pudclk: if num < 32 { GPPUDCLK0::Register } else { GPPUDCLK1::Register },
                pud_val: 1 << (num % 32),
            },
            function: function::Unknown,
            pud: pud::Unknown
        }
    }

    /// switch any pin into an input pin
    pub fn to_input(self) -> Pin<function::Input, PUD> {
        self.config.fsel.modify(self.config.fsel_field, Function::Input as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::Input,
            pud: self.pud,
        }
    }

    /// switch any pin into an output pin
    pub fn to_output(self) -> Pin<function::Output, PUD> {
        self.config.fsel.modify(self.config.fsel_field, Function::Output as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::Output,
            pud: self.pud,
        }
    }

    /// switch any pin into an pin with active alt function 0
    pub fn to_alt_f0(self) -> Pin<function::AltFunc0, PUD> {
        self.config.fsel.modify(self.config.fsel_field, Function::Alt0 as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::AltFunc0,
            pud: self.pud,
        }
    }

    /// switch any pin into an pin with active alt function 0
    pub fn to_alt_f1(self) -> Pin<function::AltFunc1, PUD> {
        self.config.fsel.modify(self.config.fsel_field, Function::Alt1 as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::AltFunc1,
            pud: self.pud,
        }
    }

    /// switch any pin into an pin with active alt function 0
    pub fn to_alt_f2(self) -> Pin<function::AltFunc2, PUD> {
        self.config.fsel.modify(self.config.fsel_field, Function::Alt2 as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::AltFunc2,
            pud: self.pud,
        }
    }

    /// switch any pin into an pin with active alt function 0
    pub fn to_alt_f3(self) -> Pin<function::AltFunc3, PUD> {
        self.config.fsel.modify(self.config.fsel_field, Function::Alt3 as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::AltFunc3,
            pud: self.pud,
        }
    }

    /// switch any pin into an pin with active alt function 0
    pub fn to_alt_f4(self) -> Pin<function::AltFunc4, PUD> {
        self.config.fsel.modify(self.config.fsel_field, Function::Alt4 as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::AltFunc4,
            pud: self.pud,
        }
    }

    /// switch any pin into an pin with active alt function 0
    pub fn to_alt_f5(self) -> Pin<function::AltFunc5, PUD> {
        self.config.fsel.modify(self.config.fsel_field, Function::Alt5 as u32);
        Pin {
            num: self.num,
            config: self.config,
            function: function::AltFunc5,
            pud: self.pud,
        }
    }

    /// Disable PullUp/Down for the pin
    pub fn to_pud_disabled(self) -> Pin<FUNC, pud::Disabled> {
        self.set_pud(Pud::Disabled);

        Pin {
            num: self.num,
            config: self.config,
            function: self.function,
            pud: pud::Disabled,
        }
    }

    /// Enable PullUp for the pin
    pub fn to_pud_up(self) -> Pin<FUNC, pud::PullUp> {
        self.set_pud(Pud::PullUp);

        Pin {
            num: self.num,
            config: self.config,
            function: self.function,
            pud: pud::PullUp,
        }
    }

    /// Enable PullDown for the pin
    pub fn to_pud_down(self) -> Pin<FUNC, pud::PullDown> {
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
        for _ in 0..150 { unsafe { asm!("NOP") } };
        // 3. write the pin to upate into the PUDCLCK register
        self.config.pudclk.set(self.config.pud_val);
        // 4. wait 150 cycles to settle the new settings
        for _ in 0..150 { unsafe { asm!("NOP") } };
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
}


// Define the registers of the GPIO that are used to access the pin's
// the GPIO register base address is provided in the super module
use super::GPIO_BASE as GPIO_BASE;

define_registers! [
    GPFSEL0: ReadWrite<u32> @ GPIO_BASE + 0x00 => [],
    GPFSEL1: ReadWrite<u32> @ GPIO_BASE + 0x04 => [],
    GPFSEL2: ReadWrite<u32> @ GPIO_BASE + 0x08 => [],
    GPFSEL3: ReadWrite<u32> @ GPIO_BASE + 0x0C => [],
    GPFSEL4: ReadWrite<u32> @ GPIO_BASE + 0x10 => [],
    GPFSEL5: ReadWrite<u32> @ GPIO_BASE + 0x14 => [],
    GPSET0: WriteOnly<u32> @ GPIO_BASE + 0x1C => [],
    GPSET1: WriteOnly<u32> @ GPIO_BASE + 0x20 => [],
    GPCLR0: WriteOnly<u32> @ GPIO_BASE + 0x28 => [],
    GPCLR1: WriteOnly<u32> @ GPIO_BASE + 0x2C => [],
    GPPUD: ReadWrite<u32> @ GPIO_BASE + 0x94 => [
        PUD OFFSET(0) BITS(2)
    ],
    GPPUDCLK0: ReadWrite<u32> @ GPIO_BASE + 0x98 => [],
    GPPUDCLK1: ReadWrite<u32> @ GPIO_BASE + 0x9C => []
];

// GPIO pin function register config values
#[repr(u32)]
enum Function {
    Input  = 0b000,
    Output = 0b001,
    Alt0   = 0b100,
    Alt1   = 0b101,
    Alt2   = 0b110,
    Alt3   = 0b111,
    Alt4   = 0b011,
    Alt5   = 0b010,
}

// GPIO pull up/down register config values
#[repr(u8)]
enum Pud {
    Disabled = 0b00,
    PullDown = 0b01,
    PullUp   = 0b10
}

#[derive(Clone)]
struct PinConfig {
    pub fsel: ReadWrite<u32>,
    pub fsel_field: RegisterField<u32>,
    pub set: WriteOnly<u32>,
    pub clear: WriteOnly<u32>,
    pub setclr_val: u32,
    pub pudclk: ReadWrite<u32>,
    pub pud_val: u32,
}