use core::fmt::Write;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_usb::class::cdc_acm::Sender;
use heapless::String;
use log::{Metadata, Record};

pub static LOG_CHANNEL: Channel<CriticalSectionRawMutex, String<128>, 16> = Channel::new();

#[embassy_executor::task]
pub async fn log_task(mut sender: Sender<'static, Driver<'static, USB>>) {
    loop {
        let msg = LOG_CHANNEL.receive().await;
        let _ = sender.write_packet(msg.as_bytes()).await;
        let _ = sender.write_packet(b"\r\n").await;
    }
}

struct UsbLogger;

impl log::Log for UsbLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level_str = match record.level() {
            log::Level::Error => "ERROR",
            log::Level::Warn => "WARN",
            log::Level::Info => "INFO",
            log::Level::Debug => "DEBUG",
            log::Level::Trace => "TRACE",
        };

        let mut s = String::<128>::new();
        let _ = write!(&mut s, "[{}] ", level_str);
        let _ = write!(&mut s, "{}", record.args());

        let _ = LOG_CHANNEL.try_send(s);
    }

    fn flush(&self) {}
}

pub fn init_logger() -> Result<(), log::SetLoggerError> {
    static LOGGER: UsbLogger = UsbLogger;
    log::set_logger(&LOGGER)?;
    log::set_max_level(log::LevelFilter::Info);
    Ok(())
}
