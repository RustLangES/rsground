use std::sync::LazyLock;

use rsground_runner::lsp::*;

pub const LSP_INITIALIZATION: LazyLock<InitializeResult> = LazyLock::new(|| InitializeResult {
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
            trigger_characters: Some([":", ".", "'", "("].into_iter().map(String::from).collect()),
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
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
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
            },
        )),
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
});
