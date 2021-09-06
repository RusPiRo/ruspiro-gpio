# Changelog

## :melon: v0.4.3

This is a maintenance release ensuring succesful build with the latest nightly (2021-09-05) version.

- ### :wrench: Maintenance

  - build the single crate with `aarch-unknown-none` target
  - remove `const_fn` feature
  - change usage of `llvm_asm` feature and macro into `asm`
  
## :peach: v0.4.2

This is mainly a maintenance release. It contains the migration to the github build pipeline and some minor adjustments.

- ### :wrench: Maintenance

  - migrate build pipeline to github actions
  - update versions of dependend crates
  - minor fixes to allow building with new rust nightly version. The version can be found in the `rust-toolchain.toml` file.

## :banana: v0.4.1

- ### :detective: Fixes

  - remove `asm!` macro usages and replace with `llvm_asm!`
  - use `cargo make` to stabilize cross-platform builds

## :pizza: v0.4.0

- ### :bulb: Features

  - New function to ``toggle`` an ``Output`` ``Pin`` between high and low.
  - adding a function to lit a connected LED with direct ``unsafe`` peripheral access. This might
    be helpful to produce debug hints in case there is no console output possible.
  - introduce the ``GpioError`` type for functions that return a ``Result`` in this crate
  - Introducing the possibility to register functions/closures to event detections from a GPIO input pin.
    Those functions/closure will execute in the context of the interrupt handler for those events

- ### :wrench: Maintenance

  - Based on "best practices" the functions for the ``Pin`` that changes their behaviour are renamed
    from ``to_*`` to ``into_*``.
  - the ``ruspiro_pi3`` feature is no longer active by default
  - increase code quality using ``carge fmt`` and ``cargo clippy``
  - move all mmio register definitions into a lowlevel ``interface.rs`` file

- ### :book: Documentation

  - Update documentation on the new functions and the existing ones.
