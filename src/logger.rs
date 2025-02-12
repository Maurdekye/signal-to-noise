use std::io::{Write, stdout};

use log::{LevelFilter, Log, SetLoggerError, set_logger, set_max_level};

pub struct Logger;

impl Logger {
    pub fn install(level: LevelFilter) -> Result<(), SetLoggerError> {
        set_logger(&Logger).map(|_| set_max_level(level))
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.target().starts_with("signal_to_noise") && metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!(
                "[{}:{}] {}",
                record.target(),
                record.line().unwrap_or_default(),
                record.args()
            );
        }
    }

    fn flush(&self) {
        stdout().flush().unwrap();
    }
}
