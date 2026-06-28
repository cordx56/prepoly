//! The Prepoly language server.
//!
//! A stdio JSON-RPC LSP server that drives the Prepoly front end (lex, parse,
//! lower, resolve, type-check) to provide diagnostics, semantic-token
//! highlighting, hover types, and go-to-definition. The heavy path -- per-edit
//! diagnostics -- is incremental: only changed items and their users are
//! re-checked (see [`analysis`]).

mod analysis;
mod backend;
mod document;
mod features;
mod render;
#[cfg(test)]
mod tests;

use std::io;

use tower_lsp_server::{LspService, Server};

use backend::Backend;

/// Send the compiler/server's own logs to stderr; stdout is the LSP transport.
/// Controlled by `PREPOLY_LOG` (the same variable the driver uses), defaulting
/// to warnings only.
fn init_tracing() {
    use tracing_subscriber::filter::LevelFilter;
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .with_env_var("PREPOLY_LOG")
        .from_env_lossy();
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(io::stderr)
        .without_time()
        .try_init();
}

#[tokio::main]
async fn main() {
    init_tracing();
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
