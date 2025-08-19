use std::io::{self, Write};
use std::sync::{Arc, LazyLock};

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, StreamHandler};
use futures::StreamExt;
use itertools::Itertools;
use rsground_runner::hakoniwa_ext::HakoniwaChildExt;
use rsground_runner::lsp::notification::{Initialized, Notification};
use rsground_runner::lsp::request::{Initialize, Request};
use rsground_runner::lsp::{
    InitializeResult, InitializedParams, LspId, LspInput, LspNotify, LspOutput, LspRequest,
    LspResponse, ServerInfo,
};
use rsground_runner::{PipeWriter, Runner};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::utils::{define_local_logger, ArcStr};
use crate::ws::messages::InternalMessage;

use super::project_runner::{start_job, AbortSender};

define_local_logger!(local_log as "backend::lsp" {
    client_stdin,
    initialize,
    notify => "notification",
    request,
    response,
    stderr,
    stdin,
    stdout,
});

const LSP_INITIALIZATION_ID: &str = "rsground::initialize";

const LSP_SERVER_INITIALIZATION: LazyLock<InitializeResult> = LazyLock::new(|| {
    use rsground_runner::lsp::*;

    InitializeResult {
        capabilities: rsground_runner::lsp::ServerCapabilities {
            call_hierarchy_provider: Some(CallHierarchyServerCapability::Simple(true)),
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(true),
            }),
            completion_provider: Some(CompletionOptions {
                completion_item: Some(CompletionOptionsCompletionItem {
                    label_details_support: Some(false),
                }),
                resolve_provider: Some(false),
                trigger_characters: Some(
                    [":", ".", "'", "("].into_iter().map(String::from).collect(),
                ),
                ..Default::default()
            }),
            declaration_provider: Some(DeclarationCapability::Simple(true)),
            definition_provider: Some(OneOf::Left(true)),
            diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
                inter_file_dependencies: true,
                workspace_diagnostics: true,
                ..Default::default()
            })),
            document_formatting_provider: Some(OneOf::Left(true)),
            document_highlight_provider: Some(OneOf::Left(true)),
            document_on_type_formatting_provider: Some(DocumentOnTypeFormattingOptions {
                first_trigger_character: String::from("."),
                more_trigger_character: Some(
                    ["=", "<", ">", "{", "}", "|"]
                        .into_iter()
                        .map(String::from)
                        .collect(),
                ),
            }),
            document_range_formatting_provider: Some(OneOf::Left(false)),
            document_symbol_provider: Some(OneOf::Left(true)),
            experimental: Some(serde_json::json!({
              "externalDocs": true,
              "hoverRange": true,
              "joinLines": true,
              "matchingBrace": true,
              "moveItem": true,
              "onEnter": true,
              "openCargoToml": true,
              "parentModule": true,
              "runnables": {"kinds": [ "cargo" ]},
              "ssr": true,
              "workspaceSymbolScopeKindFiltering": true
            })),
            folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            implementation_provider: Some(ImplementationProviderCapability::Simple(true)),
            inlay_hint_provider: Some(OneOf::Right(InlayHintServerCapabilities::Options(
                InlayHintOptions {
                    resolve_provider: Some(false),
                    ..Default::default()
                },
            ))),
            position_encoding: Some(PositionEncodingKind::new("utf-16")),
            references_provider: Some(OneOf::Left(true)),
            rename_provider: Some(OneOf::Right(RenameOptions {
                prepare_provider: Some(true),
                work_done_progress_options: WorkDoneProgressOptions::default(),
            })),
            selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
            semantic_tokens_provider: Some(
                SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                    //   "full": Object {"delta": Bool(true)},
                    full: Some(SemanticTokensFullOptions::Delta { delta: Some(true) }),
                    legend: SemanticTokensLegend {
                        #[rustfmt::skip]
                        token_modifiers: [
                            "async", "documentation", "declaration", "static", "defaultLibrary",
                            "associated", "attribute", "callable", "constant", "consuming",
                            "controlFlow", "crateRoot", "injected", "intraDocLink", "library",
                            "macro", "mutable", "procMacro", "public", "reference", "trait",
                            "unsafe",
                        ]
                        .into_iter()
                        .map(SemanticTokenModifier::new)
                        .collect(),

                        #[rustfmt::skip]
                        token_types: [
                            "comment", "decorator", "enumMember", "enum", "function", "interface",
                            "keyword", "macro", "method", "namespace", "number", "operator",
                            "parameter", "property", "string", "struct", "typeParameter", "variable",
                            "angle", "arithmetic", "attributeBracket", "attribute", "bitwise",
                            "boolean", "brace", "bracket", "builtinAttribute", "builtinType",
                            "character", "colon", "comma", "comparison", "constParameter", "const",
                            "deriveHelper", "derive", "dot", "escapeSequence", "formatSpecifier",
                            "generic", "invalidEscapeSequence", "label", "lifetime", "logical",
                            "macroBang", "parenthesis", "procMacro", "punctuation", "selfKeyword",
                            "selfTypeKeyword", "semicolon", "static", "toolModule", "typeAlias",
                            "union", "unresolvedReference",
                        ]
                        .into_iter()
                        .map(SemanticTokenType::new)
                        .collect(),
                    },
                    range: Some(true),
                    ..Default::default()
                }),
            ),
            signature_help_provider: Some(SignatureHelpOptions {
                trigger_characters: Some(["(", ",", "<"].into_iter().map(String::from).collect()),
                ..Default::default()
            }),
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    change: Some(TextDocumentSyncKind::INCREMENTAL),
                    open_close: Some(true),
                    save: Some(TextDocumentSyncSaveOptions::SaveOptions(
                        SaveOptions::default(),
                    )),
                    ..Default::default()
                },
            )),
            type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
            workspace: Some(WorkspaceServerCapabilities {
                file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                    will_rename: Some(FileOperationRegistrationOptions {
                        filters: [
                            FileOperationFilter {
                                pattern: FileOperationPattern {
                                    glob: String::from("**/*.rs"),
                                    matches: Some(FileOperationPatternKind::File),
                                    ..Default::default()
                                },
                                scheme: Some(String::from("file")),
                            },
                            FileOperationFilter {
                                pattern: FileOperationPattern {
                                    glob: String::from("**"),
                                    matches: Some(FileOperationPatternKind::Folder),
                                    ..Default::default()
                                },
                                scheme: Some(String::from("file")),
                            },
                        ]
                        .into(),
                    }),
                    ..Default::default()
                }),
                workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                    change_notifications: Some(OneOf::Left(true)),
                    supported: Some(true),
                }),
                ..Default::default()
            }),
            workspace_symbol_provider: Some(OneOf::Left(true)),
            ..Default::default()
        },
        server_info: Some(ServerInfo {
            name: String::from("rust-analyzer"),
            version: Some(String::from("1.85.1 (4eb1612 2025-03-15)")),
        }),
    }
});

pub struct ProjectLsp {
    project_id: Uuid,
    broadcast: broadcast::Sender<InternalMessage>,
    runner: Arc<Runner>,
    instance: Option<(AbortSender, PipeWriter)>,
}

impl ProjectLsp {
    pub async fn start(
        project_id: Uuid,
        broadcast: broadcast::Sender<InternalMessage>,
        runner: Arc<Runner>,
    ) -> Addr<Self> {
        let project_lsp = Self {
            project_id,
            broadcast,
            runner,
            instance: None,
        };

        project_lsp.start()
    }
}

impl Actor for ProjectLsp {
    type Context = Context<ProjectLsp>;
}

// Client-side messages

#[derive(Message)]
#[rtype(result = "()")]
pub struct Execute;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Stdin(String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientStdin(pub ArcStr, pub serde_json::Value);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Kill;

impl Handler<Execute> for ProjectLsp {
    type Result = ();

    fn handle(&mut self, _: Execute, ctx: &mut Self::Context) -> Self::Result {
        if self.instance.is_some() {
            local_log::warn!(target: self.project_id, "rust-analyzer already started");
            return;
        }

        local_log::debug!(target: self.project_id, "Starting");

        let Ok((child, stdin, stdout, stderr)) = self.runner.start_rls() else {
            local_log::error!(target: self.project_id, "Failed to start rust-analyzer");
            return;
        };

        start_job(
            self,
            ctx,
            move |abort, this, ctx| {
                this.instance.replace((abort, stdin));

                match LspInput::request::<Initialize>(LSP_INITIALIZATION_ID, Default::default()) {
                    Ok(req) => {
                        local_log::initialize::info!(target: this.project_id, "Initialize sended");
                        ctx.notify(Stdin(req))
                    }
                    Err(err) => {
                        local_log::initialize::error!(target: this.project_id, "{err:?}");
                        ctx.notify(Kill);
                    }
                }
            },
            async move |abort| child.wait_or_abort(abort).await,
            |_, this, _| {
                drop(this.instance.take());
            },
        );

        ctx.add_stream(stdout.into_lsp());
        ctx.add_stream(stderr.stream::<2048>().map(Stderr));
    }
}

impl Handler<Kill> for ProjectLsp {
    type Result = ();

    fn handle(&mut self, _: Kill, _: &mut Self::Context) -> Self::Result {
        if let Some((abort, _)) = self.instance.take() {
            _ = abort.send(());
        }
    }
}

impl Handler<Stdin> for ProjectLsp {
    type Result = ();

    fn handle(&mut self, Stdin(msg): Stdin, _: &mut Self::Context) -> Self::Result {
        if let Some((_, stdin)) = self.instance.as_mut() {
            let msg = format!("Content-Length: {}\r\n\r\n{msg}", msg.len());
            if let Err(err) = stdin.write_all(msg.as_bytes()).and_then(|_| stdin.flush()) {
                local_log::stdin::error!(target: self.project_id, "Cannot write: {err}")
            }
        }
    }
}

impl Handler<ClientStdin> for ProjectLsp {
    type Result = ();

    fn handle(
        &mut self,
        ClientStdin(client_id, msg): ClientStdin,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let Ok(msg) = serde_json::from_value::<LspOutput>(msg).map_err(
            |err| local_log::client_stdin::error!(target: self.project_id, "Deserialize: {err:?}"),
        ) else {
            return;
        };

        // Handle initialization
        match msg {
            LspOutput::Request(LspRequest { id, ref method, .. })
                if method == Initialize::METHOD =>
            {
                _ = self.broadcast.send(InternalMessage::Lsp {
                    client_id: Some(client_id.into()),
                    data: LspInput::response_value::<Initialize>(
                        id,
                        Ok(&*LSP_SERVER_INITIALIZATION),
                    )
                    .expect("Empty params should not fail"),
                });
                return;
            }
            LspOutput::Notify(LspNotify { ref method, .. }) if method == Initialized::METHOD => {
                return
            }
            _ => {}
        }

        macro_rules! with_id {
            ($id:expr, |$inner:ident| $body:expr) => {
                serde_json::to_string($id)
                    .map(|id| LspId::String(format!("client_id::{client_id}::{id}").into()))
                    .map(|$inner| $body)
            };
        }

        let msg = match msg {
            LspOutput::Response(LspResponse::Ok {
                jsonrpc,
                id,
                result,
            }) => with_id!(&id, |id| LspOutput::Response(LspResponse::Ok {
                id,
                jsonrpc,
                result,
            })),
            LspOutput::Response(LspResponse::Err { jsonrpc, id, error }) => {
                with_id!(&id, |id| LspOutput::Response(LspResponse::Err {
                    id,
                    jsonrpc,
                    error,
                }))
            }
            LspOutput::Request(req) => {
                with_id!(&req.id, |id| LspOutput::Request(LspRequest { id, ..req }))
            }
            notify @ LspOutput::Notify(..) => Ok(notify),
        };

        let Ok(msg) = msg
            .and_then(|msg| serde_json::to_string(&msg))
            .map_err(|err| {
                local_log::client_stdin::error!(
                    target: self.project_id,
                    "Cannot serialize: {err}"
                )
            })
        else {
            return;
        };

        ctx.notify(Stdin(msg));
    }
}

// Internal-side messages

type Stdout = Result<LspOutput, io::Error>;

impl StreamHandler<Stdout> for ProjectLsp {
    fn handle(&mut self, msg: Stdout, ctx: &mut Self::Context) {
        let Ok(msg) = msg.map_err(|err| local_log::error!(target: self.project_id, "{err:?}"))
        else {
            return;
        };

        match msg {
            LspOutput::Response(res) if res.id() == LSP_INITIALIZATION_ID => match res {
                LspResponse::Ok { result, .. } => {
                    local_log::initialize::info!(target: self.project_id, "Initialize received");
                    local_log::initialize::trace!(target: self.project_id, "{result:?}");
                    if let Ok(noti) = LspInput::notify::<Initialized>(InitializedParams {}) {
                        ctx.notify(Stdin(noti));
                    }
                }
                LspResponse::Err { error, .. } => {
                    local_log::initialize::error!(target: self.project_id, "{error:?}");
                    ctx.notify(Kill);
                }
            },

            LspOutput::Response(res) => {
                local_log::response::trace!(target: self.project_id, "{res:?}");
                let (client_id, res_id) = res
                    .id()
                    .clone()
                    .into_string()
                    .and_then(|s| {
                        s.strip_prefix("client_id::")
                            .and_then(|s| s.splitn(2, "::").collect_tuple::<(_, _)>())
                            .map(|(client_id, original_id)| {
                                (
                                    Arc::from(client_id),
                                    serde_json::from_str::<LspId>(original_id).inspect_err(|err| {
                                        local_log::response::error!(
                                            target: self.project_id,
                                            "Cannot deserialize original_id ({original_id}): {err}"
                                        )
                                    }),
                                )
                            })
                    })
                    .and_then(|(client_id, original_id)| {
                        original_id.ok().map(|original_id| (client_id, original_id))
                    })
                    .unzip();

                // Update res id if required to recover the original id
                let res = match res_id {
                    None => res,
                    Some(id) => match res {
                        LspResponse::Ok {
                            result, jsonrpc, ..
                        } => LspResponse::Ok {
                            id,
                            result,
                            jsonrpc,
                        },
                        LspResponse::Err { error, jsonrpc, .. } => {
                            LspResponse::Err { id, error, jsonrpc }
                        }
                    },
                };

                match serde_json::to_value(res) {
                    Ok(data) => {
                        _ = self
                            .broadcast
                            .send(InternalMessage::Lsp { client_id, data });
                    }
                    Err(err) => {
                        local_log::response::error!(target: self.project_id, "Cannot serialize {err:?}");
                    }
                }
            }
            LspOutput::Request(req) => {
                local_log::request::trace!(target: self.project_id, "{req:?}");
            }
            LspOutput::Notify(noti) => {
                local_log::notify::trace!(target: self.project_id, "{noti:?}");
                match serde_json::to_value(noti) {
                    Ok(data) => {
                        _ = self.broadcast.send(InternalMessage::Lsp {
                            client_id: None,
                            data,
                        });
                    }
                    Err(err) => {
                        local_log::response::error!(target: self.project_id, "Cannot serialize {err:?}");
                    }
                }
            }
        }
    }
}

struct Stderr(Result<Vec<u8>, io::Error>);

impl StreamHandler<Stderr> for ProjectLsp {
    fn handle(&mut self, Stderr(msg): Stderr, _: &mut Self::Context) {
        let Ok(msg) = msg
            .map_err(|err| local_log::stderr::error!(target: self.project_id, "{err:?}"))
            .map(String::from_utf8)
            .and_then(|s| {
                s.map_err(|err| local_log::stderr::error!(target: self.project_id, "Deserialize: {err:?}"))
            })
        else {
            return;
        };

        const TIMESTAMP_LEN: usize = "YYYY-MM-DDTHH:mm:ss".len();

        // if has space exacly where timestamp ends
        let has_timestamp = msg
            .get(..TIMESTAMP_LEN)
            .is_some_and(|s| s.chars().all(|c| c != '.'))
            && msg
                .get(TIMESTAMP_LEN..TIMESTAMP_LEN + 1)
                .is_some_and(|c| c == ".");

        let msg = if has_timestamp {
            msg.chars()
                .skip(TIMESTAMP_LEN)
                .skip_while(|c| *c != ' ')
                .skip(1)
                .collect()
        } else {
            msg
        };

        local_log::stderr::warn!(target: self.project_id, "{msg:?}");
    }
}
