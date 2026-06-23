#![no_std]
#![no_main]

#[cfg(debug_assertions)]
mod logger;
#[cfg(debug_assertions)]
mod usb_logger;

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! info {
    ($($tt:tt)*) => {{}};
}
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! warn {
    ($($tt:tt)*) => {{}};
}
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! error {
    ($($tt:tt)*) => {{}};
}
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug {
    ($ ($tt:tt)*) => {{}};
}
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! trace {
    ($($tt:tt)*) => {{}};
}

#[cfg(debug_assertions)]
use log::*;

use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use cyw43_setup::{FW, NVRAM};
use embassy_executor::Spawner;
use embassy_rp as hal;
use embassy_rp::bind_interrupts;
use embassy_rp::block::ImageDef;
use embassy_rp::dma::{self, Channel};
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_time::{Duration, Timer};
use panic_halt as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>>,
) -> ! {
    runner.run().await
}

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = hal::block::ImageDef::secure_exe();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    #[cfg(debug_assertions)]
    {
        let class = usb_logger::init_usb(spawner, p.USB).await;
        let (sender, _reader) = class.split();
        spawner.spawn(logger::log_task(sender)).unwrap();
        logger::init_logger().unwrap();
    }
    info!("System started, initializing CYW43...");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);

    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        RM2_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        Channel::new(p.DMA_CH0, Irqs),
    );

    let fw = &FW;
    let nvram = &NVRAM;

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw, nvram).await;

    spawner.spawn(cyw43_task(runner)).unwrap();

    Timer::after(Duration::from_millis(100)).await;
    info!("CYW43 ready, starting LED blink...");

    let mut led_state = false;
    loop {
        led_state = !led_state;
        control.gpio_set(0, led_state).await;
        if led_state {
            info!("LED ON");
        } else {
            info!("LED OFF");
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Template Rust Pico 2W"),
    embassy_rp::binary_info::rp_program_description!(
        c"USB serial logging + CYW43 initialization + onboard LED blinky"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];
