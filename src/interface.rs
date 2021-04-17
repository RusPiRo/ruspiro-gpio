/***************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **************************************************************************************************/

//! # Gpio low level functions
//!

use crate::GpioEvent;
use ruspiro_mmio_register::*;

// MMIO peripheral base address based on the pi model we build for
#[cfg(any(feature = "ruspiro_pi3", feature = "ruspiro_pi3_test"))]
const PERIPHERAL_BASE: usize = 0x3F00_0000;

/// Base address for GPIO MMIO registers
const GPIO_BASE: usize = PERIPHERAL_BASE + 0x0020_0000;

/// The two existing GPIO banks
pub(crate) enum GpioBank {
    Bank0,
    Bank1,
}

// GPIO pin function register config values
#[repr(u32)]
pub(crate) enum Function {
    Input = 0b000,
    Output = 0b001,
    Alt0 = 0b100,
    Alt1 = 0b101,
    Alt2 = 0b110,
    Alt3 = 0b111,
    Alt4 = 0b011,
    Alt5 = 0b010,
}

// GPIO pull up/down register config values
#[repr(u8)]
pub(crate) enum Pud {
    Disabled = 0b00,
    PullDown = 0b01,
    PullUp = 0b10,
}

/// Activate the event detection for a specific gpio pin
pub(crate) fn activate_detect_event(pin: u32, event: GpioEvent) {
    let slot = pin & 31;
    let event_field = RegisterField::<u32>::new(1, slot);
    match pin / 32 {
        0 => {
            match event {
                GpioEvent::RisingEdge => GPREN0::Register.modify(event_field, 1),
                GpioEvent::FallingEdge => GPFEN0::Register.modify(event_field, 1),
                GpioEvent::BothEdges => {
                    GPREN0::Register.modify(event_field, 1);
                    GPFEN0::Register.modify(event_field, 1)
                }
                GpioEvent::High => GPHEN0::Register.modify(event_field, 1),
                GpioEvent::Low => GPLEN0::Register.modify(event_field, 1),
                GpioEvent::AsyncRisingEdge => GPAREN0::Register.modify(event_field, 1),
                GpioEvent::AsyncFallingEdge => GPAFEN0::Register.modify(event_field, 1),
                GpioEvent::AsyncBothEdges => {
                    GPAREN0::Register.modify(event_field, 1);
                    GPAFEN0::Register.modify(event_field, 1)
                }
            };
        }
        1 => {
            match event {
                GpioEvent::RisingEdge => GPREN1::Register.modify(event_field, 1),
                GpioEvent::FallingEdge => GPFEN1::Register.modify(event_field, 1),
                GpioEvent::BothEdges => {
                    GPREN1::Register.modify(event_field, 1);
                    GPFEN1::Register.modify(event_field, 1)
                }
                GpioEvent::High => GPHEN1::Register.modify(event_field, 1),
                GpioEvent::Low => GPLEN1::Register.modify(event_field, 1),
                GpioEvent::AsyncRisingEdge => GPAREN1::Register.modify(event_field, 1),
                GpioEvent::AsyncFallingEdge => GPAFEN1::Register.modify(event_field, 1),
                GpioEvent::AsyncBothEdges => {
                    GPAREN1::Register.modify(event_field, 1);
                    GPAFEN1::Register.modify(event_field, 1)
                }
            };
        }
        _ => (),
    }
}

/// De-activate the event detection for a specific gpio pin
#[allow(dead_code)]
pub(crate) fn deactivate_detect_event(pin: u32, event: GpioEvent) {
    let slot = pin & 31;
    let event_field = RegisterField::<u32>::new(1, slot);
    match pin / 32 {
        0 => {
            match event {
                GpioEvent::RisingEdge => GPREN0::Register.modify(event_field, 0),
                GpioEvent::FallingEdge => GPFEN0::Register.modify(event_field, 0),
                GpioEvent::BothEdges => {
                    GPREN0::Register.modify(event_field, 0);
                    GPFEN0::Register.modify(event_field, 0)
                }
                GpioEvent::High => GPHEN0::Register.modify(event_field, 0),
                GpioEvent::Low => GPLEN0::Register.modify(event_field, 0),
                GpioEvent::AsyncRisingEdge => GPAREN0::Register.modify(event_field, 0),
                GpioEvent::AsyncFallingEdge => GPAFEN0::Register.modify(event_field, 0),
                GpioEvent::AsyncBothEdges => {
                    GPAREN0::Register.modify(event_field, 0);
                    GPAFEN0::Register.modify(event_field, 0)
                }
            };
        }
        1 => {
            match event {
                GpioEvent::RisingEdge => GPREN1::Register.modify(event_field, 0),
                GpioEvent::FallingEdge => GPFEN1::Register.modify(event_field, 0),
                GpioEvent::BothEdges => {
                    GPREN1::Register.modify(event_field, 0);
                    GPFEN1::Register.modify(event_field, 0)
                }
                GpioEvent::High => GPHEN1::Register.modify(event_field, 0),
                GpioEvent::Low => GPLEN1::Register.modify(event_field, 0),
                GpioEvent::AsyncRisingEdge => GPAREN1::Register.modify(event_field, 0),
                GpioEvent::AsyncFallingEdge => GPAFEN1::Register.modify(event_field, 0),
                GpioEvent::AsyncBothEdges => {
                    GPAREN1::Register.modify(event_field, 0);
                    GPAFEN1::Register.modify(event_field, 0)
                }
            };
        }
        _ => (),
    }
}

/// De-activate all events detection for a specific gpio pin
pub(crate) fn deactivate_all_detect_events(pin: u32) {
    let slot = pin & 31;
    let event_field = RegisterField::<u32>::new(1, slot);
    match pin / 32 {
        0 => {
            GPREN0::Register.modify(event_field, 0);
            GPFEN0::Register.modify(event_field, 0);
            GPHEN0::Register.modify(event_field, 0);
            GPLEN0::Register.modify(event_field, 0);
            GPAREN0::Register.modify(event_field, 0);
            GPAFEN0::Register.modify(event_field, 0);
        }
        1 => {
            GPREN1::Register.modify(event_field, 0);
            GPFEN1::Register.modify(event_field, 0);
            GPHEN1::Register.modify(event_field, 0);
            GPLEN1::Register.modify(event_field, 0);
            GPAREN1::Register.modify(event_field, 0);
            GPAFEN1::Register.modify(event_field, 0);
        }
        _ => (),
    }
}

/// Read the event detect status register for the specified bank
pub(crate) fn get_detected_events(bank: GpioBank) -> u32 {
    match bank {
        GpioBank::Bank0 => GPEDS0::Register.get(),
        GpioBank::Bank1 => GPEDS1::Register.get(),
    }
}

/// Reset the event detect status register for the specified bank to acknowledge the
/// event within the interrupt handler
pub(crate) fn acknowledge_detected_events(events: u32, bank: GpioBank) {
    match bank {
        GpioBank::Bank0 => GPEDS0::Register.set(events),
        GpioBank::Bank1 => GPEDS1::Register.set(events),
    }
}

// Define the registers of the GPIO that are used to access the pin's
define_mmio_register! [
    /// Alt-Function select register for pin 0..9
    pub(crate) GPFSEL0<ReadWrite<u32>@(GPIO_BASE)>,
    /// Alt-Function select register for pin 10..19
    pub(crate) GPFSEL1<ReadWrite<u32>@(GPIO_BASE + 0x04)>,
    /// Alt-Function select register for pin 20..29
    pub(crate) GPFSEL2<ReadWrite<u32>@(GPIO_BASE + 0x08)>,
    /// Alt-Function select register for pin 30..39
    pub(crate) GPFSEL3<ReadWrite<u32>@(GPIO_BASE + 0x0C)>,
    /// Alt-Function select register for pin 40..49
    pub(crate) GPFSEL4<ReadWrite<u32>@(GPIO_BASE + 0x10)>,
    /// Alt-Function select register for pin 50..53
    pub(crate) GPFSEL5<ReadWrite<u32>@(GPIO_BASE + 0x14)>,
    /// Output Pin set register for pin 0..31
    pub(crate) GPSET0<WriteOnly<u32>@(GPIO_BASE + 0x1C)>,
    /// Output Pin set register for pin 32..53
    pub(crate) GPSET1<WriteOnly<u32>@(GPIO_BASE + 0x20)>,
    /// Output Pin clear register for pin 0..31
    pub(crate) GPCLR0<WriteOnly<u32>@(GPIO_BASE + 0x28)>,
    /// Output Pin clear register for pin 32..53
    pub(crate) GPCLR1<WriteOnly<u32>@(GPIO_BASE + 0x2C)>,
    /// Read Pin level register for pin 0..31
    pub(crate) GPLEV0<ReadOnly<u32>@(GPIO_BASE + 0x34)>,
    /// Read Pin level register for pin 32..53
    pub(crate) GPLEV1<ReadOnly<u32>@(GPIO_BASE + 0x38)>,
    /// Pull-Up/Down configuration register
    pub(crate) GPPUD<ReadWrite<u32>@(GPIO_BASE + 0x94)> {
        PUD OFFSET(0) BITS(2)
    },
    /// Pull-Up/Down clock register for pin 0..31
    pub(crate) GPPUDCLK0<ReadWrite<u32>@(GPIO_BASE + 0x98)>,
    /// Pull-Up/Down clock register for pin 32..53
    pub(crate) GPPUDCLK1<ReadWrite<u32>@(GPIO_BASE + 0x9C)>,
    /// GPIO Pin event detect status bank 0 (pin 0..31)
    GPEDS0<ReadWrite<u32>@(GPIO_BASE + 0x40)>,
    /// GPIO Pin event detect status bank 1 (pin 32..53)
    GPEDS1<ReadWrite<u32>@(GPIO_BASE + 0x44)>,
    /// GPIO Pin rising edge detect enable bank 0 (pin 0..31)
    GPREN0<ReadWrite<u32>@(GPIO_BASE + 0x4c)>,
    /// GPIO Pin rising edge detect enable bank 1 (pin 32..53)
    GPREN1<ReadWrite<u32>@(GPIO_BASE + 0x50)>,
    /// GPIO Pin falling edge detect enable bank 0 (pin 0..31)
    GPFEN0<ReadWrite<u32>@(GPIO_BASE + 0x58)>,
    /// GPIO Pin falling edge detect enable bank 1 (pin 32..53)
    GPFEN1<ReadWrite<u32>@(GPIO_BASE + 0x5c)>,
    /// GPIO Pin high detect enable bank 0 (pin 0..31)
    GPHEN0<ReadWrite<u32>@(GPIO_BASE + 0x64)>,
    /// GPIO Pin high detect enable bank 1 (pin 32..53)
    GPHEN1<ReadWrite<u32>@(GPIO_BASE + 0x68)>,
    /// GPIO Pin low detect enable bank 0 (pin 0..31)
    GPLEN0<ReadWrite<u32>@(GPIO_BASE + 0x70)>,
    /// GPIO Pin low detect enable bank 1 (pin 32..53)
    GPLEN1<ReadWrite<u32>@(GPIO_BASE + 0x74)>,
    /// GPIO Pin async rising edge detect enable bank 0 (pin 0..31)
    GPAREN0<ReadWrite<u32>@(GPIO_BASE + 0x7c)>,
    /// GPIO Pin async rising edge detect enable bank 1 (pin 32..53)
    GPAREN1<ReadWrite<u32>@(GPIO_BASE + 0x80)>,
    /// GPIO Pin async falling edge detect enable bank 0 (pin 0..31)
    GPAFEN0<ReadWrite<u32>@(GPIO_BASE + 0x88)>,
    /// GPIO Pin async falling edge detect enable bank 1 (pin 32..53)
    GPAFEN1<ReadWrite<u32>@(GPIO_BASE + 0x8c)>
];
