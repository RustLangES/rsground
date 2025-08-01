#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, reason = "this file no test in debug mode")
)]
mod common;

use std::io::Write;
use std::time::Duration;

use common::cargo::cargo_init;
use futures::{StreamExt, TryStreamExt};
use rsground_runner::hakoniwa_ext::HakoniwaChildExt;
use rsground_runner::Runner;
use tokio::io::AsyncReadExt;

fn make_request(id: u16, method: &str, params: &str) -> String {
    let content = format!(r#"{{"jsonrpc":"2.0","id":{id},"method":"{method}","params":{params}}}"#);
    format!("Content-Length: {}\r\n\r\n{content}", content.len())
}

fn make_notify(method: &str, params: &str) -> String {
    let content = format!(r#"{{"jsonrpc":"2.0","method":"{method}","params":{params}}}"#);
    format!("Content-Length: {}\r\n\r\n{content}", content.len())
}

// codemirror
const CM_OPTIONS: &str = r#"{
    "capabilities": {
        "textDocument": {
            "hover": {
                "dynamicRegistration": true,
                "contentFormat": ["plaintext", "markdown"]
            },
            "completion": {
                "dynamicRegistration": true,
                "completionItem": {
                    "commitCharactersSupport": true,
                    "documentationFormat": ["plaintext", "markdown"]
                },
                "contextSupport": false
            },
            "signatureHelp": {
                "dynamicRegistration": true,
                "signatureInformation": {
                    "documentationFormat": ["plaintext", "markdown"]
                }
            },
            "declaration": {
                "dynamicRegistration": true,
                "linkSupport": true
            },
            "definition": {
                "dynamicRegistration": true,
                "linkSupport": true
            },
            "typeDefinition": {
                "dynamicRegistration": true,
                "linkSupport": true
            },
            "implementation": {
                "dynamicRegistration": true,
                "linkSupport": true
            }
        },
        "workspace": {
            "didChangeConfiguration": {
                "dynamicRegistration": true
            }
        },
        "window": {
            "workDoneProgress": true
        }
    },
    "initializationOptions": null,
    "processId": null,
    "rootUri": "file://"#;

const SERVER_INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"positionEncoding":"utf-16","textDocumentSync":{"openClose":true,"change":2,"save":{}},"selectionRangeProvider":true,"hoverProvider":true,"completionProvider":{"resolveProvider":false,"triggerCharacters":[":",".","'","("],"completionItem":{"labelDetailsSupport":false}},"signatureHelpProvider":{"triggerCharacters":["(",",","<"]},"definitionProvider":true,"typeDefinitionProvider":true,"implementationProvider":true,"referencesProvider":true,"documentHighlightProvider":true,"documentSymbolProvider":true,"workspaceSymbolProvider":true,"codeActionProvider":true,"codeLensProvider":{"resolveProvider":true},"documentFormattingProvider":true,"documentRangeFormattingProvider":false,"documentOnTypeFormattingProvider":{"firstTriggerCharacter":".","moreTriggerCharacter":["=","<",">","{","(","|"]},"renameProvider":{"prepareProvider":true},"foldingRangeProvider":true,"declarationProvider":true,"workspace":{"workspaceFolders":{"supported":true,"changeNotifications":true},"fileOperations":{"willRename":{"filters":[{"scheme":"file","pattern":{"glob":"**/*.rs","matches":"file"}},{"scheme":"file","pattern":{"glob":"**","matches":"folder"}}]}}},"callHierarchyProvider":true,"semanticTokensProvider":{"legend":{"tokenTypes":["comment","decorator","enumMember","enum","function","interface","keyword","macro","method","namespace","number","operator","parameter","property","string","struct","typeParameter","variable","angle","arithmetic","attributeBracket","attribute","bitwise","boolean","brace","bracket","builtinAttribute","builtinType","character","colon","comma","comparison","constParameter","const","deriveHelper","derive","dot","escapeSequence","formatSpecifier","generic","invalidEscapeSequence","label","lifetime","logical","macroBang","parenthesis","procMacro","punctuation","selfKeyword","selfTypeKeyword","semicolon","static","toolModule","typeAlias","union","unresolvedReference"],"tokenModifiers":["async","documentation","declaration","static","defaultLibrary","associated","attribute","callable","constant","consuming","controlFlow","crateRoot","injected","intraDocLink","library","macro","mutable","procMacro","public","reference","trait","unsafe"]},"range":true,"full":{"delta":true}},"inlayHintProvider":{"resolveProvider":false},"diagnosticProvider":{"interFileDependencies":true,"workspaceDiagnostics":false},"experimental":{"externalDocs":true,"hoverRange":true,"joinLines":true,"matchingBrace":true,"moveItem":true,"onEnter":true,"openCargoToml":true,"parentModule":true,"runnables":{"kinds":["cargo"]},"ssr":true,"workspaceSymbolScopeKindFiltering":true}},"serverInfo":{"name":"rust-analyzer","version":"1.85.1 (4eb1612 2025-03-15)"}}}"#;

/// Only test in release mode, this is a slow test
#[cfg(not(debug_assertions))]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn rust_analyzer_start() {
    let runner = Runner::new().await.unwrap();
    cargo_init(&runner).await;

    let (child, mut stdin, stdout, stderr) = runner.start_rls().unwrap();

    let a = tokio::spawn(async move {
        let content = make_request(
            1,
            "initialize",
            &format!("{CM_OPTIONS}{}\"}}", runner.home().display()),
        );
        println!("\x1b[34m[STDIN] {content}\x1b[0m");
        stdin.write_all(content.as_bytes()).unwrap();

        _ = tokio::time::sleep(Duration::from_millis(1000)).await;

        let content = make_notify("initialized", "{}");
        println!("\x1b[34m[STDIN] {content}\x1b[0m");
        stdin.write_all(content.as_bytes()).unwrap();

        _ = tokio::time::sleep(Duration::from_millis(1000)).await;

        let content = make_notify("exit", "{}");
        println!("\x1b[34m[STDIN] {content}\x1b[0m");
        stdin.write_all(content.as_bytes()).unwrap();
    });

    let b = tokio::spawn(async move {
        let mut stdout = stdout
            .into_lsp()
            .inspect_ok(|msg| {
                println!("\x1b[32m[STDOUT]: {msg}\x1b[0m");
            })
            .inspect_err(|msg| {
                println!("\x1b[31m[STDOUT/ERROR]: {msg}\x1b[0m");
            });

        let initialize = stdout
            .next()
            .await
            .expect("Should send server initialization")
            .expect("Cannot get stdout");

        assert_eq!(initialize, SERVER_INITIALIZE);

        stdout.map(|_| ()).collect::<()>().await;
        println!("[STDOUT/END]");
    });

    let c = tokio::spawn(async move {
        stderr
            .stream::<1024>()
            .inspect_ok(|msg| {
                println!("\x1b[31m[STDERR]: {}\x1b[0m", String::from_utf8_lossy(&msg));
            })
            .inspect_err(|msg| {
                println!("\x1b[31m[STDERR/ERROR]: {}\x1b[0m", msg);
            })
            .map(|_| ())
            .collect::<()>()
            .await;
        println!("[STDERR/END]");
    });

    let (exit, _, _, _) = tokio::join!(child.async_wait(), a, b, c);

    println!("{exit:#?}");

    panic!();
    assert!(exit.success())
}
