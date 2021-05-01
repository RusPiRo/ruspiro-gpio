/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Unsafe GPIO access
//!
//! Sometimes during development, when there is no console output possible or atomic operations
//! are not yet usable due to missing MMU setup, it's helpful to verify system behaviour by
//! activating a LED connected to a GPIO of the board. To reduce dependency to any other
//! configuration in this scenario it is helpful to use direct hardware
//! access for the GPIO's accepting the "danger" and the fact it is <b>unsafe</b> to do so
//!

use core::ptr::{read_volatile, write_volatile};
/// Let a LED lit connected to the given GPIO number
///
/// # Safety
/// This access is unsafe as it circumvent all safe constructs available in the `ruspiro-gpio`crate.
#[no_mangle]
pub unsafe fn lit_debug_led(num: u32) {
  let fsel_num = num / 10;
  let fsel_shift = (num % 10) * 3;
  let fsel_addr = 0x3f20_0000 + 4 * fsel_num;
  let set_addr = 0x3f20_001c + num / 32;
  let mut fsel: u32 = read_volatile(fsel_addr as *const u32);
  fsel &= !(7 << fsel_shift);
  fsel |= 1 << fsel_shift;
  write_volatile(fsel_addr as *mut u32, fsel);

  let set: u32 = 1 << (num & 0x1F);
  write_volatile(set_addr as *mut u32, set);
}
