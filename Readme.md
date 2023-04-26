# Nostd-wifi-lamp
A wifi lamp controllable via curl POST requests
## Usage
Sadly, I cannot provide binaries as they include WI-FI SSID and Passwords.
After [installing Rust](https://rustup.rs) follow these steps
1. Install Rust nightly
```bash
rustup default nightly
```
2. Install the `cargo-espflash` tool
```bash
# We need the newest espflash as of now
cargo install espflash --git https://github.com/esp-rs/cargo-espflash.git
```
3. Clone this repository
```bash
git clone https://github.com/Nereuxofficial/nostd-wifi-lamp
```
4. flash it to your ESP32
```bash
export SSID="your SSID" PASSWORD="your password"a
cargo run --release
```
(If the last step doesn't work try pressing the `Boot` Button during the command, which always worked for me.)

## Credits
[bjoernQ](https://github.com/bjoernQ) for fixing an error where the stack overflowed into the heap and i had no idea why it was crashing.
[esp-rs](https://github.com/esp-rs/) for the Rust tooling around ESP32s