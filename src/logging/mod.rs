#![allow(dead_code)]
mod span;
mod with_context;
pub use span::*;
pub use with_context::*;

use codespan_reporting::diagnostic::*;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;

pub type FileId = usize;

/// Simple logger for testing purposes. Only supports a single file
#[derive(Debug, Clone)]
pub struct MockLogger {
    logger: Logger,
    file_logger: FileLogger,
}

impl MockLogger {
    pub fn new(source: impl Into<String>) -> Self {
        let mut logger = Logger::new();
        let file_logger = logger.new_file("<test-file>", source);
        Self {
            logger,
            file_logger,
        }
    }

    pub fn logger(&mut self) -> &mut Logger {
        &mut self.logger
    }

    pub fn file(&mut self) -> &mut FileLogger {
        &mut self.file_logger
    }
}

#[derive(Debug, Clone, Default)]
pub struct Logger {
    files: SimpleFiles<String, String>,
    diagnostics: Vec<Diagnostic<FileId>>,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            files: SimpleFiles::new(),
            diagnostics: vec![],
        }
    }

    pub fn consume_file(
        &mut self,
        logger: FileLogger,
    ) {
        self.diagnostics.extend(logger.diagnostics);
    }

    pub fn new_file(
        &mut self,
        file_name: impl Into<String>,
        file_contents: impl Into<String>,
    ) -> FileLogger {
        let file_id = self.files.add(file_name.into(), file_contents.into());
        FileLogger {
            id: file_id,
            diagnostics: vec![],
        }
    }

    pub fn print_logs(&self) {
        let mut writer = codespan_reporting::term::termcolor::StandardStream::stderr(
            codespan_reporting::term::termcolor::ColorChoice::Always,
        );
        let config = codespan_reporting::term::Config {
            display_style: codespan_reporting::term::DisplayStyle::Rich,
            ..Default::default()
        };
        for d in &self.diagnostics {
            let _ = term::emit_to_write_style(&mut writer, &config, &self.files, d);
        }
    }
    pub fn is_ok(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|d| d.severity < Severity::Error)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic<FileId>> {
        self.diagnostics.iter()
    }

    pub fn error_messages(&self) -> Vec<String> {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity >= Severity::Error)
            .map(|diagnostic| diagnostic.message.clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct FileLogger {
    id: FileId,
    diagnostics: Vec<Diagnostic<FileId>>,
}

impl FileLogger {
    pub fn new(id: FileId) -> Self {
        Self {
            id,
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
        self.diagnostic(Severity::Bug, message)
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
    pub fn escalate_to_bug(&mut self) {
        for d in &mut self.diagnostics {
            d.severity = Severity::Bug;
        }
    }
    pub fn is_ok(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|d| d.severity < Severity::Error)
    }
    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic<FileId>> {
        self.diagnostics.iter()
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
    logger: &'a mut FileLogger,
    severity: Severity,
    message: String,
    labels: Vec<Label<FileId>>,
    notes: Vec<String>,
}

impl IntoIterator for FileLogger {
    type Item = Diagnostic<FileId>;

    type IntoIter = std::vec::IntoIter<Diagnostic<FileId>>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl<'a> IntoIterator for &'a FileLogger {
    type Item = &'a Diagnostic<FileId>;

    type IntoIter = std::slice::Iter<'a, Diagnostic<FileId>>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.iter()
    }
}
