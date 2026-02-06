#![allow(dead_code)]
mod into_log;
mod span;
mod with_context;
pub use into_log::*;
pub use span::*;
pub use with_context::*;

use codespan_reporting::diagnostic::*;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;

pub type FileId = usize;

#[derive(Debug, Clone)]
pub struct Logger {
    id: FileId,
    diagnostics: Vec<Diagnostic<FileId>>,
}

impl Logger {
    pub fn new(id: FileId) -> Self {
        Self {
            id,
            diagnostics: vec![],
        }
    }
    /// Produces a mock logger for testing purposes
    pub fn mock() -> Self {
        Self {
            id: 0,
            diagnostics: vec![],
        }
    }
    pub fn spawn_new(&self) -> Self {
        Self {
            id: self.id,
            diagnostics: vec![],
        }
    }
    pub fn merge_with(
        &mut self,
        other: Self,
    ) {
        assert_eq!(self.id, other.id, "Merged loggers with different file IDs");
        self.diagnostics.extend_from_slice(&other.diagnostics);
    }
    pub fn consume_diagnostic(
        &mut self,
        diagnostic: Diagnostic<FileId>,
    ) {
        self.diagnostics.push(diagnostic);
    }
    pub fn diagnostic(
        &mut self,
        severity: Severity,
        message: impl Into<String>,
    ) -> LogBuilder<'_> {
        LogBuilder {
            logger: self,
            severity,
            message: message.into(),
            labels: vec![],
            notes: vec![],
        }
    }
    pub fn bug(
        &mut self,
        message: impl Into<String>,
    ) -> LogBuilder<'_> {
        self.diagnostic(Severity::Warning, message)
    }
    pub fn error(
        &'_ mut self,
        message: impl Into<String>,
    ) -> LogBuilder<'_> {
        self.diagnostic(Severity::Error, message)
    }
    pub fn warning(
        &mut self,
        message: impl Into<String>,
    ) -> LogBuilder<'_> {
        self.diagnostic(Severity::Warning, message)
    }
    pub fn help(
        &mut self,
        message: impl Into<String>,
    ) -> LogBuilder<'_> {
        self.diagnostic(Severity::Help, message)
    }
    pub fn is_ok(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|d| d.severity < Severity::Error)
    }
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic<FileId>> {
        self.diagnostics.iter()
    }
    pub fn new_span(
        &self,
        start: usize,
        width: usize,
    ) -> Span {
        Span {
            file_id: self.id,
            start,
            width,
        }
    }
    pub fn print(
        &self,
        files: &SimpleFiles<String, String>,
    ) {
        let mut writer = codespan_reporting::term::termcolor::StandardStream::stderr(
            codespan_reporting::term::termcolor::ColorChoice::Always,
        );
        let config = codespan_reporting::term::Config {
            display_style: codespan_reporting::term::DisplayStyle::Rich,
            ..Default::default()
        };
        for d in self.iter() {
            let _ = term::emit_to_write_style(&mut writer, &config, files, d);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    severity: Severity,
    message: String,
    labels: Vec<Label<FileId>>,
    notes: Vec<String>,
}

#[derive(Debug)]
#[must_use]
pub struct LogBuilder<'a>
where
    FileId: Clone,
{
    logger: &'a mut Logger,
    severity: Severity,
    message: String,
    labels: Vec<Label<FileId>>,
    notes: Vec<String>,
}

impl IntoIterator for Logger {
    type Item = Diagnostic<FileId>;

    type IntoIter = std::vec::IntoIter<Diagnostic<FileId>>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl<'a> IntoIterator for &'a Logger {
    type Item = &'a Diagnostic<FileId>;

    type IntoIter = std::slice::Iter<'a, Diagnostic<FileId>>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.iter()
    }
}
