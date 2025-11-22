#![allow(dead_code)]
mod span;
mod with_context;
pub use span::*;
pub use with_context::*;

use codespan_reporting::diagnostic::*;

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
        assert_eq!(self.id, other.id);
        self.diagnostics.extend_from_slice(&other.diagnostics);
    }
    #[must_use]
    pub fn diagnostic(
        &mut self,
        severity: Severity,
        message: impl Into<String>,
    ) -> LogBuilder {
        LogBuilder {
            logger: self,
            severity,
            message: message.into(),
            labels: vec![],
            notes: vec![],
        }
    }
    #[must_use]
    pub fn bug(
        &mut self,
        message: impl Into<String>,
    ) -> LogBuilder {
        self.diagnostic(Severity::Warning, message)
    }
    #[must_use]
    pub fn error(
        &mut self,
        message: impl Into<String>,
    ) -> LogBuilder {
        self.diagnostic(Severity::Error, message)
    }
    #[must_use]
    pub fn warning(
        &mut self,
        message: impl Into<String>,
    ) -> LogBuilder {
        self.diagnostic(Severity::Warning, message)
    }
    #[must_use]
    pub fn help(
        &mut self,
        message: impl Into<String>,
    ) -> LogBuilder {
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
}

#[derive(Debug, Clone)]
pub struct Log {
    severity: Severity,
    message: String,
    labels: Vec<Label<FileId>>,
    notes: Vec<String>,
}

#[derive(Debug)]
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
