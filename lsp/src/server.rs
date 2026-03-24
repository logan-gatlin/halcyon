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
use std::time::Duration;

use crossbeam_channel::RecvTimeoutError;
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
    CompletionItem,
    CompletionList,
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
    completion_trigger_characters,
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

    let capabilities = serde_json::to_value(server_capabilities())?;
    let _init_params = connection.initialize(capabilities)?;

    let mut server = Server::new();

    loop {
        server.drain_typecheck_results(&connection)?;
        let message = match connection.receiver.recv_timeout(Duration::from_millis(50)) {
            Ok(message) => message,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };

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

fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(completion_trigger_characters()),
            ..CompletionOptions::default()
        }),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        hover_provider: Some(true.into()),
        rename_provider: Some(OneOf::Left(true)),
        definition_provider: Some(OneOf::Left(true)),
        references_provider: Some(OneOf::Left(true)),
        ..ServerCapabilities::default()
    }
}

fn completion_response(items: Vec<CompletionItem>) -> CompletionResponse {
    CompletionResponse::List(CompletionList {
        is_incomplete: true,
        items,
    })
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
        self.drain_typecheck_results(connection)?;

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
        self.drain_typecheck_results(connection)?;

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
            return Ok(Some(completion_response(Vec::new())));
        };
        let Some(root_path) = self
            .ensure_frontend_for_path(&path, connection)
            .map_err(|error| response_error(error.to_string()))?
        else {
            return Ok(Some(completion_response(Vec::new())));
        };

        let Some(bundle) = self.bundles.get(&root_path) else {
            return Ok(Some(completion_response(Vec::new())));
        };
        let Some(source) = self.source_for_path_from_frontend(&path, &bundle.frontend) else {
            return Ok(Some(completion_response(Vec::new())));
        };

        let context = completion_context_at(&source, params.text_document_position.position)
            .unwrap_or_default();
        let symbols = bundle
            .typed
            .as_ref()
            .map(|typed| &typed.analysis.symbols)
            .unwrap_or(&self.base_symbols);
        let items = completion_items(symbols, &context);
        Ok(Some(completion_response(items)))
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
        let Some(name_index) = &bundle.frontend.name_index else {
            return Err(response_error(
                "Rename unavailable due to syntax/type errors in bundle",
            ));
        };
        let rename_symbol = rename_symbol_for_trait_item(&bundle.frontend, &symbol);
        // Cross-bundle renaming should never be allowed.
        // The rename operation is intentionally scoped to exactly one analyzed bundle.
        if rename_symbol.path.major != bundle.frontend.bundle_name {
            return Err(response_error("Cross-bundle rename is not supported"));
        }

        let mut spans = name_index.references(&rename_symbol).into_vec();
        spans.extend(trait_method_impl_definition_spans(
            &bundle.frontend,
            name_index,
            &rename_symbol,
        ));

        let mut seen_spans = HashSet::new();
        let mut changes: HashMap<Uri, Vec<TextEdit>> = HashMap::new();
        for span in spans {
            if !seen_spans.insert(span) {
                continue;
            }
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
                        frontend.bundle_name.clone(),
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
        bundle_name: String,
        open_document_sources: HashMap<PathBuf, String>,
    ) {
        let sender = self.typecheck_result_sender.clone();
        let base_symbols = self.typecheck_base_symbols_for_bundle(&bundle_name);

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

    fn typecheck_base_symbols_for_bundle(
        &self,
        bundle_name: &str,
    ) -> halcyon_lib::types::SymbolTable {
        if bundle_name != halcyon_lib::CORE_MODULE_NAME {
            return self.base_symbols.clone();
        }

        let core_primitives = core_primitive_paths();
        let mut symbols = halcyon_lib::types::SymbolTable::new();
        for (path, definition) in self.base_symbols.type_definitions() {
            if core_primitives.contains(path) {
                symbols.insert_type(path.clone(), definition.clone());
            }
        }
        symbols
    }

    fn drain_typecheck_results(
        &mut self,
        connection: &Connection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let open_document_versions = self
            .open_documents
            .iter()
            .map(|(path, document)| (path.clone(), document.version))
            .collect::<HashMap<_, _>>();

        while let Ok(result) = self.typecheck_result_receiver.try_recv() {
            let Some(bundle) = self.bundles.get_mut(&result.root_path) else {
                continue;
            };
            if bundle.generation != result.generation {
                continue;
            }
            match result.analysis {
                Ok(analysis) => {
                    let published_uris = publish_bundle_diagnostics(
                        connection,
                        &analysis.source_files,
                        &analysis.diagnostics,
                        &open_document_versions,
                        &bundle.published_uris,
                    )?;
                    bundle.typed = Some(TypedSnapshot {
                        generation: result.generation,
                        analysis,
                    });
                    bundle.published_uris = published_uris;
                }
                Err(error) => {
                    eprintln!("{error}");
                }
            }
        }

        Ok(())
    }
}

fn rename_symbol_for_trait_item(
    frontend: &FrontendBundleAnalysis,
    symbol: &halcyon_lib::ir::ScopedPath,
) -> halcyon_lib::ir::ScopedPath {
    if symbol.namespace != halcyon_lib::ir::NameSpace::Term {
        return symbol.clone();
    }
    let Some(module) = &frontend.module else {
        return symbol.clone();
    };

    module
        .statements
        .iter()
        .find_map(|statement| {
            let halcyon_lib::ir::Statement::Impl { methods, .. } = statement else {
                return None;
            };
            methods.iter().find_map(|method| {
                if method.impl_path != symbol.path {
                    return None;
                }
                if !is_declared_trait_method(frontend, &method.trait_method) {
                    return None;
                }
                Some(halcyon_lib::ir::ScopedPath {
                    path: method.trait_method.clone(),
                    namespace: halcyon_lib::ir::NameSpace::Term,
                })
            })
        })
        .unwrap_or_else(|| symbol.clone())
}

fn core_primitive_paths() -> HashSet<halcyon_lib::ir::Path> {
    [
        "Unit", "Integer", "Real", "Boolean", "String", "Glyph", "Array", "Fn",
    ]
    .into_iter()
    .map(halcyon_lib::ir::Path::core)
    .collect()
}

fn trait_method_impl_definition_spans(
    frontend: &FrontendBundleAnalysis,
    name_index: &halcyon_lib::ir::NameIndex,
    symbol: &halcyon_lib::ir::ScopedPath,
) -> Vec<halcyon_lib::Span> {
    if symbol.namespace != halcyon_lib::ir::NameSpace::Term {
        return Vec::new();
    }
    if !is_declared_trait_method(frontend, &symbol.path) {
        return Vec::new();
    }

    let Some(module) = &frontend.module else {
        return Vec::new();
    };

    module
        .statements
        .iter()
        .filter_map(|statement| {
            let halcyon_lib::ir::Statement::Impl { methods, .. } = statement else {
                return None;
            };
            Some(methods.as_ref())
        })
        .flatten()
        .filter(|method| method.trait_method == symbol.path)
        .filter_map(|method| {
            let impl_symbol = halcyon_lib::ir::ScopedPath {
                path: method.impl_path.clone(),
                namespace: halcyon_lib::ir::NameSpace::Term,
            };
            name_index.definitions.get(&impl_symbol)
        })
        .flat_map(|spans| spans.iter().copied())
        .collect()
}

fn is_declared_trait_method(
    frontend: &FrontendBundleAnalysis,
    trait_method: &halcyon_lib::ir::Path,
) -> bool {
    frontend.module.as_ref().is_some_and(|module| {
        module.statements.iter().any(|statement| {
            let halcyon_lib::ir::Statement::Trait { methods, .. } = statement else {
                return false;
            };
            methods.iter().any(|method| method.path == *trait_method)
        })
    })
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
    use lsp_types::notification::PublishDiagnostics;
    use lsp_types::{
        CodeActionContext,
        Diagnostic,
        PublishDiagnosticsParams,
    };

    fn placeholder_type_expr() -> halcyon_lib::ir::TypeExpr {
        halcyon_lib::ir::TypeExpr {
            comments: String::new(),
            kind: halcyon_lib::ir::TypeExprKind::Placeholder,
            span: Span::Generated,
        }
    }

    fn frontend_with_module(module: halcyon_lib::ir::Module<()>) -> FrontendBundleAnalysis {
        FrontendBundleAnalysis {
            root_path: std::path::PathBuf::from("/tmp/bundle.hc"),
            bundle_name: "demo".to_string(),
            source_files: Vec::new().into_boxed_slice(),
            diagnostics: Vec::new().into_boxed_slice(),
            name_index: None,
            module: Some(module),
        }
    }

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

    #[test]
    fn server_capabilities_advertise_aggressive_completion_triggers() {
        let capabilities = server_capabilities();
        let trigger_characters = capabilities
            .completion_provider
            .and_then(|completion| completion.trigger_characters)
            .expect("completion trigger characters should be configured");

        assert_eq!(trigger_characters, completion_trigger_characters());
    }

    #[test]
    fn completion_response_marks_completion_list_incomplete() {
        let response = completion_response(Vec::new());
        let CompletionResponse::List(list) = response else {
            panic!("completion response should use completion list")
        };

        assert!(list.is_incomplete);
    }

    #[test]
    fn typecheck_base_symbols_for_core_bundle_only_keep_core_primitives() {
        let server = Server::new();

        let symbols = server.typecheck_base_symbols_for_bundle(halcyon_lib::CORE_MODULE_NAME);

        assert!(symbols.terms().is_empty());
        assert!(symbols.trait_defs().is_empty());
        assert!(symbols.trait_impls().is_empty());
        let type_paths = symbols
            .type_definitions()
            .keys()
            .cloned()
            .collect::<HashSet<_>>();
        assert_eq!(type_paths, core_primitive_paths());
    }

    #[test]
    fn typecheck_base_symbols_for_non_core_bundle_reuse_full_base_symbols() {
        let server = Server::new();

        let symbols = server.typecheck_base_symbols_for_bundle("demo");

        assert_eq!(symbols.terms().len(), server.base_symbols.terms().len());
        assert_eq!(
            symbols.type_definitions().len(),
            server.base_symbols.type_definitions().len()
        );
        assert_eq!(
            symbols.trait_defs().len(),
            server.base_symbols.trait_defs().len()
        );
        assert_eq!(
            symbols.trait_impls().len(),
            server.base_symbols.trait_impls().len()
        );
    }

    #[test]
    fn rename_symbol_for_trait_item_maps_impl_method_to_trait_method() {
        let trait_path = halcyon_lib::ir::Path::new("demo", "Alternative");
        let trait_method = halcyon_lib::ir::Path::new("demo", "or_with");
        let impl_method = halcyon_lib::ir::Path::new("demo", "or_with#0");
        let module = halcyon_lib::ir::Module {
            name: "demo".to_string(),
            statements: vec![
                halcyon_lib::ir::Statement::Trait {
                    comments: String::new(),
                    path: trait_path.clone(),
                    parameters: Vec::new().into_boxed_slice(),
                    methods: vec![halcyon_lib::ir::TraitMethodDecl {
                        path: trait_method.clone(),
                        type_expr: placeholder_type_expr(),
                        span: Span::Generated,
                    }]
                    .into_boxed_slice(),
                },
                halcyon_lib::ir::Statement::Impl {
                    comments: String::new(),
                    trait_path,
                    arguments: Vec::new().into_boxed_slice(),
                    methods: vec![halcyon_lib::ir::ImplMethod {
                        trait_method: trait_method.clone(),
                        impl_path: impl_method.clone(),
                        value: halcyon_lib::ir::Term::unit(),
                        span: Span::Generated,
                    }]
                    .into_boxed_slice(),
                },
            ]
            .into_boxed_slice(),
        };
        let frontend = frontend_with_module(module);
        let symbol = halcyon_lib::ir::ScopedPath {
            path: impl_method,
            namespace: halcyon_lib::ir::NameSpace::Term,
        };

        let mapped = rename_symbol_for_trait_item(&frontend, &symbol);
        assert_eq!(mapped.path, trait_method);
        assert_eq!(mapped.namespace, halcyon_lib::ir::NameSpace::Term);
    }

    #[test]
    fn trait_method_impl_definition_spans_include_impl_item_spans() {
        let trait_path = halcyon_lib::ir::Path::new("demo", "Alternative");
        let trait_method = halcyon_lib::ir::Path::new("demo", "or_with");
        let impl_method = halcyon_lib::ir::Path::new("demo", "or_with#0");
        let module = halcyon_lib::ir::Module {
            name: "demo".to_string(),
            statements: vec![
                halcyon_lib::ir::Statement::Trait {
                    comments: String::new(),
                    path: trait_path.clone(),
                    parameters: Vec::new().into_boxed_slice(),
                    methods: vec![halcyon_lib::ir::TraitMethodDecl {
                        path: trait_method.clone(),
                        type_expr: placeholder_type_expr(),
                        span: Span::Generated,
                    }]
                    .into_boxed_slice(),
                },
                halcyon_lib::ir::Statement::Impl {
                    comments: String::new(),
                    trait_path,
                    arguments: Vec::new().into_boxed_slice(),
                    methods: vec![halcyon_lib::ir::ImplMethod {
                        trait_method: trait_method.clone(),
                        impl_path: impl_method.clone(),
                        value: halcyon_lib::ir::Term::unit(),
                        span: Span::Generated,
                    }]
                    .into_boxed_slice(),
                },
            ]
            .into_boxed_slice(),
        };
        let frontend = frontend_with_module(module);

        let mut definitions = HashMap::new();
        let impl_span = Span::Source {
            start: 12,
            width: 7,
            file_id: Some(1),
        };
        definitions.insert(
            halcyon_lib::ir::ScopedPath {
                path: impl_method,
                namespace: halcyon_lib::ir::NameSpace::Term,
            },
            vec![impl_span].into_boxed_slice(),
        );
        let name_index = halcyon_lib::ir::NameIndex {
            definitions,
            usages: HashMap::new(),
        };
        let symbol = halcyon_lib::ir::ScopedPath {
            path: trait_method,
            namespace: halcyon_lib::ir::NameSpace::Term,
        };

        let spans = trait_method_impl_definition_spans(&frontend, &name_index, &symbol);
        assert_eq!(spans, vec![impl_span]);
    }

    #[test]
    fn trait_method_impl_definition_spans_collect_across_impl_files() {
        let trait_path = halcyon_lib::ir::Path::new("demo", "Alternative");
        let trait_method = halcyon_lib::ir::Path::new("demo", "or_with");
        let impl_method_a = halcyon_lib::ir::Path::new("demo", "or_with#0");
        let impl_method_b = halcyon_lib::ir::Path::new("demo", "or_with#1");
        let module = halcyon_lib::ir::Module {
            name: "demo".to_string(),
            statements: vec![
                halcyon_lib::ir::Statement::Trait {
                    comments: String::new(),
                    path: trait_path.clone(),
                    parameters: Vec::new().into_boxed_slice(),
                    methods: vec![halcyon_lib::ir::TraitMethodDecl {
                        path: trait_method.clone(),
                        type_expr: placeholder_type_expr(),
                        span: Span::Generated,
                    }]
                    .into_boxed_slice(),
                },
                halcyon_lib::ir::Statement::Impl {
                    comments: String::new(),
                    trait_path: trait_path.clone(),
                    arguments: Vec::new().into_boxed_slice(),
                    methods: vec![halcyon_lib::ir::ImplMethod {
                        trait_method: trait_method.clone(),
                        impl_path: impl_method_a.clone(),
                        value: halcyon_lib::ir::Term::unit(),
                        span: Span::Generated,
                    }]
                    .into_boxed_slice(),
                },
                halcyon_lib::ir::Statement::Impl {
                    comments: String::new(),
                    trait_path,
                    arguments: Vec::new().into_boxed_slice(),
                    methods: vec![halcyon_lib::ir::ImplMethod {
                        trait_method: trait_method.clone(),
                        impl_path: impl_method_b.clone(),
                        value: halcyon_lib::ir::Term::unit(),
                        span: Span::Generated,
                    }]
                    .into_boxed_slice(),
                },
            ]
            .into_boxed_slice(),
        };
        let frontend = frontend_with_module(module);

        let span_a = Span::Source {
            start: 4,
            width: 7,
            file_id: Some(1),
        };
        let span_b = Span::Source {
            start: 9,
            width: 7,
            file_id: Some(2),
        };
        let name_index = halcyon_lib::ir::NameIndex {
            definitions: HashMap::from([
                (
                    halcyon_lib::ir::ScopedPath {
                        path: impl_method_a,
                        namespace: halcyon_lib::ir::NameSpace::Term,
                    },
                    vec![span_a].into_boxed_slice(),
                ),
                (
                    halcyon_lib::ir::ScopedPath {
                        path: impl_method_b,
                        namespace: halcyon_lib::ir::NameSpace::Term,
                    },
                    vec![span_b].into_boxed_slice(),
                ),
            ]),
            usages: HashMap::new(),
        };
        let symbol = halcyon_lib::ir::ScopedPath {
            path: trait_method,
            namespace: halcyon_lib::ir::NameSpace::Term,
        };

        let spans = trait_method_impl_definition_spans(&frontend, &name_index, &symbol);
        assert_eq!(spans, vec![span_a, span_b]);
    }

    #[test]
    fn rename_symbol_for_trait_item_skips_external_trait_impl_methods() {
        let trait_method = halcyon_lib::ir::Path::new("core", "hkt::flat_map");
        let impl_method = halcyon_lib::ir::Path::new("demo", "flat_map#0");
        let module = halcyon_lib::ir::Module {
            name: "demo".to_string(),
            statements: vec![halcyon_lib::ir::Statement::Impl {
                comments: String::new(),
                trait_path: halcyon_lib::ir::Path::new("core", "hkt::Monad"),
                arguments: Vec::new().into_boxed_slice(),
                methods: vec![halcyon_lib::ir::ImplMethod {
                    trait_method,
                    impl_path: impl_method.clone(),
                    value: halcyon_lib::ir::Term::unit(),
                    span: Span::Generated,
                }]
                .into_boxed_slice(),
            }]
            .into_boxed_slice(),
        };
        let frontend = frontend_with_module(module);
        let symbol = halcyon_lib::ir::ScopedPath {
            path: impl_method,
            namespace: halcyon_lib::ir::NameSpace::Term,
        };

        let mapped = rename_symbol_for_trait_item(&frontend, &symbol);
        assert_eq!(mapped.path, symbol.path);
    }

    #[test]
    fn drain_typecheck_results_publishes_typed_diagnostics() {
        let (server_connection, client_connection) = Connection::memory();
        let mut server = Server::new();

        let root_path = std::path::PathBuf::from("/tmp/demo/bundle.hc");
        let source_path = std::path::PathBuf::from("/tmp/demo/opt.hc");
        let source = "let value = unknown\n".to_string();
        let source_files = vec![AnalysisSourceFile {
            id: 1,
            path: source_path.clone(),
            source: source.clone(),
        }]
        .into_boxed_slice();

        server.bundles.insert(
            root_path.clone(),
            BundleState {
                frontend: FrontendBundleAnalysis {
                    root_path: root_path.clone(),
                    bundle_name: "demo".to_string(),
                    source_files: source_files.clone(),
                    diagnostics: Vec::new().into_boxed_slice(),
                    name_index: None,
                    module: None,
                },
                typed: None,
                generation: 1,
                published_uris: HashSet::new(),
            },
        );

        let diagnostic = halcyon_lib::SerializedDiagnostic {
            severity: "error".to_string(),
            code: None,
            message: "Typed failure".to_string(),
            labels: vec![halcyon_lib::SerializedDiagnosticLabel {
                style: "primary".to_string(),
                file_name: source_path.to_string_lossy().to_string(),
                message: "Unknown value".to_string(),
                range_start: 12,
                range_end: 19,
                start: halcyon_lib::SerializedDiagnosticLocation {
                    line: 1,
                    column: 13,
                },
                end: halcyon_lib::SerializedDiagnosticLocation {
                    line: 1,
                    column: 19,
                },
            }],
            notes: Vec::new(),
        };
        let typed_analysis = BundleAnalysis {
            root_path: root_path.clone(),
            bundle_name: "demo".to_string(),
            source_files,
            diagnostics: vec![diagnostic].into_boxed_slice(),
            symbols: server.base_symbols.clone(),
            name_index: None,
        };

        server
            .typecheck_result_sender
            .send(TypecheckResult {
                root_path: root_path.clone(),
                generation: 1,
                analysis: Ok(typed_analysis),
            })
            .expect("should queue typecheck result");
        server
            .drain_typecheck_results(&server_connection)
            .expect("draining typecheck results should publish diagnostics");

        let notification = client_connection
            .receiver
            .try_iter()
            .find_map(|message| {
                match message {
                    Message::Notification(notification)
                        if notification.method == PublishDiagnostics::METHOD =>
                    {
                        Some(notification)
                    }
                    _ => None,
                }
            })
            .expect("typed diagnostics should be published");
        let params: PublishDiagnosticsParams = serde_json::from_value(notification.params)
            .expect("publishDiagnostics payload should deserialize");

        assert_eq!(params.diagnostics.len(), 1);
        assert!(params.diagnostics[0].message.contains("Typed failure"));
        assert!(
            server
                .bundles
                .get(&root_path)
                .and_then(|bundle| bundle.typed.as_ref())
                .is_some(),
            "typed snapshot should be retained after publishing diagnostics"
        );
    }
}
