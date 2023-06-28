use crate::core::ports;
use crate::core::ports::Logger;
use slog::Logger as SlogLogger;
use slog::{o, Drain};
use std::process;

pub struct SlogLoggerAdapter {
    logger: SlogLogger,
}

impl SlogLoggerAdapter {
    pub fn new() -> Self {
        // Create a synchronous logger drain
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();

        // Create an asynchronous logger drain
        let drain = slog_async::Async::new(drain).build().fuse();

        // Create the root logger
        let log = SlogLogger::root(drain, o!());
        Self { logger: log }
    }
}

impl Logger for SlogLoggerAdapter {
    fn info(&self, msg: &str) {
        slog::info!(self.logger, "{}", msg);
    }

    fn warn(&self, msg: &str) {
        slog::warn!(self.logger, "{}", msg);
    }

    fn error(&self, msg: &str) {
        slog::error!(self.logger, "{}", msg);
    }
    fn fatal(&self, msg: &str) {
        slog::crit!(self.logger, "{}", msg);
        process::exit(1);
    }
}
