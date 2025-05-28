use core::fmt::Write as _;

use bevy_app::{App, Plugin};
use log::{Level, LevelFilter, Metadata, Record};
use psx::sys::tty::TTY;

/// Integrates logging with the MGBA emulator.
#[derive(Default)]
pub struct PSXLogPlugin;

impl Plugin for PSXLogPlugin {
    fn build(&self, _app: &mut App) {
        // SAFETY: Plugin::build should only be called once at a time
        unsafe {
            if let Ok(()) = log::set_logger_racy(&PSXLogger) {
                log::set_max_level_racy(LevelFilter::Info);
            }
        }
    }
}

struct PSXLogger;

impl log::Log for PSXLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Error => TTY.write_str("[ERROR]").ok(),
                Level::Warn => TTY.write_str("[WARN] ").ok(),
                Level::Info => TTY.write_str("[INFO] ").ok(),
                Level::Debug => TTY.write_str("[DEBUG]").ok(),
                Level::Trace => TTY.write_str("[TRACE]").ok(),
            };

            TTY.write_fmt(*record.args()).ok();

            unsafe {
                psx::sys::kernel::psx_printf(b"\n\0".as_ptr() as *const i8);
            }
        }
    }

    fn flush(&self) {}
}
