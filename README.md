# RusPiRo GPIO access abstraction for Raspberry Pi

This crate provide as simple to use and safe abstraction of the GPIO's available on the Raspberry Pi 3.

## Usage
To use the crate just add the following dependency to your ``Cargo.toml`` file:
```
[dependencies]
ruspiro-gpio = { git = "https://github.com/RusPiRo/ruspiro-gpio", tag = "v0.0.1" }
```

Once done the access to the GPIO abstraction is available in your rust files like so:
```
use ruspiro_gpio::GPIO;

fn demo() {
    // "grab" the GPIO in a safe way and use the provided closure to work with it
    // as long as the closure is executed, no other core can access the GPIO to configure pins etc.
    GPIO.take_for(|gpio| {
        // retrieving a pin gives a Result<>. If the pin is not already taken it returns an Ok()
        // with the pin.
        if let Ok(pin) = gpio.get_pin(17) {
            // as we have now access to the pin, configure it as output and set it to high
            // to lit a connected LED
            pin.to_output().high();
        }
    })
}
```


## License
Licensed under Apache License, Version 2.0, ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)