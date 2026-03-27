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
    RequestId,
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
use serde::de::DeserializeOwned;

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

fn parse_request_or_invalid_response<T>(
    request: Request,
    connection: &Connection,
) -> Result<Option<(RequestId, T)>, Box<dyn std::error::Error>>
where
    T: DeserializeOwned,
{
    let method = request.method.clone();
    let request_id = request.id.clone();
    match parse_request::<T>(request) {
        Ok(parsed) => Ok(Some(parsed)),
        Err(error) => {
            send_response::<serde_json::Value>(
                request_id,
                Err(response_error(format!(
                    "Invalid params for `{method}`: {error}"
                ))),
                connection,
            )?;
            Ok(None)
        }
    }
}

fn parse_notification_or_skip<T>(notification: Notification) -> Option<T>
where
    T: DeserializeOwned,
{
    let method = notification.method.clone();
    match parse_notification::<T>(notification) {
        Ok(params) => Some(params),
        Err(error) => {
            eprintln!("Ignoring malformed notification `{method}`: {error}");
            None
        }
    }
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
                if let Some((id, params)) =
                    parse_request_or_invalid_response::<CompletionParams>(request, connection)?
                {
                    let result = self.completion(params, connection);
                    send_response(id, result, connection)?;
                }
            }
            CodeActionRequest::METHOD => {
                if let Some((id, params)) =
                    parse_request_or_invalid_response::<CodeActionParams>(request, connection)?
                {
                    let result = self.code_action(params, connection);
                    send_response(id, result, connection)?;
                }
            }
            HoverRequest::METHOD => {
                if let Some((id, params)) =
                    parse_request_or_invalid_response::<HoverParams>(request, connection)?
                {
                    let result = self.hover(params, connection);
                    send_response(id, result, connection)?;
                }
            }
            GotoDefinition::METHOD => {
                if let Some((id, params)) =
                    parse_request_or_invalid_response::<GotoDefinitionParams>(request, connection)?
                {
                    let result = self.goto_definition(params, connection);
                    send_response(id, result, connection)?;
                }
            }
            References::METHOD => {
                if let Some((id, params)) =
                    parse_request_or_invalid_response::<ReferenceParams>(request, connection)?
                {
                    let result = self.references(params, connection);
                    send_response(id, result, connection)?;
                }
            }
            Rename::METHOD => {
                if let Some((id, params)) =
                    parse_request_or_invalid_response::<RenameParams>(request, connection)?
                {
                    let result = self.rename(params, connection);
                    send_response(id, result, connection)?;
                }
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
                if let Some(params) =
                    parse_notification_or_skip::<DidOpenTextDocumentParams>(notification)
                {
                    self.did_open(params, connection)?;
                }
            }
            DidChangeTextDocument::METHOD => {
                if let Some(params) =
                    parse_notification_or_skip::<DidChangeTextDocumentParams>(notification)
                {
                    self.did_change(params, connection)?;
                }
            }
            DidCloseTextDocument::METHOD => {
                if let Some(params) =
                    parse_notification_or_skip::<DidCloseTextDocumentParams>(notification)
                {
                    self.did_close(params, connection)?;
                }
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
        let doc_comment = hover_doc_comment_for_symbol(&bundle.frontend, &symbol);

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
        if let Some(doc_comment) = doc_comment {
            markdown.push_str("\n\n**Docs**\n\n");
            markdown.push_str(&doc_comment);
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

fn hover_doc_comment_for_symbol(
    frontend: &FrontendBundleAnalysis,
    symbol: &halcyon_lib::ir::ScopedPath,
) -> Option<String> {
    let module = frontend.module.as_ref()?;
    module
        .statements
        .iter()
        .find_map(|statement| statement_doc_comment(statement, symbol))
}

fn statement_doc_comment(
    statement: &halcyon_lib::ir::Statement<()>,
    symbol: &halcyon_lib::ir::ScopedPath,
) -> Option<String> {
    match statement {
        halcyon_lib::ir::Statement::Term(term) => {
            let halcyon_lib::ir::TermKind::Let {
                assignee,
                scope: halcyon_lib::ir::ScopeKind::Global,
                ..
            } = &term.kind
            else {
                return None;
            };
            if symbol.namespace != halcyon_lib::ir::NameSpace::Term {
                return None;
            }
            let mut paths = Vec::new();
            collect_pattern_paths_for_docs(assignee, &mut paths);
            if paths.into_iter().any(|path| path == symbol.path) {
                normalized_doc_comment(&term.comments)
            } else {
                None
            }
        }
        halcyon_lib::ir::Statement::ConstructorAlias { comments, path, .. } => {
            if path != &symbol.path {
                return None;
            }
            if !matches!(
                symbol.namespace,
                halcyon_lib::ir::NameSpace::Term | halcyon_lib::ir::NameSpace::Constructor
            ) {
                return None;
            }
            normalized_doc_comment(comments)
        }
        halcyon_lib::ir::Statement::Type {
            comments,
            path,
            def,
            ..
        } => {
            if symbol.namespace == halcyon_lib::ir::NameSpace::Type && path == &symbol.path {
                return normalized_doc_comment(comments);
            }
            if !matches!(
                symbol.namespace,
                halcyon_lib::ir::NameSpace::Term | halcyon_lib::ir::NameSpace::Constructor
            ) {
                return None;
            }
            let matches_constructor = match def.kind() {
                halcyon_lib::ir::TypeDefKind::Struct(_) | halcyon_lib::ir::TypeDefKind::Expr(_) => {
                    symbol.path == *path
                }
                halcyon_lib::ir::TypeDefKind::Sum(variants) => {
                    variants
                        .keys()
                        .any(|variant| symbol.path == path.sibling(variant))
                }
            };
            if matches_constructor {
                normalized_doc_comment(comments)
            } else {
                None
            }
        }
        halcyon_lib::ir::Statement::Trait {
            comments,
            path,
            associated_types,
            methods,
            ..
        } => {
            if symbol.namespace == halcyon_lib::ir::NameSpace::Trait && path == &symbol.path {
                return normalized_doc_comment(comments);
            }
            if symbol.namespace == halcyon_lib::ir::NameSpace::Type
                && associated_types.iter().any(|item| item.path == symbol.path)
            {
                return normalized_doc_comment(comments);
            }
            if symbol.namespace == halcyon_lib::ir::NameSpace::Term
                && methods.iter().any(|method| method.path == symbol.path)
            {
                return normalized_doc_comment(comments);
            }
            None
        }
        halcyon_lib::ir::Statement::TraitAlias { comments, path, .. } => {
            if symbol.namespace == halcyon_lib::ir::NameSpace::Trait && path == &symbol.path {
                normalized_doc_comment(comments)
            } else {
                None
            }
        }
        halcyon_lib::ir::Statement::Impl {
            comments, methods, ..
        } => {
            if symbol.namespace == halcyon_lib::ir::NameSpace::Term
                && methods.iter().any(|method| method.impl_path == symbol.path)
            {
                normalized_doc_comment(comments)
            } else {
                None
            }
        }
        halcyon_lib::ir::Statement::Wasm(_) => None,
    }
}

fn normalized_doc_comment(comments: &str) -> Option<String> {
    let trimmed = comments.trim();
    if trimmed.is_empty() || trimmed.contains("@HIDDEN") {
        return None;
    }
    Some(trimmed.to_string())
}

fn collect_pattern_paths_for_docs(
    pattern: &halcyon_lib::ir::Pattern<()>,
    paths: &mut Vec<halcyon_lib::ir::Path>,
) {
    match &pattern.kind {
        halcyon_lib::ir::PatternKind::Identifier(path) => paths.push(path.clone()),
        halcyon_lib::ir::PatternKind::Tuple(patterns) => {
            for pattern in patterns {
                collect_pattern_paths_for_docs(pattern, paths);
            }
        }
        halcyon_lib::ir::PatternKind::Constructor(_, inner)
        | halcyon_lib::ir::PatternKind::TypeHint(inner, _) => {
            collect_pattern_paths_for_docs(inner, paths);
        }
        halcyon_lib::ir::PatternKind::Struct(fields) => {
            for pattern in fields.values() {
                collect_pattern_paths_for_docs(pattern, paths);
            }
        }
        halcyon_lib::ir::PatternKind::Array {
            starting,
            glob,
            ending,
        } => {
            for pattern in starting.iter().chain(ending.iter()) {
                collect_pattern_paths_for_docs(pattern, paths);
            }
            if let halcyon_lib::ir::Glob::Named(path) = glob {
                paths.push(path.clone());
            }
        }
        halcyon_lib::ir::PatternKind::Hole
        | halcyon_lib::ir::PatternKind::ConstConstructor(_)
        | halcyon_lib::ir::PatternKind::Immediate(_) => {}
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
    use lsp_types::notification::PublishDiagnostics;
    use lsp_types::{
        CodeActionContext,
        CompletionResponse,
        Diagnostic,
        Hover,
        HoverContents,
        PartialResultParams,
        PublishDiagnosticsParams,
        TextDocumentIdentifier,
        TextDocumentItem,
        TextDocumentPositionParams,
        WorkDoneProgressParams,
    };
    use serde_json::json;
    use std::time::{
        SystemTime,
        UNIX_EPOCH,
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

    struct TempWorkspace {
        root_dir: PathBuf,
        bundle_path: PathBuf,
        bundle_uri: Uri,
        source: String,
    }

    impl TempWorkspace {
        fn new(source: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0);
            let root_dir = std::env::temp_dir().join(format!(
                "halcyon-lsp-server-tests-{}-{nanos}",
                std::process::id()
            ));
            std::fs::create_dir_all(&root_dir).expect("temp workspace should be created");
            let bundle_path = root_dir.join("bundle.hc");
            std::fs::write(&bundle_path, source)
                .expect("temp bundle source should be written to disk");
            let bundle_uri = path_to_uri(&bundle_path).expect("bundle path should convert to URI");
            Self {
                root_dir,
                bundle_path,
                bundle_uri,
                source: source.to_string(),
            }
        }
    }

    impl Drop for TempWorkspace {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root_dir);
        }
    }

    fn request_with_json(
        id: i32,
        method: &str,
        params: serde_json::Value,
    ) -> Request {
        Request {
            id: RequestId::from(id),
            method: method.to_string(),
            params,
        }
    }

    fn take_response(
        connection: &Connection,
        id: i32,
    ) -> Response {
        connection
            .receiver
            .try_iter()
            .find_map(|message| {
                let Message::Response(response) = message else {
                    return None;
                };
                (response.id == RequestId::from(id)).then_some(response)
            })
            .unwrap_or_else(|| panic!("expected response for request id {id}"))
    }

    fn open_workspace_document(
        server: &mut Server,
        connection: &Connection,
        workspace: &TempWorkspace,
    ) {
        open_document(
            server,
            connection,
            workspace.bundle_uri.clone(),
            workspace.source.clone(),
        );
    }

    fn open_document(
        server: &mut Server,
        connection: &Connection,
        uri: Uri,
        source: String,
    ) {
        server
            .handle_notification(
                Notification::new(
                    DidOpenTextDocument::METHOD.to_string(),
                    DidOpenTextDocumentParams {
                        text_document: TextDocumentItem {
                            uri,
                            language_id: "halcyon".to_string(),
                            version: 1,
                            text: source,
                        },
                    },
                ),
                connection,
            )
            .expect("didOpen notification should be handled");
    }

    struct SymbolCoverageFixture {
        workspace: TempWorkspace,
        other_path: PathBuf,
        other_uri: Uri,
        root_source: String,
        other_source: String,
    }

    impl SymbolCoverageFixture {
        fn new() -> Self {
            let root_source = [
                "bundle demo",
                "import \"other.hc\"",
                "",
                "let module_decl = item_module",
                "let module_use_root = item_module",
                "let type_decl = ItemType",
                "let type_use_root = ItemType",
                "let constructor_decl = ItemCtor",
                "let constructor_use_root = ItemCtor",
                "let trait_decl = ItemTrait",
                "let trait_use_root = ItemTrait",
                "let term_decl = item_term",
                "let term_use_root = item_term",
                "let wasm_decl = item_wasm",
                "let wasm_use_root = item_wasm",
                "",
            ]
            .join("\n");

            let other_source = [
                "let module_use_other = item_module",
                "let type_use_other = ItemType",
                "let constructor_use_other = ItemCtor",
                "let trait_use_other = ItemTrait",
                "let term_use_other = item_term",
                "let wasm_use_other = item_wasm",
                "",
            ]
            .join("\n");

            let workspace = TempWorkspace::new(&root_source);
            let other_path = workspace.root_dir.join("other.hc");
            std::fs::write(&other_path, &other_source)
                .expect("secondary fixture file should be written");
            let other_uri =
                path_to_uri(&other_path).expect("secondary file path should convert to URI");

            Self {
                workspace,
                other_path,
                other_uri,
                root_source,
                other_source,
            }
        }

        fn nth_offset(
            source: &str,
            token: &str,
            occurrence: usize,
        ) -> usize {
            let mut start = 0usize;
            for index in 0..=occurrence {
                let relative = source[start..]
                    .find(token)
                    .unwrap_or_else(|| panic!("missing occurrence #{index} for token `{token}`"));
                let absolute = start + relative;
                if index == occurrence {
                    return absolute;
                }
                start = absolute + token.len();
            }
            unreachable!("loop always returns on requested occurrence")
        }

        fn source_span(
            source: &str,
            token: &str,
            occurrence: usize,
            file_id: usize,
        ) -> Span {
            Span::Source {
                start: Self::nth_offset(source, token, occurrence),
                width: token.len(),
                file_id: Some(file_id),
            }
        }

        fn seed_server(
            &self,
            server: &mut Server,
        ) {
            let root_path = normalize_path(&self.workspace.bundle_path);
            let other_path = normalize_path(&self.other_path);

            let source_files = vec![
                AnalysisSourceFile {
                    id: 1,
                    path: root_path.clone(),
                    source: self.root_source.clone(),
                },
                AnalysisSourceFile {
                    id: 2,
                    path: other_path,
                    source: self.other_source.clone(),
                },
            ]
            .into_boxed_slice();

            let symbol = |minor: &str, namespace: halcyon_lib::ir::NameSpace| {
                halcyon_lib::ir::ScopedPath {
                    path: halcyon_lib::ir::Path::new("demo", minor),
                    namespace,
                }
            };

            let definitions = HashMap::from([
                (
                    symbol("item_module", halcyon_lib::ir::NameSpace::Module),
                    vec![Self::source_span(&self.root_source, "item_module", 0, 1)]
                        .into_boxed_slice(),
                ),
                (
                    symbol("ItemType", halcyon_lib::ir::NameSpace::Type),
                    vec![Self::source_span(&self.root_source, "ItemType", 0, 1)].into_boxed_slice(),
                ),
                (
                    symbol("ItemCtor", halcyon_lib::ir::NameSpace::Constructor),
                    vec![Self::source_span(&self.root_source, "ItemCtor", 0, 1)].into_boxed_slice(),
                ),
                (
                    symbol("ItemTrait", halcyon_lib::ir::NameSpace::Trait),
                    vec![Self::source_span(&self.root_source, "ItemTrait", 0, 1)]
                        .into_boxed_slice(),
                ),
                (
                    symbol("item_term", halcyon_lib::ir::NameSpace::Term),
                    vec![Self::source_span(&self.root_source, "item_term", 0, 1)]
                        .into_boxed_slice(),
                ),
                (
                    symbol("item_wasm", halcyon_lib::ir::NameSpace::Wasm),
                    vec![Self::source_span(&self.root_source, "item_wasm", 0, 1)]
                        .into_boxed_slice(),
                ),
            ]);

            let usages = HashMap::from([
                (
                    symbol("item_module", halcyon_lib::ir::NameSpace::Module),
                    vec![
                        Self::source_span(&self.root_source, "item_module", 1, 1),
                        Self::source_span(&self.other_source, "item_module", 0, 2),
                    ]
                    .into_boxed_slice(),
                ),
                (
                    symbol("ItemType", halcyon_lib::ir::NameSpace::Type),
                    vec![
                        Self::source_span(&self.root_source, "ItemType", 1, 1),
                        Self::source_span(&self.other_source, "ItemType", 0, 2),
                    ]
                    .into_boxed_slice(),
                ),
                (
                    symbol("ItemCtor", halcyon_lib::ir::NameSpace::Constructor),
                    vec![
                        Self::source_span(&self.root_source, "ItemCtor", 1, 1),
                        Self::source_span(&self.other_source, "ItemCtor", 0, 2),
                    ]
                    .into_boxed_slice(),
                ),
                (
                    symbol("ItemTrait", halcyon_lib::ir::NameSpace::Trait),
                    vec![
                        Self::source_span(&self.root_source, "ItemTrait", 1, 1),
                        Self::source_span(&self.other_source, "ItemTrait", 0, 2),
                    ]
                    .into_boxed_slice(),
                ),
                (
                    symbol("item_term", halcyon_lib::ir::NameSpace::Term),
                    vec![
                        Self::source_span(&self.root_source, "item_term", 1, 1),
                        Self::source_span(&self.other_source, "item_term", 0, 2),
                    ]
                    .into_boxed_slice(),
                ),
                (
                    symbol("item_wasm", halcyon_lib::ir::NameSpace::Wasm),
                    vec![
                        Self::source_span(&self.root_source, "item_wasm", 1, 1),
                        Self::source_span(&self.other_source, "item_wasm", 0, 2),
                    ]
                    .into_boxed_slice(),
                ),
            ]);

            server.bundles.insert(
                root_path.clone(),
                BundleState {
                    frontend: FrontendBundleAnalysis {
                        root_path,
                        bundle_name: "demo".to_string(),
                        source_files,
                        diagnostics: Vec::new().into_boxed_slice(),
                        name_index: Some(halcyon_lib::ir::NameIndex {
                            definitions,
                            usages,
                        }),
                        module: None,
                    },
                    typed: None,
                    generation: 1,
                    published_uris: HashSet::new(),
                },
            );
        }
    }

    fn position_for_token(
        source: &str,
        context: &str,
        token: &str,
    ) -> Position {
        let context_start = source
            .find(context)
            .unwrap_or_else(|| panic!("missing context `{context}`"));
        let token_offset = context
            .find(token)
            .unwrap_or_else(|| panic!("missing token `{token}` in context `{context}`"));
        let position = byte_offset_to_utf16_position(source, context_start + token_offset);
        Position {
            line: position.line,
            character: position.character,
        }
    }

    fn request_hover(
        server: &mut Server,
        server_connection: &Connection,
        client_connection: &Connection,
        request_id: i32,
        uri: &Uri,
        position: Position,
    ) -> Hover {
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
        };
        server
            .handle_request(
                request_with_json(
                    request_id,
                    HoverRequest::METHOD,
                    serde_json::to_value(params).expect("hover params should serialize"),
                ),
                server_connection,
            )
            .expect("hover request should be handled");

        let response = take_response(client_connection, request_id);
        assert!(response.error.is_none(), "hover request should not fail");
        let payload = response
            .result
            .expect("hover request should return a result payload");
        let hover: Option<Hover> =
            serde_json::from_value(payload).expect("hover payload should deserialize");
        hover.expect("hover request should resolve a symbol")
    }

    fn hover_markdown(hover: Hover) -> String {
        let HoverContents::Markup(markup) = hover.contents else {
            panic!("hover response should be markdown");
        };
        markup.value
    }

    fn request_rename(
        server: &mut Server,
        server_connection: &Connection,
        client_connection: &Connection,
        request_id: i32,
        uri: &Uri,
        position: Position,
        new_name: &str,
    ) -> WorkspaceEdit {
        let params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position,
            },
            new_name: new_name.to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };
        server
            .handle_request(
                request_with_json(
                    request_id,
                    Rename::METHOD,
                    serde_json::to_value(params).expect("rename params should serialize"),
                ),
                server_connection,
            )
            .expect("rename request should be handled");

        let response = take_response(client_connection, request_id);
        assert!(response.error.is_none(), "rename request should not fail");
        let payload = response
            .result
            .expect("rename request should return a result payload");
        let edit: Option<WorkspaceEdit> =
            serde_json::from_value(payload).expect("rename payload should deserialize");
        edit.expect("rename request should return workspace edits")
    }

    fn assert_rename_updates_both_files(
        edit: WorkspaceEdit,
        root_uri: &Uri,
        other_uri: &Uri,
        expected_name: &str,
    ) {
        let changes = edit
            .changes
            .expect("rename workspace edit should include text changes");
        let root_edits = changes
            .get(root_uri)
            .expect("root file should receive rename edits");
        let other_edits = changes
            .get(other_uri)
            .expect("secondary file should receive rename edits");

        assert!(
            !root_edits.is_empty(),
            "root file should have at least one edit"
        );
        assert!(
            !other_edits.is_empty(),
            "secondary file should have at least one edit"
        );
        assert!(
            root_edits
                .iter()
                .chain(other_edits.iter())
                .all(|edit| edit.new_text == expected_name),
            "rename should only emit replacement text `{expected_name}`"
        );
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
                    associated_types: Vec::new().into_boxed_slice(),
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
                    associated_types: Vec::new().into_boxed_slice(),
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
                    associated_types: Vec::new().into_boxed_slice(),
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
                    associated_types: Vec::new().into_boxed_slice(),
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
                    associated_types: Vec::new().into_boxed_slice(),
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
                    associated_types: Vec::new().into_boxed_slice(),
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
                    associated_types: Vec::new().into_boxed_slice(),
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
                associated_types: Vec::new().into_boxed_slice(),
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
    fn handle_request_returns_invalid_params_for_malformed_payloads() {
        let (server_connection, client_connection) = Connection::memory();
        let mut server = Server::new();

        server
            .handle_request(
                request_with_json(1, Completion::METHOD, json!({ "invalid": true })),
                &server_connection,
            )
            .expect("malformed request should not crash handler");

        let response = take_response(&client_connection, 1);
        let error = response
            .error
            .expect("invalid params request should return an error response");
        assert_eq!(error.code, ErrorCode::InvalidParams as i32);
        assert!(
            error
                .message
                .contains("Invalid params for `textDocument/completion`")
        );
    }

    #[test]
    fn handle_notification_skips_malformed_payloads_and_keeps_serving() {
        let (server_connection, client_connection) = Connection::memory();
        let mut server = Server::new();

        server
            .handle_notification(
                Notification::new(
                    DidOpenTextDocument::METHOD.to_string(),
                    json!({ "invalid": true }),
                ),
                &server_connection,
            )
            .expect("malformed notification should be ignored");

        server
            .handle_request(
                request_with_json(2, "demo/unknown", serde_json::Value::Null),
                &server_connection,
            )
            .expect("server should continue handling later requests");

        let response = take_response(&client_connection, 2);
        let error = response
            .error
            .expect("unknown methods should return method-not-found response");
        assert_eq!(error.code, ErrorCode::MethodNotFound as i32);
    }

    #[test]
    fn completion_request_returns_keyword_items_through_request_handler() {
        let (server_connection, client_connection) = Connection::memory();
        let mut server = Server::new();
        let workspace = TempWorkspace::new("bundle demo\nlet answer = 1\nlet probe = le\n");
        open_workspace_document(&mut server, &server_connection, &workspace);

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: workspace.bundle_uri.clone(),
                },
                position: Position {
                    line: 2,
                    character: 14,
                },
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        };
        server
            .handle_request(
                request_with_json(
                    3,
                    Completion::METHOD,
                    serde_json::to_value(params).expect("completion params should serialize"),
                ),
                &server_connection,
            )
            .expect("completion request should be handled");

        let response = take_response(&client_connection, 3);
        let payload = response
            .result
            .expect("completion request should return a result payload");
        let completion: Option<CompletionResponse> =
            serde_json::from_value(payload).expect("completion payload should deserialize");
        let Some(CompletionResponse::List(list)) = completion else {
            panic!("completion response should use completion list");
        };

        assert!(
            list.items.iter().any(|item| item.label == "let"),
            "keyword completion should include `let`"
        );
    }

    #[test]
    fn rename_request_returns_workspace_edits_for_local_symbols() {
        let (server_connection, client_connection) = Connection::memory();
        let mut server = Server::new();
        let workspace = TempWorkspace::new("bundle demo\nlet value = 1\nlet alias = value\n");
        open_workspace_document(&mut server, &server_connection, &workspace);

        let params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: workspace.bundle_uri.clone(),
                },
                position: Position {
                    line: 2,
                    character: 13,
                },
            },
            new_name: "renamed".to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };
        server
            .handle_request(
                request_with_json(
                    4,
                    Rename::METHOD,
                    serde_json::to_value(params).expect("rename params should serialize"),
                ),
                &server_connection,
            )
            .expect("rename request should be handled");

        let response = take_response(&client_connection, 4);
        let payload = response
            .result
            .expect("rename request should return a result payload");
        let edit: Option<WorkspaceEdit> =
            serde_json::from_value(payload).expect("rename payload should deserialize");
        let Some(edit) = edit else {
            panic!("rename request should produce workspace edits");
        };
        let changes = edit
            .changes
            .expect("rename workspace edit should include text changes");
        let text_edits = changes
            .get(&workspace.bundle_uri)
            .expect("bundle document should receive rename edits");

        assert!(
            text_edits.len() >= 2,
            "rename should update declaration and at least one usage"
        );
        assert!(text_edits.iter().all(|edit| edit.new_text == "renamed"));
    }

    #[test]
    fn hover_covers_symbol_kinds_within_and_across_files() {
        let (server_connection, client_connection) = Connection::memory();
        let mut server = Server::new();
        let fixture = SymbolCoverageFixture::new();
        fixture.seed_server(&mut server);

        let root_uri = fixture.workspace.bundle_uri.clone();
        let root_source = fixture.root_source.clone();
        let other_uri = fixture.other_uri.clone();
        let other_source = fixture.other_source.clone();

        let mut request_id = 500;
        let mut assert_hover_contains =
            |uri: &Uri,
             position: Position,
             expected_path_fragment: &str,
             expected_namespace: &str| {
                request_id += 1;
                let markdown = hover_markdown(request_hover(
                    &mut server,
                    &server_connection,
                    &client_connection,
                    request_id,
                    uri,
                    position,
                ));
                assert!(
                    markdown.contains(expected_path_fragment),
                    "hover should contain path fragment `{expected_path_fragment}`, got: {markdown}"
                );
                assert!(
                    markdown.contains(&format!("**Namespace**: `{expected_namespace}`")),
                    "hover should contain namespace `{expected_namespace}`, got: {markdown}"
                );
            };

        assert_hover_contains(
            &root_uri,
            position_for_token(&root_source, "let module_decl = item_module", "item_module"),
            "**Path**: `demo::item_module`",
            "module",
        );
        assert_hover_contains(
            &other_uri,
            position_for_token(
                &other_source,
                "let module_use_other = item_module",
                "item_module",
            ),
            "**Path**: `demo::item_module`",
            "module",
        );

        assert_hover_contains(
            &root_uri,
            position_for_token(&root_source, "let type_decl = ItemType", "ItemType"),
            "**Path**: `demo::ItemType`",
            "type",
        );
        assert_hover_contains(
            &other_uri,
            position_for_token(&other_source, "let type_use_other = ItemType", "ItemType"),
            "**Path**: `demo::ItemType`",
            "type",
        );

        assert_hover_contains(
            &root_uri,
            position_for_token(&root_source, "let constructor_decl = ItemCtor", "ItemCtor"),
            "**Path**: `demo::ItemCtor`",
            "constructor",
        );
        assert_hover_contains(
            &other_uri,
            position_for_token(
                &other_source,
                "let constructor_use_other = ItemCtor",
                "ItemCtor",
            ),
            "**Path**: `demo::ItemCtor`",
            "constructor",
        );

        assert_hover_contains(
            &root_uri,
            position_for_token(&root_source, "let trait_decl = ItemTrait", "ItemTrait"),
            "**Path**: `demo::ItemTrait`",
            "trait",
        );
        assert_hover_contains(
            &other_uri,
            position_for_token(
                &other_source,
                "let trait_use_other = ItemTrait",
                "ItemTrait",
            ),
            "**Path**: `demo::ItemTrait`",
            "trait",
        );

        assert_hover_contains(
            &root_uri,
            position_for_token(&root_source, "let term_decl = item_term", "item_term"),
            "**Path**: `demo::item_term`",
            "term",
        );
        assert_hover_contains(
            &other_uri,
            position_for_token(&other_source, "let term_use_other = item_term", "item_term"),
            "**Path**: `demo::item_term`",
            "term",
        );

        assert_hover_contains(
            &root_uri,
            position_for_token(&root_source, "let wasm_decl = item_wasm", "item_wasm"),
            "**Path**: `demo::item_wasm`",
            "wasm",
        );
        assert_hover_contains(
            &other_uri,
            position_for_token(&other_source, "let wasm_use_other = item_wasm", "item_wasm"),
            "**Path**: `demo::item_wasm`",
            "wasm",
        );
    }

    #[test]
    fn hover_includes_doc_comments_for_declarations_and_cross_file_usages() {
        let (server_connection, client_connection) = Connection::memory();
        let mut server = Server::new();

        let root_source = [
            "bundle demo",
            "import \"other.hc\"",
            "",
            "--> Root trait docs.",
            "trait LocalTrait : a =",
            "  let trait_item : a -> a",
            "end",
            "",
            "--> Root type docs.",
            "type LocalType =",
            "  | LocalCtor",
            "",
            "--> Root term docs.",
            "let root_term = other_term",
            "",
            "--> Hidden docs.",
            "--> @HIDDEN",
            "let hidden_term = 1",
            "",
        ]
        .join("\n");
        let other_source = ["--> Other term docs.", "let other_term = LocalCtor", ""].join("\n");

        let workspace = TempWorkspace::new(&root_source);
        let other_path = workspace.root_dir.join("other.hc");
        std::fs::write(&other_path, &other_source)
            .expect("secondary source file should be written");
        let other_uri =
            path_to_uri(&other_path).expect("secondary source path should convert to URI");

        open_workspace_document(&mut server, &server_connection, &workspace);
        open_document(
            &mut server,
            &server_connection,
            other_uri.clone(),
            other_source.clone(),
        );

        let root_term_hover = hover_markdown(request_hover(
            &mut server,
            &server_connection,
            &client_connection,
            700,
            &workspace.bundle_uri,
            position_for_token(&root_source, "let root_term = other_term", "root_term"),
        ));
        assert!(root_term_hover.contains("**Docs**"));
        assert!(root_term_hover.contains("Root term docs."));

        let cross_file_hover = hover_markdown(request_hover(
            &mut server,
            &server_connection,
            &client_connection,
            701,
            &workspace.bundle_uri,
            position_for_token(&root_source, "let root_term = other_term", "other_term"),
        ));
        assert!(cross_file_hover.contains("Other term docs."));

        let constructor_hover = hover_markdown(request_hover(
            &mut server,
            &server_connection,
            &client_connection,
            702,
            &workspace.bundle_uri,
            position_for_token(&root_source, "| LocalCtor", "LocalCtor"),
        ));
        assert!(constructor_hover.contains("Root type docs."));

        let trait_hover = hover_markdown(request_hover(
            &mut server,
            &server_connection,
            &client_connection,
            703,
            &workspace.bundle_uri,
            position_for_token(&root_source, "trait LocalTrait : a =", "LocalTrait"),
        ));
        assert!(trait_hover.contains("Root trait docs."));

        let hidden_hover = hover_markdown(request_hover(
            &mut server,
            &server_connection,
            &client_connection,
            704,
            &workspace.bundle_uri,
            position_for_token(&root_source, "let hidden_term = 1", "hidden_term"),
        ));
        assert!(!hidden_hover.contains("Hidden docs."));
    }

    #[test]
    fn rename_covers_symbol_kinds_across_and_within_files() {
        let (server_connection, client_connection) = Connection::memory();
        let mut server = Server::new();
        let fixture = SymbolCoverageFixture::new();
        fixture.seed_server(&mut server);

        let root_uri = fixture.workspace.bundle_uri.clone();
        let root_source = fixture.root_source.clone();
        let other_uri = fixture.other_uri.clone();
        let other_source = fixture.other_source.clone();

        let mut request_id = 600;

        request_id += 1;
        let module_edit = request_rename(
            &mut server,
            &server_connection,
            &client_connection,
            request_id,
            &other_uri,
            position_for_token(
                &other_source,
                "let module_use_other = item_module",
                "item_module",
            ),
            "renamed_module",
        );
        assert_rename_updates_both_files(module_edit, &root_uri, &other_uri, "renamed_module");

        request_id += 1;
        let type_edit = request_rename(
            &mut server,
            &server_connection,
            &client_connection,
            request_id,
            &root_uri,
            position_for_token(&root_source, "let type_decl = ItemType", "ItemType"),
            "RenamedType",
        );
        assert_rename_updates_both_files(type_edit, &root_uri, &other_uri, "RenamedType");

        request_id += 1;
        let constructor_edit = request_rename(
            &mut server,
            &server_connection,
            &client_connection,
            request_id,
            &other_uri,
            position_for_token(
                &other_source,
                "let constructor_use_other = ItemCtor",
                "ItemCtor",
            ),
            "RenamedCtor",
        );
        assert_rename_updates_both_files(constructor_edit, &root_uri, &other_uri, "RenamedCtor");

        request_id += 1;
        let trait_edit = request_rename(
            &mut server,
            &server_connection,
            &client_connection,
            request_id,
            &other_uri,
            position_for_token(
                &other_source,
                "let trait_use_other = ItemTrait",
                "ItemTrait",
            ),
            "RenamedTrait",
        );
        assert_rename_updates_both_files(trait_edit, &root_uri, &other_uri, "RenamedTrait");

        request_id += 1;
        let term_edit = request_rename(
            &mut server,
            &server_connection,
            &client_connection,
            request_id,
            &other_uri,
            position_for_token(&other_source, "let term_use_other = item_term", "item_term"),
            "renamed_term",
        );
        assert_rename_updates_both_files(term_edit, &root_uri, &other_uri, "renamed_term");

        request_id += 1;
        let wasm_edit = request_rename(
            &mut server,
            &server_connection,
            &client_connection,
            request_id,
            &other_uri,
            position_for_token(&other_source, "let wasm_use_other = item_wasm", "item_wasm"),
            "renamed_wasm",
        );
        assert_rename_updates_both_files(wasm_edit, &root_uri, &other_uri, "renamed_wasm");
    }

    #[test]
    fn drain_typecheck_results_drops_stale_generations() {
        let (server_connection, _client_connection) = Connection::memory();
        let mut server = Server::new();
        let workspace = TempWorkspace::new("bundle demo\nlet value = 1\n");

        let source_files = vec![AnalysisSourceFile {
            id: 1,
            path: workspace.bundle_path.clone(),
            source: workspace.source.clone(),
        }]
        .into_boxed_slice();
        let root_path = workspace.bundle_path.clone();

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
                generation: 2,
                published_uris: HashSet::new(),
            },
        );

        let make_analysis = |message: &str| {
            BundleAnalysis {
                root_path: root_path.clone(),
                bundle_name: "demo".to_string(),
                source_files: source_files.clone(),
                diagnostics: vec![halcyon_lib::SerializedDiagnostic {
                    severity: "warning".to_string(),
                    code: None,
                    message: message.to_string(),
                    labels: vec![halcyon_lib::SerializedDiagnosticLabel {
                        style: "primary".to_string(),
                        file_name: workspace.bundle_path.to_string_lossy().to_string(),
                        message: message.to_string(),
                        range_start: 0,
                        range_end: 6,
                        start: halcyon_lib::SerializedDiagnosticLocation { line: 1, column: 1 },
                        end: halcyon_lib::SerializedDiagnosticLocation { line: 1, column: 7 },
                    }],
                    notes: Vec::new(),
                }]
                .into_boxed_slice(),
                symbols: server.base_symbols.clone(),
                name_index: None,
            }
        };

        server
            .typecheck_result_sender
            .send(TypecheckResult {
                root_path: root_path.clone(),
                generation: 2,
                analysis: Ok(make_analysis("latest")),
            })
            .expect("latest typecheck result should queue");
        server
            .typecheck_result_sender
            .send(TypecheckResult {
                root_path: root_path.clone(),
                generation: 1,
                analysis: Ok(make_analysis("stale")),
            })
            .expect("stale typecheck result should queue");
        server
            .drain_typecheck_results(&server_connection)
            .expect("draining typecheck results should succeed");

        let typed = server
            .bundles
            .get(&root_path)
            .and_then(|bundle| bundle.typed.as_ref())
            .expect("latest typed snapshot should be retained");

        assert_eq!(typed.generation, 2);
        assert!(typed.analysis.diagnostics[0].message.contains("latest"));
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
