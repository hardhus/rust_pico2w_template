# Rust Pico 2W Template

A minimal, production-ready asynchronous template for the **Raspberry Pi Pico 2W (RP2350 + CYW43439)**. This template configures the Embassy framework to enable unified global logging over USB via the standard `log` crate macros (`info!`, `warn!`, etc.) and controls the onboard wireless LED without requiring any local `.bin` firmware blobs.

## Features

- **Embassy Async Runtime** – Full event-driven architecture utilizing async/await, eliminating blocking delays (`delay_ms`) to maximize CPU efficiency.
- **Global USB Serial Logging** – Fully integrated with the `log` crate. Use `info!`, `warn!`, `error!` anywhere in your codebase; logs are safely batched into an atomic async queue (`embassy-sync::channel`) and sent straight over the native USB port.
- **Embedded Firmware Injection** – Leverages `cyw43-setup` to dynamically inject the CYW43439 Wi-Fi controller microcode layout directly at startup. No manually downloaded binary blobs are needed in the workspace.
- **Pure Software Reloading** – Configured with a `picotool` runner setup. Dropping the board into **BOOTSEL** mode and typing `cargo run` automatically flashes, verifies, and hot-boots the device over USB.

## Dependencies

### 1. Hardware
- **Raspberry Pi Pico 2W**
- **USB Data Cable** (For flashing the binary and reading the live logs)

### 2. Software
The compilation toolchain requires the official `picotool` binary to flash the chip over USB.

- **Windows:** Download the pre-compiled `picotool.exe` from the official Raspberry Pi GitHub releases, then add it to your system Environment PATH.
- **Linux (Ubuntu/Debian):** `sudo apt install picotool`
- **macOS:** `brew install picotool`

Ensure you have the Rust compiler target layer ready for the Cortex-M33 architecture:
```bash
rustup target add thumbv8m.main-none-eabihf

```

## Quick Start

### 1. Implementation & Deployment

Clone the workspace, compile the static image targets, and flash it down to the target memory layout:

```bash
# Clone the repository
git clone [https://github.com/hardhus/rust_pico_template.git](https://github.com/hardhus/rust_pico_template.git)
cd rust_pico_template

# Hold the BOOTSEL button, plug in the USB cable, then run:
cargo run

```

### 2. Verification

Once `picotool` signals `OK` and the device reboots, run your preferred hardware monitoring engine to read the logs over the USB cable:

```bash
# Windows
serial-monitor.exe

# Linux/macOS equivalents
screen /dev/ttyACM0 115200

```

Expected live execution log:

```text
Connected to COM5
Press Control-X to exit
[INFO] System started, initializing CYW43...
[INFO] CYW43 ready, starting LED blink...
[INFO] LED ON
[INFO] LED OFF

```

## Workspace Layout

```text
.
├── .cargo/
│   └── config.toml      # Architecture matrix target details and picotool execution hook
├── src/
│   ├── main.rs          # System initialization, execution spawner logic, and blinky block
│   ├── logger.rs        # Global UsbLogger implementation backing the log facade
│   └── usb_logger.rs    # Safe static cell-backed USB device initialization stack
├── build.rs             # Output memory allocation and linker script staging script
├── Cargo.toml           # Complete dependency definitions (Embassy git targets + cyw43 crates)
├── memory.x             # Core ARM memory boundary flash definition
└── rp235x_riscv.x       # Secondary fallback RISC-V hazard core linking configuration

```

## Hardware Level Details

The onboard LED on the Raspberry Pi Pico 2W is not physically coupled to standard processor GPIO lines. Instead, it is routed as an internal peripheral bitmask inside the **Infineon CYW43439** wireless chip.

This project orchestrates a dedicated **PIO (Programmable I/O)** state machine alongside a high-performance **DMA Channel** to drive an asynchronous SPI communication loop with the CYW43 controller, allowing context-safe LED state switches over safe `embassy_rp::Peri` proxy transfers.

## License

This project is licensed under the **MIT License** - see the LICENSE file for details.

## Acknowledgements

* Embassy – embedded async framework
* Raspberry Pi – Pico 2W hardware
