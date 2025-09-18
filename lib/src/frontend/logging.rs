use crate::Span;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Debug = 0,
    Warn = 1,
    Error = 2,
}

/// Log message generated during compilation
#[derive(Debug, Clone)]
pub struct Log {
    pub severity: Severity,
    pub span: Option<Span>,
    /// Short description of the error
    pub message: String,
    /// Possible solution for the error
    pub help: Option<String>,
}

pub fn error(message: impl Into<String>) -> Log {
    Log {
        severity: Severity::Error,
        span: None,
        message: message.into(),
        help: None,
    }
}

impl Log {
    pub fn span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct Logger {
    source_name: String,
    logs: Vec<Log>,
}

impl Logger {
    pub fn new(source_name: impl Into<String>) -> Self {
        Self {
            source_name: source_name.into(),
            logs: vec![],
        }
    }

    /// Append a log message
    pub fn log(&mut self, log: Log) {
        self.logs.push(log);
    }

    /// Returns true if no errors are reported so far
    pub fn is_ok(&self) -> bool {
        self.logs.iter().all(|l| l.severity < Severity::Error)
    }

    /// Destroy the logger, returning all recorded logs
    pub fn into_logs(self) -> Vec<Log> {
        self.logs
    }

    /// Destroy the logger, returning all recorded logs that are at least as bad as the `filter`
    pub fn into_filtered_logs(self, filter: Severity) -> Vec<Log> {
        self.logs
            .into_iter()
            .filter(|l| l.severity >= filter)
            .collect()
    }
}
