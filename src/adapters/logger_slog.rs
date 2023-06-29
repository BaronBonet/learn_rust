use crate::core::ports;
use slog::{o, Drain};
use std::process;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct SlogLoggerAdapter {
    logger: Arc<Mutex<slog::Logger>>,
}

impl SlogLoggerAdapter {
    pub fn new() -> Self {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();

        let drain = slog_async::Async::new(drain).build().fuse();
        let logger = Arc::new(Mutex::new(slog::Logger::root(drain, o!())));

        Self { logger }
    }
}

impl ports::Logger for SlogLoggerAdapter {
    fn debug(&self, msg: &str) {
        let logger = self.logger.lock().unwrap();
        slog::debug!(logger, "{}", msg);
    }

    fn info(&self, msg: &str) {
        let logger = self.logger.lock().unwrap();
        slog::info!(logger, "{}", msg);
    }

    fn warn(&self, msg: &str) {
        let logger = self.logger.lock().unwrap();
        slog::warn!(logger, "{}", msg);
    }

    fn error(&self, msg: &str) {
        let logger = self.logger.lock().unwrap();
        slog::error!(logger, "{}", msg);
    }
    fn fatal(&self, msg: &str) {
        let logger = self.logger.lock().unwrap();
        slog::crit!(logger, "{}", msg);
        process::exit(1);
    }
}
