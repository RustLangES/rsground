use std::sync::LazyLock;

use rsground_runner::lsp::*;

pub const LSP_INITIALIZATION: LazyLock<InitializeParams> = LazyLock::new(|| InitializeParams {
    capabilities: ClientCapabilities {
        text_document: Some(TextDocumentClientCapabilities {
            completion: Some(CompletionClientCapabilities {
                completion_item: Some(CompletionItemCapability {
                    commit_characters_support: Some(true),
                    deprecated_support: Some(false),
                    documentation_format: Some(vec![MarkupKind::Markdown]),
                    preselect_support: Some(false),
                    snippet_support: Some(false),
                    ..Default::default()
                }),
                context_support: Some(false),
                dynamic_registration: Some(true),
                ..Default::default()
            }),
            declaration: Some(GotoCapability {
                dynamic_registration: Some(true),
                link_support: Some(true),
            }),
            definition: Some(GotoCapability {
                dynamic_registration: Some(true),
                link_support: Some(true),
            }),
            hover: Some(HoverClientCapabilities {
                dynamic_registration: Some(true),
                content_format: Some(vec![MarkupKind::Markdown]),
                ..Default::default()
            }),
            implementation: Some(GotoCapability {
                dynamic_registration: Some(true),
                link_support: Some(true),
            }),
            signature_help: Some(SignatureHelpClientCapabilities {
                dynamic_registration: Some(true),
                signature_information: Some(SignatureInformationSettings {
                    documentation_format: Some(vec![MarkupKind::Markdown]),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            synchronization: Some(TextDocumentSyncClientCapabilities {
                did_save: Some(false),
                dynamic_registration: Some(true),
                will_save: Some(false),
                will_save_wait_until: Some(false),
            }),
            type_definition: Some(GotoCapability {
                dynamic_registration: Some(true),
                link_support: Some(true),
            }),
            ..Default::default()
        }),
        window: Some(WindowClientCapabilities {
            work_done_progress: Some(true),
            ..Default::default()
        }),
        workspace: Some(WorkspaceClientCapabilities {
            did_change_configuration: Some(DynamicRegistrationClientCapabilities {
                dynamic_registration: Some(true),
            }),
            ..Default::default()
        }),
        ..Default::default()
    },

    initialization_options: Some(serde_json::json!({
        "cachePriming": {
            "enable": false
        },
        "cargo": {
            "allTargets": false,
            "buildScripts": {
                "enable": false
            },
        },
        "completion": {
            "limit": 25
        },
        "numThreads": 1,
    })),
    ..Default::default()
});
