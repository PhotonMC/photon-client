//! The client's log: one line per record on stderr, filtered by `VOXEL_LOG`.
//!
//! The probe and profile reports (`key=value` lines) are not log records; they
//! stay on stdout.

use std::env;
use std::io::Write;

use log::{Level, LevelFilter, Log, Metadata, Record};

struct Logger {
    level: LevelFilter,
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // A broken stderr leaves nowhere to report the failure.
            let _ = writeln!(
                std::io::stderr().lock(),
                "[{} {}] {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

/// Start logging at the level `VOXEL_LOG` names (`error`, `warn`, `info`,
/// `debug`, `trace` or `off`), `info` when it is unset or not one of those.
pub fn init() {
    let level = env::var("VOXEL_LOG")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(Level::Info.to_level_filter());
    // A second call finds the logger already set, which is as good.
    if log::set_boxed_logger(Box::new(Logger { level })).is_ok() {
        log::set_max_level(level);
    }
}
