#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, reason = "this file no test in debug mode")
)]
mod common;

use core::fmt;
use std::io::Write;
use std::time::Duration;

use common::cargo::cargo_init;
use futures::{StreamExt, TryStreamExt};
use rsground_runner::hakoniwa_ext::HakoniwaChildExt;
use rsground_runner::lsp::{LspNotify, LspOutput, LspRequest, LspResponse};
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

fn initialization_options(root_uri: impl fmt::Display) -> String {
    // codemirror capabilities
    format!(
        r#"{{
            "capabilities": {{
                "textDocument": {{
                    "hover":{{"dynamicRegistration":true,"contentFormat":["plaintext", "markdown"]}},
                    "completion":{{"dynamicRegistration":true,"completionItem":{{"commitCharactersSupport":true,"documentationFormat":["plaintext","markdown"]}},"contextSupport":false}},
                    "signatureHelp":{{"dynamicRegistration":true,"signatureInformation":{{"documentationFormat":["plaintext","markdown"]}}}},
                    "declaration":{{"dynamicRegistration":true,"linkSupport":true}},
                    "definition":{{"dynamicRegistration":true,"linkSupport":true}},
                    "typeDefinition":{{"dynamicRegistration":true,"linkSupport":true}},
                    "implementation":{{"dynamicRegistration":true,"linkSupport":true}}
                }},
                "window":{{"workDoneProgress":true}}
            }},
            "initializationOptions": null,
            "processId": null,
            "rootUri": "file://{root_uri}"
        }}"#
    )
}

/// Only test in release mode, this is a slow test
// #[cfg(not(debug_assertions))]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn rust_analyzer_start() {
    let runner = Runner::new().await.unwrap();
    cargo_init(&runner).await;

    let res = Runner::collect_output(
        &mut runner.cmd("/bin/ldd", ["/libexec/rust-analyzer-proc-macro-srv"]),
    )
    .await;
    println!("{res:#?}");

    let res = Runner::collect_output(
        &mut runner.cmd("/libexec/rust-analyzer-proc-macro-srv", [] as [&str; 0]),
    )
    .await;
    println!("{res:#?}");

    // let (child, mut stdin, stdout, stderr) = runner.start_rls().unwrap();
    //
    // let a = tokio::spawn(async move {
    //     let content = make_request(
    //         1,
    //         "initialize",
    //         &initialization_options(runner.home().display()),
    //     );
    //     println!("\x1b[34m[STDIN] {content}\x1b[0m");
    //     stdin.write_all(content.as_bytes()).unwrap();
    //
    //     let content = make_notify("initialized", "{}");
    //     println!("\x1b[34m[STDIN] {content}\x1b[0m");
    //     stdin.write_all(content.as_bytes()).unwrap();
    //
    //     _ = tokio::time::sleep(Duration::from_millis(2000)).await;
    //
    //     let content = make_notify("exit", "{}");
    //     println!("\x1b[34m[STDIN] {content}\x1b[0m");
    //     stdin.write_all(content.as_bytes()).unwrap();
    // });
    //
    // let b = tokio::spawn(async move {
    //     let mut stdout = stdout
    //         .into_lsp()
    //         .inspect_ok(|msg| match msg {
    //             LspOutput::Response(LspResponse::Ok { id, result, .. }) => {
    //                 println!("\x1b[32m[STDOUT/RESPONSE] ({id}): {result:?}\x1b[0m");
    //             }
    //             LspOutput::Response(LspResponse::Err { id, error, .. }) => {
    //                 println!("\x1b[31m[STDOUT/RESPONSE] ({id}): {error:?}\x1b[0m");
    //             }
    //             LspOutput::Request(LspRequest {
    //                 id, method, params, ..
    //             }) => {
    //                 println!("\x1b[32m[STDOUT/REQUEST] ({id}/{method}): {params}\x1b[0m");
    //             }
    //             LspOutput::Notify(LspNotify { method, params, .. }) => {
    //                 println!("\x1b[32m[STDOUT/NOTIFY] ({method}): {params}\x1b[0m");
    //             }
    //         })
    //         .inspect_err(|msg| {
    //             println!("\x1b[31m[STDOUT/ERROR]: {msg}\x1b[0m");
    //         });
    //
    //     let initialize = stdout
    //         .next()
    //         .await
    //         .expect("Should send server initialization")
    //         .expect("Cannot get stdout");
    //
    //     let initialize = initialize
    //         .as_response()
    //         .expect("Should send server initialization");
    //
    //     assert_eq!(initialize.id(), 1, "Initialize request made from id 1");
    //
    //     assert!(initialize.is_ok(), "should be success");
    //
    //     stdout.map(|_| ()).collect::<()>().await;
    //     println!("[STDOUT/END]");
    // });
    //
    // let c = tokio::spawn(async move {
    //     stderr
    //         .stream::<1024>()
    //         .inspect_ok(|msg| {
    //             println!("\x1b[31m[STDERR]: {}\x1b[0m", String::from_utf8_lossy(&msg));
    //         })
    //         .inspect_err(|msg| {
    //             println!("\x1b[31m[STDERR/ERROR]: {}\x1b[0m", msg);
    //         })
    //         .map(|_| ())
    //         .collect::<()>()
    //         .await;
    //     println!("[STDERR/END]");
    // });
    //
    // let (exit, _, _, _) = tokio::join!(child.async_wait(), a, b, c);
    //
    // println!("{exit:#?}");

    panic!();
    // assert!(exit.success())
}
