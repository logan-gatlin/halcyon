#![allow(dead_code)]
mod span;
pub use span::*;

use codespan_reporting::diagnostic::*;

pub type FileId = usize;
pub type LoggerT = Logger<FileId>;

#[derive(Debug, Clone)]
pub struct Logger<FileId> {
    id: FileId,
    diagnostics: Vec<Diagnostic<FileId>>,
}

impl<FileId> Logger<FileId>
where
    FileId: Clone,
{
    pub fn new(id: FileId) -> Self {
        Self {
            id,
            diagnostics: vec![],
        }
    }
    #[must_use]
    pub fn diagnostic(
        &mut self,
        severity: Severity,
        message: impl Into<String>,
    ) -> LogBuilder<FileId> {
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
    ) -> LogBuilder<FileId> {
        self.diagnostic(Severity::Warning, message)
    }

    #[must_use]
    pub fn error(
        &mut self,
        message: impl Into<String>,
    ) -> LogBuilder<FileId> {
        self.diagnostic(Severity::Error, message)
    }

    #[must_use]
    pub fn warning(
        &mut self,
        message: impl Into<String>,
    ) -> LogBuilder<FileId> {
        self.diagnostic(Severity::Warning, message)
    }

    #[must_use]
    pub fn help(
        &mut self,
        message: impl Into<String>,
    ) -> LogBuilder<FileId> {
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

#[derive(Debug)]
pub struct LogBuilder<'a, FileId>
where
    FileId: Clone,
{
    logger: &'a mut Logger<FileId>,
    severity: Severity,
    message: String,
    labels: Vec<Label<FileId>>,
    notes: Vec<String>,
}

impl<'a, FileId> LogBuilder<'a, FileId>
where
    FileId: Clone,
{
    #[must_use]
    pub fn label(
        mut self,
        style: LabelStyle,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        let span = span.start..(span.start + span.width);
        self.labels.push(Label {
            style,
            file_id: self.logger.id.clone(),
            range: span,
            message: message.into(),
        });
        self
    }

    #[must_use]
    pub fn primary(
        self,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        self.label(LabelStyle::Primary, message, span)
    }

    #[must_use]
    pub fn secondary(
        self,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        self.label(LabelStyle::Secondary, message, span)
    }

    #[must_use]
    pub fn note(
        mut self,
        message: impl Into<String>,
    ) -> Self {
        self.notes.push(message.into());
        self
    }

    pub fn done(self) -> &'a mut Logger<FileId> {
        self.logger.diagnostics.push(Diagnostic {
            severity: self.severity,
            code: None,
            message: self.message,
            labels: self.labels,
            notes: self.notes,
        });
        self.logger
    }
}

impl<FileId> IntoIterator for Logger<FileId> {
    type Item = Diagnostic<FileId>;

    type IntoIter = std::vec::IntoIter<Diagnostic<FileId>>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl<'a, FileId> IntoIterator for &'a Logger<FileId> {
    type Item = &'a Diagnostic<FileId>;

    type IntoIter = std::slice::Iter<'a, Diagnostic<FileId>>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.iter()
    }
}
