use super::*;
use crate::Span;
use codespan_reporting::diagnostic::{
    Label,
    LabelStyle,
};

pub trait WithContext: Sized {
    type DoneT;
    #[must_use = "Log should be submitted using the `.done()` method"]
    fn label(
        self,
        style: LabelStyle,
        message: impl Into<String>,
        span: Span,
    ) -> Self;
    #[must_use = "Log should be submitted using the `.done()` method"]
    fn note(
        self,
        message: impl Into<String>,
    ) -> Self;
    #[must_use = "Log should be submitted using the `.done()` method"]
    fn primary(
        self,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        self.label(LabelStyle::Primary, message, span)
    }
    #[must_use = "Log should be submitted using the `.done()` method"]
    fn secondary(
        self,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        self.label(LabelStyle::Secondary, message, span)
    }
    fn done(self) -> Self::DoneT;
}

impl<'a> WithContext for super::LogBuilder<'a> {
    type DoneT = &'a mut FileLogger;
    fn label(
        mut self,
        style: LabelStyle,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        if Span::Generated == span {
            self.notes
                .push("This lint occured in generated code".into());
        }
        let file_id = span.file_id().unwrap_or(self.logger.id());
        let span = span.range();
        self.labels.push(Label {
            style,
            file_id,
            range: span,
            message: message.into(),
        });
        self
    }
    fn note(
        mut self,
        message: impl Into<String>,
    ) -> Self {
        self.notes.push(message.into());
        self
    }
    fn done(self) -> &'a mut FileLogger {
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

impl<'a, T> WithContext for Result<T, super::LogBuilder<'a>> {
    type DoneT = Option<T>;
    fn label(
        self,
        style: LabelStyle,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        self.map_err(|e| e.label(style, message, span))
    }

    fn note(
        self,
        message: impl Into<String>,
    ) -> Self {
        self.map_err(|e| e.note(message))
    }

    fn done(self) -> Self::DoneT {
        self.map_err(|e| e.done()).ok()
    }
}
