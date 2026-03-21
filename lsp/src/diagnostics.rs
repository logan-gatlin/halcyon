use std::collections::{
    HashMap,
    HashSet,
};
use std::path::{
    Path,
    PathBuf,
};

use halcyon_lib::tooling::{
    AnalysisSourceFile,
    byte_offset_to_utf16_position,
};
use lsp_server::{
    Connection,
    Message,
    Notification,
};
use lsp_types::notification::{
    Notification as _,
    PublishDiagnostics,
};
use lsp_types::{
    Diagnostic,
    DiagnosticRelatedInformation,
    DiagnosticSeverity,
    Location,
    NumberOrString,
    PublishDiagnosticsParams,
    Uri,
};

use crate::util::{
    normalize_path,
    path_to_uri,
    text_range,
};

pub fn publish_bundle_diagnostics(
    connection: &Connection,
    source_files: &[AnalysisSourceFile],
    serialized_diagnostics: &[halcyon_lib::SerializedDiagnostic],
    open_document_versions: &HashMap<PathBuf, i32>,
    previous_uris: &HashSet<Uri>,
) -> Result<HashSet<Uri>, Box<dyn std::error::Error>> {
    let source_lookup = source_files
        .iter()
        .map(|file| (file.path.clone(), file.source.as_str()))
        .collect::<HashMap<_, _>>();

    let mut diagnostics_by_path: HashMap<PathBuf, Vec<Diagnostic>> = HashMap::new();
    for diagnostic in serialized_diagnostics {
        let Some((path, converted)) = convert_diagnostic(diagnostic, &source_lookup) else {
            continue;
        };
        diagnostics_by_path.entry(path).or_default().push(converted);
    }

    let mut current_uris = HashSet::new();
    for source_file in source_files {
        if !source_file.path.is_absolute() {
            continue;
        }
        let Some(uri) = path_to_uri(&source_file.path) else {
            continue;
        };
        let version = open_document_versions.get(&source_file.path).copied();
        let diagnostics = diagnostics_by_path
            .remove(&source_file.path)
            .unwrap_or_default();
        publish_diagnostics(connection, uri.clone(), diagnostics, version)?;
        current_uris.insert(uri);
    }

    for stale in previous_uris.difference(&current_uris) {
        publish_diagnostics(connection, stale.clone(), Vec::new(), None)?;
    }

    Ok(current_uris)
}

pub fn publish_diagnostics(
    connection: &Connection,
    uri: Uri,
    diagnostics: Vec<Diagnostic>,
    version: Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version,
    };
    let notification = Notification::new(PublishDiagnostics::METHOD.to_string(), params);
    connection
        .sender
        .send(Message::Notification(notification))?;
    Ok(())
}

fn convert_diagnostic(
    diagnostic: &halcyon_lib::SerializedDiagnostic,
    source_lookup: &HashMap<PathBuf, &str>,
) -> Option<(PathBuf, Diagnostic)> {
    let anchor = diagnostic
        .labels
        .iter()
        .enumerate()
        .filter_map(|(index, label)| {
            let path = normalize_path(Path::new(&label.file_name));
            let source = source_lookup.get(&path).copied()?;
            Some((index, label, path, source))
        })
        .find(|(_, label, ..)| label.style == "primary")
        .or_else(|| {
            diagnostic
                .labels
                .iter()
                .enumerate()
                .find_map(|(index, label)| {
                    let path = normalize_path(Path::new(&label.file_name));
                    let source = source_lookup.get(&path).copied()?;
                    Some((index, label, path, source))
                })
        })?;

    let (anchor_index, anchor, path, source) = anchor;
    let start = byte_offset_to_utf16_position(source, anchor.range_start);
    let end = byte_offset_to_utf16_position(source, anchor.range_end);
    let mut message = diagnostic.message.clone();
    if !anchor.message.is_empty() {
        message.push('\n');
        message.push_str(&anchor.message);
    }
    if !diagnostic.notes.is_empty() {
        message.push('\n');
        message.push_str(
            &diagnostic
                .notes
                .iter()
                .map(|note| format!("note: {note}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }

    let related_information = diagnostic
        .labels
        .iter()
        .enumerate()
        .filter_map(|(index, label)| {
            if index == anchor_index {
                return None;
            }
            let related_path = normalize_path(Path::new(&label.file_name));
            let related_source = source_lookup.get(&related_path).copied()?;
            let uri = path_to_uri(&related_path)?;
            let related_start = byte_offset_to_utf16_position(related_source, label.range_start);
            let related_end = byte_offset_to_utf16_position(related_source, label.range_end);
            let label_kind = if label.style == "primary" {
                "primary"
            } else {
                "secondary"
            };
            let label_message = if label.message.is_empty() {
                format!("{label_kind} span")
            } else {
                label.message.clone()
            };
            Some(DiagnosticRelatedInformation {
                location: Location {
                    uri,
                    range: text_range(related_start, related_end),
                },
                message: label_message,
            })
        })
        .collect::<Vec<_>>();

    Some((
        path,
        Diagnostic {
            range: text_range(start, end),
            severity: severity_from_str(&diagnostic.severity),
            code: diagnostic
                .code
                .as_ref()
                .map(|code| NumberOrString::String(code.clone())),
            code_description: None,
            source: Some("halcyon".to_string()),
            message,
            related_information: (!related_information.is_empty()).then_some(related_information),
            tags: None,
            data: None,
        },
    ))
}

fn severity_from_str(raw: &str) -> Option<DiagnosticSeverity> {
    match raw {
        "bug" | "error" => Some(DiagnosticSeverity::ERROR),
        "warning" => Some(DiagnosticSeverity::WARNING),
        "help" => Some(DiagnosticSeverity::HINT),
        _ => Some(DiagnosticSeverity::INFORMATION),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_diagnostic_includes_primary_label_message_and_notes() {
        let file_path = PathBuf::from("/tmp/demo.hc");
        let mut source_lookup = HashMap::new();
        source_lookup.insert(file_path.clone(), "use core\n");

        let diagnostic = halcyon_lib::SerializedDiagnostic {
            severity: "warning".to_string(),
            code: None,
            message: "Implicit bundle-relative path".to_string(),
            labels: vec![halcyon_lib::SerializedDiagnosticLabel {
                style: "primary".to_string(),
                file_name: "/tmp/demo.hc".to_string(),
                message: "`core` resolves to `root::core::`; use `bundle` to refer to the current bundle in a less ambiguous way.".to_string(),
                range_start: 4,
                range_end: 8,
                start: halcyon_lib::SerializedDiagnosticLocation { line: 1, column: 5 },
                end: halcyon_lib::SerializedDiagnosticLocation { line: 1, column: 9 },
            }],
            notes: vec!["Imported from core prelude.".to_string()],
        };

        let Some((_path, converted)) = convert_diagnostic(&diagnostic, &source_lookup) else {
            panic!("expected diagnostic to convert");
        };

        assert!(
            converted.message.contains("Implicit bundle-relative path"),
            "message should keep top-level diagnostic message"
        );
        assert!(
            converted
                .message
                .contains("`core` resolves to `root::core::`"),
            "message should include primary label text"
        );
        assert!(
            converted
                .message
                .contains("note: Imported from core prelude."),
            "message should include notes"
        );
    }

    #[test]
    fn convert_diagnostic_emits_secondary_labels_as_related_information() {
        let file_path = PathBuf::from("/tmp/demo.hc");
        let mut source_lookup = HashMap::new();
        source_lookup.insert(file_path.clone(), "use core\nlet x = core::value\n");

        let diagnostic = halcyon_lib::SerializedDiagnostic {
            severity: "warning".to_string(),
            code: None,
            message: "Ambiguous reference".to_string(),
            labels: vec![
                halcyon_lib::SerializedDiagnosticLabel {
                    style: "primary".to_string(),
                    file_name: "/tmp/demo.hc".to_string(),
                    message: "first candidate".to_string(),
                    range_start: 4,
                    range_end: 8,
                    start: halcyon_lib::SerializedDiagnosticLocation { line: 1, column: 5 },
                    end: halcyon_lib::SerializedDiagnosticLocation { line: 1, column: 9 },
                },
                halcyon_lib::SerializedDiagnosticLabel {
                    style: "secondary".to_string(),
                    file_name: "/tmp/demo.hc".to_string(),
                    message: "alternative candidate".to_string(),
                    range_start: 17,
                    range_end: 21,
                    start: halcyon_lib::SerializedDiagnosticLocation { line: 2, column: 9 },
                    end: halcyon_lib::SerializedDiagnosticLocation {
                        line: 2,
                        column: 13,
                    },
                },
            ],
            notes: vec![],
        };

        let Some((_path, converted)) = convert_diagnostic(&diagnostic, &source_lookup) else {
            panic!("expected diagnostic to convert");
        };

        let related = converted.related_information.unwrap_or_default();
        assert_eq!(related.len(), 1, "one secondary span should be related");
        assert_eq!(
            related[0].message, "alternative candidate",
            "secondary label message should be preserved"
        );
    }
}
