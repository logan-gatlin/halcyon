use std::collections::{
    HashMap,
    HashSet,
};
use std::path::{
    Path,
    PathBuf,
};
use std::sync::mpsc::{
    self,
    Receiver,
    Sender,
};

use halcyon_lib::parse::ast::{
    self,
    AstNode,
    HasName,
};
use halcyon_lib::parse::{
    self,
    SyntaxKind,
};
use halcyon_lib::tooling::{
    AnalysisSourceFile,
    BundleAnalysis,
    FrontendBundleAnalysis,
    analyze_bundle_frontend_with_symbols_and_resolver,
    analyze_bundle_with_symbols_and_resolver,
    build_core_symbols,
    byte_offset_to_utf16_position,
    find_nearest_bundle_root,
    utf16_position_to_byte_offset,
};
use halcyon_lib::{
    Logger,
    Span,
    Spanned,
};
use inflections::Inflect;
use lsp_server::{
    Connection,
    ErrorCode,
    Message,
    Notification,
    Request,
    Response,
    ResponseError,
};
use lsp_types::notification::{
    DidChangeTextDocument,
    DidCloseTextDocument,
    DidOpenTextDocument,
    Notification as _,
};
use lsp_types::request::{
    CodeActionRequest,
    Completion,
    GotoDefinition,
    HoverRequest,
    References,
    Rename,
    Request as _,
};
use lsp_types::{
    CodeAction,
    CodeActionKind,
    CodeActionOrCommand,
    CodeActionParams,
    CodeActionProviderCapability,
    CodeActionResponse,
    CompletionOptions,
    CompletionParams,
    CompletionResponse,
    DidChangeTextDocumentParams,
    DidCloseTextDocumentParams,
    DidOpenTextDocumentParams,
    GotoDefinitionParams,
    GotoDefinitionResponse,
    Hover,
    HoverContents,
    HoverParams,
    Location,
    MarkupContent,
    MarkupKind,
    OneOf,
    Position,
    ReferenceParams,
    RenameParams,
    ServerCapabilities,
    TextDocumentSyncCapability,
    TextDocumentSyncKind,
    TextEdit,
    Uri,
    WorkspaceEdit,
};

use crate::completion::{
    completion_context_at,
    completion_items,
};
use crate::diagnostics::{
    publish_bundle_diagnostics,
    publish_diagnostics,
};
use crate::keyword_hover::hover_for_keyword;
use crate::protocol::{
    parse_notification,
    parse_request,
    response_error,
    send_response,
};
use crate::util::{
    normalize_path,
    path_to_uri,
    text_range,
    uri_to_path,
};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (connection, io_threads) = Connection::stdio();

    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions::default()),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        hover_provider: Some(true.into()),
        rename_provider: Some(OneOf::Left(true)),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        ..ServerCapabilities::default()
    };

    let capabilities = serde_json::to_value(capabilities)?;
    let _init_params = connection.initialize(capabilities)?;

    let mut server = Server::new();

    for message in &connection.receiver {
        server.drain_typecheck_results();
        match message {
            Message::Request(request) => {
                if connection.handle_shutdown(&request)? {
                    break;
                }
                server.handle_request(request, &connection)?;
            }
            Message::Notification(notification) => {
                server.handle_notification(notification, &connection)?;
            }
            Message::Response(_) => {}
        }
    }

    io_threads.join()?;
    Ok(())
}

#[derive(Debug, Clone)]
struct OpenDocument {
    version: i32,
    text: String,
}

#[derive(Debug, Clone)]
struct TypedSnapshot {
    generation: u64,
    analysis: BundleAnalysis,
}

#[derive(Debug, Clone)]
struct BundleState {
    frontend: FrontendBundleAnalysis,
    typed: Option<TypedSnapshot>,
    generation: u64,
    published_uris: HashSet<Uri>,
}

#[derive(Debug)]
struct TypecheckResult {
    root_path: PathBuf,
    generation: u64,
    analysis: Result<BundleAnalysis, String>,
}

pub struct Server {
    base_symbols: halcyon_lib::types::SymbolTable,
    open_documents: HashMap<PathBuf, OpenDocument>,
    bundles: HashMap<PathBuf, BundleState>,
    typecheck_result_sender: Sender<TypecheckResult>,
    typecheck_result_receiver: Receiver<TypecheckResult>,
}

impl Server {
    pub fn new() -> Self {
        let (typecheck_result_sender, typecheck_result_receiver) = mpsc::channel();
        Self {
            base_symbols: build_core_symbols(),
            open_documents: HashMap::new(),
            bundles: HashMap::new(),
            typecheck_result_sender,
            typecheck_result_receiver,
        }
    }

    pub fn handle_request(
        &mut self,
        request: Request,
        connection: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.drain_typecheck_results();

        match request.method.as_str() {
            Completion::METHOD => {
                let (id, params) = parse_request::<CompletionParams>(request)?;
                let result = self.completion(params, connection);
                send_response(id, result, connection)?;
            }
            CodeActionRequest::METHOD => {
                let (id, params) = parse_request::<CodeActionParams>(request)?;
                let result = self.code_action(params, connection);
                send_response(id, result, connection)?;
            }
            HoverRequest::METHOD => {
                let (id, params) = parse_request::<HoverParams>(request)?;
                let result = self.hover(params, connection);
                send_response(id, result, connection)?;
            }
            GotoDefinition::METHOD => {
                let (id, params) = parse_request::<GotoDefinitionParams>(request)?;
                let result = self.goto_definition(params, connection);
                send_response(id, result, connection)?;
            }
            References::METHOD => {
                let (id, params) = parse_request::<ReferenceParams>(request)?;
                let result = self.references(params, connection);
                send_response(id, result, connection)?;
            }
            Rename::METHOD => {
                let (id, params) = parse_request::<RenameParams>(request)?;
                let result = self.rename(params, connection);
                send_response(id, result, connection)?;
            }
            _ => {
                let response = Response {
                    id: request.id,
                    result: None,
                    error: Some(ResponseError {
                        code: ErrorCode::MethodNotFound as i32,
                        message: format!("Unsupported request `{}`", request.method),
                        data: None,
                    }),
                };
                connection.sender.send(Message::Response(response))?;
            }
        }

        Ok(())
    }

    pub fn handle_notification(
        &mut self,
        notification: Notification,
        connection: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.drain_typecheck_results();

        match notification.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let params = parse_notification::<DidOpenTextDocumentParams>(notification)?;
                self.did_open(params, connection)?;
            }
            DidChangeTextDocument::METHOD => {
                let params = parse_notification::<DidChangeTextDocumentParams>(notification)?;
                self.did_change(params, connection)?;
            }
            DidCloseTextDocument::METHOD => {
                let params = parse_notification::<DidCloseTextDocumentParams>(notification)?;
                self.did_close(params, connection)?;
            }
            _ => {}
        }

        Ok(())
    }

    fn did_open(
        &mut self,
        params: DidOpenTextDocumentParams,
        connection: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(path) = uri_to_path(&params.text_document.uri) else {
            return Ok(());
        };

        self.open_documents.insert(
            path.clone(),
            OpenDocument {
                version: params.text_document.version,
                text: params.text_document.text,
            },
        );
        self.reanalyze_path(&path, connection)
    }

    fn did_change(
        &mut self,
        params: DidChangeTextDocumentParams,
        connection: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(path) = uri_to_path(&params.text_document.uri) else {
            return Ok(());
        };

        let Some(last_change) = params.content_changes.into_iter().last() else {
            return Ok(());
        };

        self.open_documents.insert(
            path.clone(),
            OpenDocument {
                version: params.text_document.version,
                text: last_change.text,
            },
        );
        self.reanalyze_path(&path, connection)
    }

    fn did_close(
        &mut self,
        params: DidCloseTextDocumentParams,
        connection: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(path) = uri_to_path(&params.text_document.uri) else {
            return Ok(());
        };

        self.open_documents.remove(&path);
        self.reanalyze_path(&path, connection)
    }

    fn completion(
        &mut self,
        params: CompletionParams,
        connection: &Connection,
    ) -> Result<Option<CompletionResponse>, ResponseError> {
        let Some(path) = uri_to_path(&params.text_document_position.text_document.uri) else {
            return Ok(Some(CompletionResponse::Array(Vec::new())));
        };
        let Some(root_path) = self
            .ensure_frontend_for_path(&path, connection)
            .map_err(|error| response_error(error.to_string()))?
        else {
            return Ok(Some(CompletionResponse::Array(Vec::new())));
        };

        let Some(bundle) = self.bundles.get(&root_path) else {
            return Ok(Some(CompletionResponse::Array(Vec::new())));
        };
        let Some(source) = self.source_for_path_from_frontend(&path, &bundle.frontend) else {
            return Ok(Some(CompletionResponse::Array(Vec::new())));
        };

        let context = completion_context_at(&source, params.text_document_position.position)
            .unwrap_or_default();
        let symbols = bundle
            .typed
            .as_ref()
            .map(|typed| &typed.analysis.symbols)
            .unwrap_or(&self.base_symbols);
        let items = completion_items(symbols, &context);
        Ok(Some(CompletionResponse::Array(items)))
    }

    fn code_action(
        &mut self,
        params: CodeActionParams,
        connection: &Connection,
    ) -> Result<Option<CodeActionResponse>, ResponseError> {
        if !quickfix_requested(&params) {
            return Ok(Some(Vec::new()));
        }

        let Some(path) = uri_to_path(&params.text_document.uri) else {
            return Ok(Some(Vec::new()));
        };
        let Some(root_path) = self
            .ensure_frontend_for_path(&path, connection)
            .map_err(|error| response_error(error.to_string()))?
        else {
            return Ok(Some(Vec::new()));
        };
        let Some(bundle) = self.bundles.get(&root_path) else {
            return Ok(Some(Vec::new()));
        };
        let Some(source) = self.source_for_path_from_frontend(&path, &bundle.frontend) else {
            return Ok(Some(Vec::new()));
        };
        let Some(file_id) = bundle.frontend.file_id_for_path(&path) else {
            return Ok(Some(Vec::new()));
        };
        let Some(range_start) = utf16_position_to_byte_offset(
            &source,
            params.range.start.line,
            params.range.start.character,
        ) else {
            return Ok(Some(Vec::new()));
        };
        let Some(range_end) = utf16_position_to_byte_offset(
            &source,
            params.range.end.line,
            params.range.end.character,
        ) else {
            return Ok(Some(Vec::new()));
        };

        let Some(source_file) = parse_source_file_for_naming_actions(
            path.as_os_str().to_string_lossy().as_ref(),
            &source,
        ) else {
            return Ok(Some(Vec::new()));
        };
        let mut actions = Vec::new();
        let mut seen = HashSet::new();
        for candidate in collect_naming_candidates(source_file) {
            if !naming_candidate_overlaps_range(&candidate, range_start, range_end) {
                continue;
            }
            let dedupe_key = format!(
                "{}:{}:{}:{}",
                candidate.start, candidate.width, candidate.current_name, candidate.replacement
            );
            if !seen.insert(dedupe_key) {
                continue;
            }

            let Some(edit) = self.naming_workspace_edit(
                &bundle.frontend,
                &params.text_document.uri,
                &source,
                file_id,
                &candidate,
            ) else {
                continue;
            };

            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: format!(
                    "Rename `{}` to `{}` ({})",
                    candidate.current_name,
                    candidate.replacement,
                    candidate.style.label()
                ),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None,
                edit: Some(edit),
                command: None,
                is_preferred: Some(true),
                disabled: None,
                data: None,
            }));
        }

        Ok(Some(actions))
    }

    fn hover(
        &mut self,
        params: HoverParams,
        connection: &Connection,
    ) -> Result<Option<Hover>, ResponseError> {
        let Some(path) = uri_to_path(&params.text_document_position_params.text_document.uri)
        else {
            return Ok(None);
        };

        let Some(root_path) = self
            .ensure_frontend_for_path(&path, connection)
            .map_err(|error| response_error(error.to_string()))?
        else {
            return Ok(None);
        };
        let Some(bundle) = self.bundles.get(&root_path) else {
            return Ok(None);
        };
        let Some(source) = self.source_for_path_from_frontend(&path, &bundle.frontend) else {
            return Ok(None);
        };

        let Some(symbol) = self.symbol_in_frontend(
            &path,
            params.text_document_position_params.position,
            &bundle.frontend,
            &source,
        ) else {
            if let Some(hover) =
                hover_for_keyword(&source, params.text_document_position_params.position)
            {
                return Ok(Some(hover));
            }
            return Ok(None);
        };

        let fully_qualified_path = symbol.path.to_string();
        let namespace = format!("{:?}", symbol.namespace).to_lowercase();
        let value_type_info = if matches!(
            symbol.namespace,
            halcyon_lib::ir::NameSpace::Term
                | halcyon_lib::ir::NameSpace::Constructor
                | halcyon_lib::ir::NameSpace::Wasm
        ) {
            bundle
                .typed
                .as_ref()
                .and_then(|typed| typed.analysis.symbols.terms().get(&symbol.path))
                .map(|scheme| scheme.pretty())
        } else {
            None
        };
        let kind_info = if symbol.namespace == halcyon_lib::ir::NameSpace::Type {
            self.type_kind_for_symbol(bundle, &symbol.path)
                .map(|kind| kind.pretty())
        } else {
            None
        };
        let stale_note = bundle
            .typed
            .as_ref()
            .is_some_and(|typed| typed.generation != bundle.generation);

        let mut markdown =
            format!("**Path**: `{fully_qualified_path}`\n\n**Namespace**: `{namespace}`");
        if let Some(type_info) = value_type_info {
            markdown.push_str(&format!("\n\n**Type**: `{type_info}`"));
        } else if matches!(
            symbol.namespace,
            halcyon_lib::ir::NameSpace::Term
                | halcyon_lib::ir::NameSpace::Constructor
                | halcyon_lib::ir::NameSpace::Wasm
        ) {
            markdown.push_str("\n\n**Type**: `_unavailable (typecheck pending)_`");
        }
        if symbol.namespace == halcyon_lib::ir::NameSpace::Type {
            if let Some(kind_info) = kind_info {
                markdown.push_str(&format!("\n\n**Kind**: `{kind_info}`"));
            } else {
                markdown.push_str("\n\n**Kind**: `_unavailable (typecheck pending)_`");
            }
        }
        if stale_note {
            markdown.push_str("\n\n_Using last completed typecheck snapshot._");
        }

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
            range: None,
        }))
    }

    fn goto_definition(
        &mut self,
        params: GotoDefinitionParams,
        connection: &Connection,
    ) -> Result<Option<GotoDefinitionResponse>, ResponseError> {
        let Some(path) = uri_to_path(&params.text_document_position_params.text_document.uri)
        else {
            return Ok(None);
        };

        let (root_path, symbol) = self.symbol_at_cursor(
            &path,
            params.text_document_position_params.position,
            connection,
        )?;
        let Some(bundle) = self.bundles.get(&root_path) else {
            return Ok(None);
        };
        let Some(name_index) = &bundle.frontend.name_index else {
            return Ok(None);
        };

        let locations = self.locations_for_spans(
            &bundle.frontend.source_files,
            name_index
                .definitions
                .get(&symbol)
                .into_iter()
                .flatten()
                .copied(),
        );
        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(GotoDefinitionResponse::Array(locations)))
        }
    }

    fn references(
        &mut self,
        params: ReferenceParams,
        connection: &Connection,
    ) -> Result<Option<Vec<Location>>, ResponseError> {
        let Some(path) = uri_to_path(&params.text_document_position.text_document.uri) else {
            return Ok(None);
        };

        let (root_path, symbol) =
            self.symbol_at_cursor(&path, params.text_document_position.position, connection)?;
        let Some(bundle) = self.bundles.get(&root_path) else {
            return Ok(None);
        };
        let Some(name_index) = &bundle.frontend.name_index else {
            return Ok(None);
        };

        let mut spans = Vec::new();
        if params.context.include_declaration
            && let Some(definitions) = name_index.definitions.get(&symbol)
        {
            spans.extend(definitions.iter().copied());
        }
        if let Some(usages) = name_index.usages.get(&symbol) {
            spans.extend(usages.iter().copied());
        }

        Ok(Some(
            self.locations_for_spans(&bundle.frontend.source_files, spans),
        ))
    }

    fn rename(
        &mut self,
        params: RenameParams,
        connection: &Connection,
    ) -> Result<Option<WorkspaceEdit>, ResponseError> {
        let Some(path) = uri_to_path(&params.text_document_position.text_document.uri) else {
            return Err(response_error("Rename only supports file:// URIs"));
        };

        let (root_path, symbol) =
            self.symbol_at_cursor(&path, params.text_document_position.position, connection)?;
        let Some(bundle) = self.bundles.get(&root_path) else {
            return Err(response_error("Bundle analysis is unavailable"));
        };
        // Cross-bundle renaming should never be allowed.
        // The rename operation is intentionally scoped to exactly one analyzed bundle.
        if symbol.path.major != bundle.frontend.bundle_name {
            return Err(response_error("Cross-bundle rename is not supported"));
        }
        let Some(name_index) = &bundle.frontend.name_index else {
            return Err(response_error(
                "Rename unavailable due to syntax/type errors in bundle",
            ));
        };

        let mut changes: HashMap<Uri, Vec<TextEdit>> = HashMap::new();
        for span in name_index.references(&symbol) {
            let Some(edit) =
                self.rename_edit_for_span(&bundle.frontend.source_files, span, &params.new_name)
            else {
                continue;
            };
            changes.entry(edit.0).or_default().push(edit.1);
        }

        if changes.is_empty() {
            return Err(response_error("No rename targets found"));
        }

        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        }))
    }

    fn rename_edit_for_span(
        &self,
        source_files: &[AnalysisSourceFile],
        span: halcyon_lib::Span,
        new_name: &str,
    ) -> Option<(Uri, TextEdit)> {
        let halcyon_lib::Span::Source {
            start: start_offset,
            width,
            file_id: Some(file_id),
        } = span
        else {
            return None;
        };

        let file = source_files.iter().find(|file| file.id == file_id)?;
        if !file.path.is_absolute() {
            return None;
        }
        let uri = path_to_uri(&file.path)?;

        let start = byte_offset_to_utf16_position(&file.source, start_offset);
        let end = byte_offset_to_utf16_position(&file.source, start_offset + width);
        let range = text_range(start, end);

        Some((
            uri,
            TextEdit {
                range,
                new_text: new_name.to_string(),
            },
        ))
    }

    fn naming_workspace_edit(
        &self,
        frontend: &FrontendBundleAnalysis,
        document_uri: &Uri,
        source: &str,
        file_id: usize,
        candidate: &NamingCandidate,
    ) -> Option<WorkspaceEdit> {
        let mut changes = HashMap::new();

        if let Some(name_index) = &frontend.name_index
            && let Some(symbol) = name_index.symbol_at(file_id, candidate.start)
            && symbol.path.major == frontend.bundle_name
        {
            for span in name_index.references(&symbol) {
                let Some((uri, edit)) =
                    self.rename_edit_for_span(&frontend.source_files, span, &candidate.replacement)
                else {
                    continue;
                };
                changes.entry(uri).or_insert_with(Vec::new).push(edit);
            }
        }

        if changes.is_empty() {
            let start = byte_offset_to_utf16_position(source, candidate.start);
            let end = byte_offset_to_utf16_position(source, candidate.start + candidate.width);
            changes.insert(
                document_uri.clone(),
                vec![TextEdit {
                    range: text_range(start, end),
                    new_text: candidate.replacement.clone(),
                }],
            );
        }

        Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        })
    }

    fn type_kind_for_symbol(
        &self,
        bundle: &BundleState,
        path: &halcyon_lib::ir::Path,
    ) -> Option<halcyon_lib::types::Kind> {
        bundle
            .typed
            .as_ref()
            .and_then(|typed| typed.analysis.symbols.type_definitions().get(path))
            .or_else(|| self.base_symbols.type_definitions().get(path))
            .map(type_definition_kind)
    }

    fn symbol_at_cursor(
        &mut self,
        path: &Path,
        position: Position,
        connection: &Connection,
    ) -> Result<(PathBuf, halcyon_lib::ir::ScopedPath), ResponseError> {
        let Some(root_path) = self
            .ensure_frontend_for_path(path, connection)
            .map_err(|error| response_error(error.to_string()))?
        else {
            return Err(response_error("No bundle.hc found for this file"));
        };

        let Some(bundle) = self.bundles.get(&root_path) else {
            return Err(response_error("Bundle analysis is unavailable"));
        };
        let Some(source) = self.source_for_path_from_frontend(path, &bundle.frontend) else {
            return Err(response_error("Could not resolve source text"));
        };
        let Some(symbol) = self.symbol_in_frontend(path, position, &bundle.frontend, &source)
        else {
            return Err(response_error("No symbol found at cursor"));
        };

        Ok((root_path, symbol))
    }

    fn symbol_in_frontend(
        &self,
        path: &Path,
        position: Position,
        frontend: &FrontendBundleAnalysis,
        source: &str,
    ) -> Option<halcyon_lib::ir::ScopedPath> {
        let name_index = frontend.name_index.as_ref()?;
        let file_id = frontend.file_id_for_path(path)?;
        let offset = utf16_position_to_byte_offset(source, position.line, position.character)?;
        name_index.symbol_at(file_id, offset)
    }

    fn locations_for_spans<I>(
        &self,
        source_files: &[AnalysisSourceFile],
        spans: I,
    ) -> Vec<Location>
    where
        I: IntoIterator<Item = halcyon_lib::Span>,
    {
        let mut seen = HashSet::new();
        let mut locations = Vec::new();

        for span in spans {
            let Some(location) = self.location_for_span(source_files, span) else {
                continue;
            };
            let key = format!(
                "{}:{}:{}:{}:{}",
                location.uri.as_str(),
                location.range.start.line,
                location.range.start.character,
                location.range.end.line,
                location.range.end.character
            );
            if seen.insert(key) {
                locations.push(location);
            }
        }

        locations
    }

    fn location_for_span(
        &self,
        source_files: &[AnalysisSourceFile],
        span: halcyon_lib::Span,
    ) -> Option<Location> {
        let halcyon_lib::Span::Source {
            start: start_offset,
            width,
            file_id: Some(file_id),
        } = span
        else {
            return None;
        };
        let file = source_files.iter().find(|file| file.id == file_id)?;
        if !file.path.is_absolute() {
            return None;
        }

        let uri = path_to_uri(&file.path)?;
        let start = byte_offset_to_utf16_position(&file.source, start_offset);
        let end = byte_offset_to_utf16_position(&file.source, start_offset + width);
        Some(Location {
            uri,
            range: text_range(start, end),
        })
    }

    fn source_for_path_from_frontend(
        &self,
        path: &Path,
        frontend: &FrontendBundleAnalysis,
    ) -> Option<String> {
        if let Some(document) = self.open_documents.get(&normalize_path(path)) {
            return Some(document.text.clone());
        }

        frontend
            .file_id_for_path(path)
            .and_then(|file_id| frontend.source_for_file_id(file_id).map(ToOwned::to_owned))
            .or_else(|| std::fs::read_to_string(path).ok())
    }

    fn ensure_frontend_for_path(
        &mut self,
        path: &Path,
        connection: &Connection,
    ) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
        let Some(root_path) = find_nearest_bundle_root(path) else {
            return Ok(None);
        };
        let root_path = normalize_path(&root_path);
        if !self.bundles.contains_key(&root_path) {
            self.reanalyze_root(&root_path, connection)?;
        }
        Ok(Some(root_path))
    }

    fn reanalyze_path(
        &mut self,
        path: &Path,
        connection: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(root_path) = find_nearest_bundle_root(path) else {
            return Ok(());
        };
        self.reanalyze_root(&normalize_path(&root_path), connection)
    }

    fn reanalyze_root(
        &mut self,
        root_path: &Path,
        connection: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let open_document_sources = self
            .open_documents
            .iter()
            .map(|(path, document)| (path.clone(), document.text.clone()))
            .collect::<HashMap<_, _>>();
        let mut resolve_source = |path: &Path| {
            let normalized = normalize_path(path);
            open_document_sources
                .get(&normalized)
                .cloned()
                .or_else(|| std::fs::read_to_string(path).ok())
                .or_else(|| std::fs::read_to_string(normalized).ok())
        };

        match analyze_bundle_frontend_with_symbols_and_resolver(
            root_path,
            &self.base_symbols,
            &mut resolve_source,
        ) {
            Ok(frontend) => {
                let previous = self.bundles.remove(root_path);
                let previous_uris = previous
                    .as_ref()
                    .map(|bundle| bundle.published_uris.clone())
                    .unwrap_or_default();
                let generation = previous
                    .as_ref()
                    .map(|bundle| bundle.generation + 1)
                    .unwrap_or(1);
                let previous_typed = previous.and_then(|bundle| bundle.typed);

                let open_document_versions = self
                    .open_documents
                    .iter()
                    .map(|(path, document)| (path.clone(), document.version))
                    .collect::<HashMap<_, _>>();
                let published_uris = publish_bundle_diagnostics(
                    connection,
                    &frontend.source_files,
                    &frontend.diagnostics,
                    &open_document_versions,
                    &previous_uris,
                )?;

                self.bundles.insert(
                    root_path.to_path_buf(),
                    BundleState {
                        frontend: frontend.clone(),
                        typed: previous_typed,
                        generation,
                        published_uris,
                    },
                );

                if frontend.module.is_some() {
                    self.spawn_typecheck(
                        root_path.to_path_buf(),
                        generation,
                        open_document_sources,
                    );
                }
            }
            Err(error) => {
                eprintln!("{}", error);
                if let Some(previous) = self.bundles.remove(root_path) {
                    for uri in previous.published_uris {
                        publish_diagnostics(connection, uri, Vec::new(), None)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn spawn_typecheck(
        &self,
        root_path: PathBuf,
        generation: u64,
        open_document_sources: HashMap<PathBuf, String>,
    ) {
        let sender = self.typecheck_result_sender.clone();
        let base_symbols = self.base_symbols.clone();

        std::thread::spawn(move || {
            let mut resolve_source = |path: &Path| {
                let normalized = normalize_path(path);
                open_document_sources
                    .get(&normalized)
                    .cloned()
                    .or_else(|| std::fs::read_to_string(path).ok())
                    .or_else(|| std::fs::read_to_string(normalized).ok())
            };
            let analysis = analyze_bundle_with_symbols_and_resolver(
                &root_path,
                &base_symbols,
                &mut resolve_source,
            )
            .map_err(|error| error.to_string());
            let _ = sender.send(TypecheckResult {
                root_path,
                generation,
                analysis,
            });
        });
    }

    fn drain_typecheck_results(&mut self) {
        while let Ok(result) = self.typecheck_result_receiver.try_recv() {
            let Some(bundle) = self.bundles.get_mut(&result.root_path) else {
                continue;
            };
            if bundle.generation != result.generation {
                continue;
            }
            match result.analysis {
                Ok(analysis) => {
                    bundle.typed = Some(TypedSnapshot {
                        generation: result.generation,
                        analysis,
                    });
                }
                Err(error) => {
                    eprintln!("{error}");
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NamingStyle {
    Snake,
    Kebab,
    Pascal,
}

impl NamingStyle {
    fn label(self) -> &'static str {
        match self {
            Self::Snake => "snake_case",
            Self::Kebab => "kebab-case",
            Self::Pascal => "PascalCase",
        }
    }

    fn normalize(
        self,
        value: &str,
    ) -> String {
        match self {
            Self::Snake => value.to_snake_case(),
            Self::Kebab => value.to_kebab_case(),
            Self::Pascal => value.to_pascal_case(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NamingCandidate {
    subject: &'static str,
    style: NamingStyle,
    current_name: String,
    replacement: String,
    start: usize,
    width: usize,
}

fn quickfix_requested(params: &CodeActionParams) -> bool {
    params.context.only.as_ref().is_none_or(|kinds| {
        kinds
            .iter()
            .any(|kind| kind.as_str().starts_with(CodeActionKind::QUICKFIX.as_str()))
    })
}

fn parse_source_file_for_naming_actions(
    file_name: &str,
    source: &str,
) -> Option<ast::SourceFile> {
    let mut logger = Logger::new();
    let mut file_logger = logger.new_file(file_name, source);
    parse::parse(source, &mut file_logger)
}

fn collect_naming_candidates(source_file: ast::SourceFile) -> Vec<NamingCandidate> {
    let mut candidates = Vec::new();
    for statement in source_file.statements() {
        collect_statement_naming_candidates(statement, &mut candidates);
    }
    candidates
}

fn collect_statement_naming_candidates(
    statement: ast::Statement,
    candidates: &mut Vec<NamingCandidate>,
) {
    match statement {
        ast::Statement::Bundle(_) | ast::Statement::Import(_) | ast::Statement::Use(_) => {}
        ast::Statement::Let(let_statement) => {
            if let_statement.is_pattern_alias() {
                if let Some(alias_name) = let_statement.alias_name_spanned()
                    && let Some(candidate) = naming_candidate_from_name(
                        "Constructor alias",
                        NamingStyle::Pascal,
                        alias_name,
                    )
                {
                    candidates.push(candidate);
                }
            } else if let Some(pattern) = let_statement.pattern() {
                collect_pattern_naming_candidates(pattern, candidates);
            }
            if let Some(value) = let_statement.value() {
                collect_expr_naming_candidates(value, candidates);
            }
        }
        ast::Statement::Do(do_statement) => {
            if let Some(value) = do_statement.value() {
                collect_expr_naming_candidates(value, candidates);
            }
        }
        ast::Statement::Type(type_statement) => {
            if let Some(type_name) = type_statement.name_text_spanned()
                && let Some(candidate) =
                    naming_candidate_from_name("Type", NamingStyle::Pascal, type_name)
            {
                candidates.push(candidate);
            }
            if let Some(type_def) = type_statement.type_def() {
                collect_type_def_naming_candidates(type_def, candidates);
            }
        }
        ast::Statement::Trait(trait_statement) => {
            if let Some(trait_name) = trait_statement.name_text_spanned()
                && let Some(candidate) =
                    naming_candidate_from_name("Trait", NamingStyle::Pascal, trait_name)
            {
                candidates.push(candidate);
            }
            if !trait_statement.is_alias() {
                for method in trait_statement.methods() {
                    if let Some(method_name) = method.name_text_spanned()
                        && let Some(candidate) = naming_candidate_from_name(
                            "Trait item",
                            NamingStyle::Snake,
                            method_name,
                        )
                    {
                        candidates.push(candidate);
                    }
                }
            }
        }
        ast::Statement::Impl(impl_statement) => {
            for method in impl_statement.methods() {
                if let Some(method_name) = method.name_text_spanned()
                    && let Some(candidate) =
                        naming_candidate_from_name("Trait item", NamingStyle::Snake, method_name)
                {
                    candidates.push(candidate);
                }
            }
        }
        ast::Statement::Module(module_node) => {
            if let Some(module_name) = module_node.name_text_spanned()
                && let Some(candidate) =
                    naming_candidate_from_name("Module", NamingStyle::Kebab, module_name)
            {
                candidates.push(candidate);
            }
            for nested_statement in module_node.statements() {
                collect_statement_naming_candidates(nested_statement, candidates);
            }
        }
        ast::Statement::Wasm(_) => {}
    }
}

fn collect_type_def_naming_candidates(
    type_def: ast::TypeDef,
    candidates: &mut Vec<NamingCandidate>,
) {
    match type_def {
        ast::TypeDef::Struct(struct_def) => {
            for field in struct_def.fields() {
                if let Some(field_name) = field.name_text_spanned()
                    && let Some(candidate) =
                        naming_candidate_from_name("Struct field", NamingStyle::Snake, field_name)
                {
                    candidates.push(candidate);
                }
            }
        }
        ast::TypeDef::Sum(sum_def) => {
            for variant in sum_def.variants() {
                if let Some(variant_name) = variant.name_text_spanned()
                    && let Some(candidate) =
                        naming_candidate_from_name("Constructor", NamingStyle::Pascal, variant_name)
                {
                    candidates.push(candidate);
                }
            }
        }
        ast::TypeDef::Alias(_) => {}
    }
}

fn collect_expr_naming_candidates(
    expr: ast::Expr,
    candidates: &mut Vec<NamingCandidate>,
) {
    match expr {
        ast::Expr::Let(let_expr) => {
            if let Some(pattern) = let_expr.pattern() {
                collect_pattern_naming_candidates(pattern, candidates);
            }
            if let Some(value) = let_expr.value() {
                collect_expr_naming_candidates(value, candidates);
            }
            if let Some(body) = let_expr.body() {
                collect_expr_naming_candidates(body, candidates);
            }
        }
        ast::Expr::Use(use_expr) => {
            if let Some(body) = use_expr.body() {
                collect_expr_naming_candidates(body, candidates);
            }
        }
        ast::Expr::Fn(fn_expr) => {
            if let Some(body) = fn_expr.body() {
                collect_expr_naming_candidates(body, candidates);
            }
        }
        ast::Expr::FnShorthand(fn_shorthand_expr) => {
            for arm in fn_shorthand_expr.arms() {
                if let Some(pattern) = arm.pattern() {
                    collect_pattern_naming_candidates(pattern, candidates);
                }
                if let Some(body) = arm.body() {
                    collect_expr_naming_candidates(body, candidates);
                }
            }
        }
        ast::Expr::If(if_expr) => {
            if let Some(condition) = if_expr.condition() {
                collect_expr_naming_candidates(condition, candidates);
            }
            if let Some(then_branch) = if_expr.then_branch() {
                collect_expr_naming_candidates(then_branch, candidates);
            }
            if let Some(else_branch) = if_expr.else_branch() {
                collect_expr_naming_candidates(else_branch, candidates);
            }
        }
        ast::Expr::Match(match_expr) => {
            if let Some(scrutinee) = match_expr.scrutinee() {
                collect_expr_naming_candidates(scrutinee, candidates);
            }
            for arm in match_expr.arms() {
                if let Some(pattern) = arm.pattern() {
                    collect_pattern_naming_candidates(pattern, candidates);
                }
                if let Some(body) = arm.body() {
                    collect_expr_naming_candidates(body, candidates);
                }
            }
        }
        ast::Expr::InlineWasm(_) | ast::Expr::Literal(_) | ast::Expr::Unit(_) => {}
        ast::Expr::Binary(binary_expr) => {
            if let Some(lhs) = binary_expr.lhs() {
                collect_expr_naming_candidates(lhs, candidates);
            }
            if let Some(rhs) = binary_expr.rhs() {
                collect_expr_naming_candidates(rhs, candidates);
            }
        }
        ast::Expr::Unary(unary_expr) => {
            if let Some(operand) = unary_expr.operand() {
                collect_expr_naming_candidates(operand, candidates);
            }
        }
        ast::Expr::Call(call_expr) => {
            if let Some(callee) = call_expr.callee() {
                collect_expr_naming_candidates(callee, candidates);
            }
            if let Some(arg) = call_expr.arg() {
                collect_expr_naming_candidates(arg, candidates);
            }
        }
        ast::Expr::Field(field_expr) => {
            if let Some(base) = field_expr.base() {
                collect_expr_naming_candidates(base, candidates);
            }
        }
        ast::Expr::Paren(paren_expr) => {
            for inner in paren_expr.inner_exprs() {
                collect_expr_naming_candidates(inner, candidates);
            }
        }
        ast::Expr::Array(array_expr) => {
            for inner in array_expr.exprs() {
                collect_expr_naming_candidates(inner, candidates);
            }
            for splat in array_expr.splats() {
                if let Some(value) = splat.expr() {
                    collect_expr_naming_candidates(value, candidates);
                }
            }
        }
        ast::Expr::Struct(struct_expr) => {
            for field in struct_expr.fields() {
                if let Some(value) = field.value() {
                    collect_expr_naming_candidates(value, candidates);
                }
            }
        }
        ast::Expr::Ident(_) | ast::Expr::Path(_) => {}
    }
}

fn collect_pattern_naming_candidates(
    pattern: ast::Pattern,
    candidates: &mut Vec<NamingCandidate>,
) {
    match pattern {
        ast::Pattern::Ident(ident) => {
            let Some(name) = ident.name_text_spanned() else {
                return;
            };
            if name.inner == "_" {
                return;
            }
            if let Some(candidate) =
                naming_candidate_from_name("Let binding", NamingStyle::Snake, name)
            {
                candidates.push(candidate);
            }
        }
        ast::Pattern::Literal(_) | ast::Pattern::Unit(_) | ast::Pattern::Path(_) => {}
        ast::Pattern::Tuple(pat_tuple) => {
            for inner in pat_tuple.patterns() {
                collect_pattern_naming_candidates(inner, candidates);
            }
        }
        ast::Pattern::Array(pat_array) => {
            for inner in pat_array.patterns() {
                collect_pattern_naming_candidates(inner, candidates);
            }
            for rest in pat_array.rest_patterns() {
                if let Some(name) = rest.binding_name_spanned()
                    && let Some(candidate) =
                        naming_candidate_from_name("Let binding", NamingStyle::Snake, name)
                {
                    candidates.push(candidate);
                }
            }
        }
        ast::Pattern::Struct(pat_struct) => {
            for field in pat_struct.fields() {
                if let Some(field_name) = field.name_text_spanned()
                    && field.pattern().is_none()
                    && !pat_field_has_equals(&field)
                    && let Some(candidate) =
                        naming_candidate_from_name("Let binding", NamingStyle::Snake, field_name)
                {
                    candidates.push(candidate);
                }
                if let Some(inner) = field.pattern() {
                    collect_pattern_naming_candidates(inner, candidates);
                }
            }
        }
        ast::Pattern::Constructor(pat_constructor) => {
            if let Some(payload) = pat_constructor.payload() {
                collect_pattern_naming_candidates(payload, candidates);
            }
        }
        ast::Pattern::TypeHint(pat_type_hint) => {
            if let Some(inner) = pat_type_hint.pattern() {
                collect_pattern_naming_candidates(inner, candidates);
            }
        }
    }
}

fn pat_field_has_equals(field: &ast::PatField) -> bool {
    field
        .syntax()
        .children_with_tokens()
        .filter_map(|element| element.into_token())
        .any(|token| token.kind() == SyntaxKind::EQUAL)
}

fn naming_candidate_from_name(
    subject: &'static str,
    style: NamingStyle,
    name: Spanned<String>,
) -> Option<NamingCandidate> {
    if is_bracketed_operator_name(&name.inner) {
        return None;
    }

    let replacement = style.normalize(&name.inner);
    if replacement == name.inner {
        return None;
    }

    let Span::Source { start, width, .. } = name.span else {
        return None;
    };
    Some(NamingCandidate {
        subject,
        style,
        current_name: name.inner,
        replacement,
        start,
        width,
    })
}

fn naming_candidate_overlaps_range(
    candidate: &NamingCandidate,
    range_start: usize,
    range_end: usize,
) -> bool {
    let candidate_start = candidate.start;
    let candidate_end = candidate.start + candidate.width;
    if range_start == range_end {
        range_start >= candidate_start && range_start <= candidate_end
    } else {
        candidate_start < range_end && range_start < candidate_end
    }
}

fn is_bracketed_operator_name(name: &str) -> bool {
    name.starts_with('[') && name.ends_with(']')
}

fn type_definition_kind(
    definition: &halcyon_lib::types::TypeDefinition
) -> halcyon_lib::types::Kind {
    let mut parameter_kinds = definition.parameter_kinds.clone();
    while parameter_kinds.len() < definition.parameters {
        parameter_kinds.push(halcyon_lib::types::Kind::Type);
    }
    parameter_kinds.truncate(definition.parameters);
    halcyon_lib::types::Kind::from_parameter_kinds(&parameter_kinds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{
        CodeActionContext,
        Diagnostic,
    };

    fn naming_candidates_for_source(source: &str) -> Vec<NamingCandidate> {
        let Some(source_file) = parse_source_file_for_naming_actions("demo.hc", source) else {
            panic!("expected source to parse");
        };
        collect_naming_candidates(source_file)
    }

    #[test]
    fn type_definition_kind_normalizes_parameter_kinds_to_arity() {
        let kind = type_definition_kind(&halcyon_lib::types::TypeDefinition {
            parameters: 2,
            parameter_kinds: vec![halcyon_lib::types::Kind::Type],
            body: halcyon_lib::types::Type::Unit,
            kind: halcyon_lib::types::TypeDefinitionKind::Named,
        });

        assert_eq!(
            kind,
            halcyon_lib::types::Kind::from_parameter_kinds(&[
                halcyon_lib::types::Kind::Type,
                halcyon_lib::types::Kind::Type,
            ])
        );
    }

    #[test]
    fn type_definition_kind_preserves_higher_kinded_parameters() {
        let kind = type_definition_kind(&halcyon_lib::types::TypeDefinition {
            parameters: 1,
            parameter_kinds: vec![halcyon_lib::types::Kind::arrow(
                halcyon_lib::types::Kind::Type,
                halcyon_lib::types::Kind::Type,
            )],
            body: halcyon_lib::types::Type::Unit,
            kind: halcyon_lib::types::TypeDefinitionKind::Alias,
        });

        assert_eq!(
            kind,
            halcyon_lib::types::Kind::arrow(
                halcyon_lib::types::Kind::arrow(
                    halcyon_lib::types::Kind::Type,
                    halcyon_lib::types::Kind::Type,
                ),
                halcyon_lib::types::Kind::Type,
            )
        );
    }

    #[test]
    fn collect_naming_candidates_detects_wrong_case_names() {
        let source = "module Demo =\n\ttype my_type = { BadField: core::Integer }\n\ttype BadSum = | lower\n\ttrait my_trait : a =\n\t\tlet BadItem : a -> a\n\tend\n\timpl my_trait core::Integer =\n\t\tlet BadItem = fn x => x\n\tend\n\tlet BadLet = 1\nend\n";
        let candidates = naming_candidates_for_source(source);

        assert!(
            candidates.iter().any(|candidate| {
                candidate.current_name == "Demo" && candidate.replacement == "demo"
            }),
            "expected module kebab-case fix"
        );
        assert!(
            candidates.iter().any(|candidate| {
                candidate.current_name == "my_type" && candidate.replacement == "MyType"
            }),
            "expected type PascalCase fix"
        );
        assert!(
            candidates.iter().any(|candidate| {
                candidate.current_name == "BadField" && candidate.replacement == "bad_field"
            }),
            "expected struct field snake_case fix"
        );
        assert!(
            candidates.iter().any(|candidate| {
                candidate.current_name == "lower" && candidate.replacement == "Lower"
            }),
            "expected constructor PascalCase fix"
        );
        assert!(
            candidates.iter().any(|candidate| {
                candidate.current_name == "BadLet" && candidate.replacement == "bad_let"
            }),
            "expected let-binding snake_case fix"
        );
    }

    #[test]
    fn collect_naming_candidates_skips_bracketed_operator_names() {
        let source = "module demo =\n\tlet [+] = fn x y => x\nend\n";
        let candidates = naming_candidates_for_source(source);

        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.current_name != "[+]"),
            "bracketed operator names should not be linted"
        );
    }

    #[test]
    fn quickfix_requested_accepts_quickfix_and_subkinds() {
        let quickfix_params = CodeActionParams {
            text_document: lsp_types::TextDocumentIdentifier {
                uri: "file:///tmp/demo.hc".parse().expect("valid URI"),
            },
            range: lsp_types::Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
            context: CodeActionContext {
                diagnostics: Vec::<Diagnostic>::new(),
                only: Some(vec![CodeActionKind::QUICKFIX]),
                trigger_kind: None,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        assert!(quickfix_requested(&quickfix_params));

        let refactor_params = CodeActionParams {
            text_document: quickfix_params.text_document,
            range: quickfix_params.range,
            context: CodeActionContext {
                diagnostics: Vec::<Diagnostic>::new(),
                only: Some(vec![CodeActionKind::REFACTOR]),
                trigger_kind: None,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };
        assert!(!quickfix_requested(&refactor_params));
    }
}
