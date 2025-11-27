# Redgear-A15

A Linux tool for controlling the Redgear A-15 gaming mouse settings.

> **NOTE: Still in Active Development..**

> **Another Note:**  
> The Redgear A-15 normally uses:
> - **VID:** `0x1BCF`
> - **PID:** `0x08A0`
>
> If your device uses a *different* VID/PID, you can
> identify the correct values by running:
>
> ```bash
> lsusb
> ```
>
> This will output lines like:
>
> ```
> Bus 001 Device 004: ID 1bcf:08a0 USB Optical Mouse
> ```
>
> Here, `1bcf` is the **VID** and `08a0` is the **PID**.
>
> If your device shows different values, update the constants in  
> `src/main.rs` (or wherever VID/PID are defined):
>
> ```rust
> const VID: u16 = 0xXXXX;
> const PID: u16 = 0xYYYY;
> ```
>
> Then rebuild the project:
>
> ```bash
> cargo build --release
> ```

## Disclaimer

This project is an unofficial driver and is **NOT** affiliated with, endorsed by, or connected to Redgear or its parent companies in any way.

## Some Notes

During the analysis of the official Windows driver, I saw the USB control packets sent for `moving_speed`, `double_click_speed`, and `rolling_speed` appeared identical regardless of the value selected in the UI.

This strongly suggests that the official driver may not be sending any actual instructions to the hardware for these settings at all, but instead relying on something else.. or it is entirely NOT possible to change these values. As a result, these specific features are currently marked as `todo!()`. And also the same for `Game Mode` and `Office Mode` (In these two cases, the software is NOT sending any packets to the hardware).

I will update the code once I find any solution to this. 

## Features

Control all aspects of your Redgear A-15 mouse directly from the terminal:

- **DPI Configuration**: Set DPI levels from 1000 to 8000 (8 preset levels)
- **LED Control**: Customize RGB lighting modes, brightness, and breathing effects
- **Fire Control**: Configure auto-fire settings with repeat count and firing intervals
- **Mouse Speed Settings**: Adjust movement speed, double-click speed, and scroll speed
- **Settings Reset**: Restore all settings to factory defaults

## Requirements

- Linux operating system
- USB access to the Redgear A-15 mouse
- Rust toolchain (for building from source)

## Installation

```bash
# Clone the repository
git clone https://github.com/vamsi200/Redgear-A15
cd Redgear-A15/

# Build the project
cargo build --release

# The compiled binary will be available at:
#   target/release/redgear-a15
```

## Usage

### DPI Settings

```bash
# Set DPI level
redgear-a15 dpi dpi3  # Sets DPI to 2400

# Available DPI levels:
# dpi1: 1000  | dpi2: 1600  | dpi3: 2400  | dpi4: 3200
# dpi5: 4800  | dpi6: 6400  | dpi7: 7200  | dpi8: 8000
```

### LED Configuration

```bash
# Set LED mode
redgear-a15 led <mode>

# Set LED brightness
redgear-a15 --led-brightness All

# Set breathing speed (1-8, higher = faster)
redgear-a15 --breathing-speed BS4

# Enable/disable LED
redgear-a15 led-status <on|off>
```

### Fire Control (Macro)

```bash
# Configure auto-fire with repeat count (0-255, default: 3)
redgear-a15 -r 5

# Set firing interval/delay (0-255, default: 6)
redgear-a15 -f 10

# Enable continuous firing
redgear-a15 --continously <enable|disable>
```


### Reset to Defaults

```bash
# Reset all mouse settings to factory defaults
redgear-a15 reset
```

#### Use --help for more details

```bash
Control Redgear A-15 mouse

Usage: regdear-a15 [OPTIONS] [COMMAND]

Commands:
  dpi         Set DPI level
  led         Set LED lighting mode
  led-status  Enable or disable LED lights
  reset       Reset all mouse settings to their default values
  help        Print this message or the help of the given subcommand(s)

Options:
  -r, --repeat <REPEAT>
          Auto-fire repeat count (0–255). Default: 3
  -f, --firing-interval <FIRING_INTERVAL>
          Delay between shots (0–255). Default: 6
      --continously <CONTINOUSLY>
          Enable/disable continuous firing. Default: Disable [possible values: enable, disable]
  -m, --moving-speed <MOVING_SPEED>
          Mouse movement speed (0–255). Default: 6
  -d, --double-click-speed <DOUBLE_CLICK_SPEED>
          Double-click speed (0–255). Default: 7
      --rolling-speed <ROLLING_SPEED>
          Mouse scroll/rolling speed (0–255). Default: 3
      --led-brightness <LED_BRIGHTNESS>
          LED brightness (All/Half). Default: All [possible values: all, half]
      --breathing-speed <BREATHING_SPEED>
          Breathing speed (1–8, higher = faster). Default: BS4 [possible values: bs1, bs2, bs3, bs4, bs5, bs6, bs7, bs8]
  -h, --help
          Print help
  -V, --version
          Print version
```

## Technical Details

This tool communicates directly with the Redgear A-15 mouse via USB HID protocol.

## License

This Project is Licensed under [MIT](https://github.com/vamsi200/Redgear-A15/blob/main/LICENSE)
