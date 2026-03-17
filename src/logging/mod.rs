#![allow(dead_code)]
mod span;
mod with_context;
pub use span::*;
pub use with_context::*;

use codespan_reporting::diagnostic::*;
use codespan_reporting::files::{
    Files,
    SimpleFiles,
};
use codespan_reporting::term;
use wasm_bindgen::prelude::*;

pub type FileId = usize;
pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<FileId>;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SerializedDiagnosticLocation {
    pub line: usize,
    pub column: usize,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SerializedDiagnosticLabel {
    pub style: String,
    pub file_name: String,
    pub message: String,
    pub range_start: usize,
    pub range_end: usize,
    pub start: SerializedDiagnosticLocation,
    pub end: SerializedDiagnosticLocation,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SerializedDiagnostic {
    pub severity: String,
    pub code: Option<String>,
    pub message: String,
    pub labels: Vec<SerializedDiagnosticLabel>,
    pub notes: Vec<String>,
}

fn fallback_location(byte_index: usize) -> SerializedDiagnosticLocation {
    SerializedDiagnosticLocation {
        line: 1,
        column: byte_index.saturating_add(1),
    }
}

fn location_for_offset(
    files: &SimpleFiles<String, String>,
    file_id: FileId,
    byte_index: usize,
) -> SerializedDiagnosticLocation {
    let Ok(line_index) = files.line_index(file_id, byte_index) else {
        return fallback_location(byte_index);
    };
    let Ok(line_range) = files.line_range(file_id, line_index) else {
        return fallback_location(byte_index);
    };
    SerializedDiagnosticLocation {
        line: line_index.saturating_add(1),
        column: byte_index
            .saturating_sub(line_range.start)
            .saturating_add(1),
    }
}

fn serialize_label(
    label: &Label<FileId>,
    files: &SimpleFiles<String, String>,
) -> SerializedDiagnosticLabel {
    let file_name = files
        .name(label.file_id)
        .map(|name| name.to_string())
        .unwrap_or_else(|_| format!("<unknown-file-{}>", label.file_id));
    let range_start = label.range.start;
    let range_end = label.range.end;
    let start = location_for_offset(files, label.file_id, range_start);
    let end_offset = if range_end > range_start {
        range_end.saturating_sub(1)
    } else {
        range_end
    };
    let end = location_for_offset(files, label.file_id, end_offset);

    SerializedDiagnosticLabel {
        style: format!("{:?}", label.style).to_lowercase(),
        file_name,
        message: label.message.clone(),
        range_start,
        range_end,
        start,
        end,
    }
}

fn serialize_diagnostic(
    diagnostic: &Diagnostic,
    files: &SimpleFiles<String, String>,
) -> SerializedDiagnostic {
    SerializedDiagnostic {
        severity: format!("{:?}", diagnostic.severity).to_lowercase(),
        code: diagnostic.code.clone(),
        message: diagnostic.message.clone(),
        labels: diagnostic
            .labels
            .iter()
            .map(|label| serialize_label(label, files))
            .collect(),
        notes: diagnostic.notes.clone(),
    }
}

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
    file_records: Vec<(FileId, String, String)>,
    diagnostics: Vec<Diagnostic>,
    /// Special "file id" used to emit linking errors which do not originate
    /// from any particular file
    linking_id: FileId,
}

impl Logger {
    pub fn new() -> Self {
        let mut files = SimpleFiles::new();
        let linking_id = files.add("Linking phase".to_string(), "".to_string());
        Self {
            files,
            file_records: vec![(linking_id, "Linking phase".to_string(), "".to_string())],
            diagnostics: vec![],
            linking_id,
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
        let file_name = file_name.into();
        let file_contents = file_contents.into();
        let file_id = self.files.add(file_name.clone(), file_contents.clone());
        self.file_records
            .push((file_id, file_name.clone(), file_contents));
        FileLogger {
            id: file_id,
            file_name,
            diagnostics: vec![],
        }
    }
    pub fn linking_logger(&mut self) -> FileLogger {
        FileLogger {
            id: self.linking_id,
            file_name: "<linking>".to_string(),
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

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }

    pub fn serialize(&self) -> Vec<SerializedDiagnostic> {
        self.diagnostics
            .iter()
            .map(|diagnostic| serialize_diagnostic(diagnostic, &self.files))
            .collect()
    }

    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn source_files(&self) -> Vec<(FileId, String, String)> {
        self.file_records.clone()
    }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct FileLogger {
    id: FileId,
    file_name: String,
    diagnostics: Vec<Diagnostic>,
}

impl FileLogger {
    pub fn new(id: FileId) -> Self {
        Self {
            id,
            file_name: "<generated>".to_string(),
            diagnostics: vec![],
        }
    }
    pub fn with_name(
        id: FileId,
        file_name: impl Into<String>,
    ) -> Self {
        Self {
            id,
            file_name: file_name.into(),
            diagnostics: vec![],
        }
    }
    pub fn spawn_new(&self) -> Self {
        Self {
            id: self.id,
            file_name: self.file_name.clone(),
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
        diagnostic: Diagnostic,
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

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn id(&self) -> FileId {
        self.id
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    pub severity: Severity,
    pub message: String,
    pub labels: Vec<Label<FileId>>,
    pub notes: Vec<String>,
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
    type Item = Diagnostic;

    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}

impl<'a> IntoIterator for &'a FileLogger {
    type Item = &'a Diagnostic;

    type IntoIter = std::slice::Iter<'a, Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_diagnostics_includes_file_context() {
        let mut logger = Logger::new();
        let mut file_logger = logger.new_file("demo.hc", "let value = missing\n");
        file_logger
            .error("Unknown identifier")
            .primary("`missing` is not defined.", Span::new(12, 7))
            .note("This came from a unit test.")
            .done();
        logger.consume_file(file_logger);

        let serialized = logger.serialize();
        assert_eq!(
            serialized.len(),
            1,
            "expected exactly one serialized diagnostic"
        );
        let Some(diagnostic) = serialized.first() else {
            panic!("expected a serialized diagnostic entry");
        };
        assert_eq!(diagnostic.severity, "error", "severity should be preserved");
        assert_eq!(
            diagnostic.message, "Unknown identifier",
            "message should be preserved"
        );
        let Some(label) = diagnostic.labels.first() else {
            panic!("expected one serialized label");
        };
        assert_eq!(label.file_name, "demo.hc", "file name should be embedded");
        assert_eq!(label.start.line, 1, "start line should be resolved");
        assert_eq!(label.start.column, 13, "start column should be resolved");
        assert_eq!(label.end.line, 1, "end line should be resolved");
        assert_eq!(label.end.column, 19, "end column should be resolved");
    }
}
