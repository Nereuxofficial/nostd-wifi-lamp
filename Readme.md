# Nostd-wifi-lamp
[![License](https://img.shields.io/badge/License-AGPLv3-blue?style=flat-square)](#license)
[![issues - Nereuxofficial](https://img.shields.io/github/issues/Nereuxofficial/nostd-wifi-lamp?style=flat-square)](https://github.com/Nereuxofficial/nostd-wifi-lamp/issues)
![Commits/m](https://img.shields.io/github/commit-activity/m/Nereuxofficial/nostd-wifi-lamp?style=flat-square)

A Wi-Fi controllable lamp written in Rust for the ESP32 using esp-hal. This was created for a blog post you can read 
[here](https://nereux.blog/posts/esp32-ws2812-dino-light-2/).
## Usage
Sadly, I cannot provide binaries as they include WI-FI SSID and Passwords.
After [installing espup](https://github.com/esp-rs/espup) follow these steps:

1. Install the `cargo-espflash` tool
```bash
# We need the newest espflash as of now
cargo install espflash --git https://github.com/esp-rs/cargo-espflash.git
```
2. Clone this repository
```bash
git clone https://github.com/Nereuxofficial/nostd-wifi-lamp
cd nostd-wifi-lamp
```
3. Flash it to your ESP32
```bash
export SSID="your SSID" PASSWORD="your password"a
cargo run --release
```
(If the last step doesn't work try pressing the `Boot` Button during the command, which always worked for me.)

## Credits
[bjoernQ](https://github.com/bjoernQ) for fixing an error where the stack overflowed into the heap and i had no idea why it was crashing.
[esp-rs](https://github.com/esp-rs/) for the Rust tooling around ESP32s
