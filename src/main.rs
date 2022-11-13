#![deny(clippy::all)]

mod app;
mod parsed_code;

use crate::app::{AppUpdate, MyApp};
use parsed_code::ParsedCode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tokio::sync::watch;
use tokio::sync::watch::Sender;
use tower_lsp::jsonrpc::Result as JsonRpcResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,
    parsed_code: Mutex<HashMap<Url, ParsedCode>>,
    update_sender: Arc<Sender<Result<AppUpdate, String>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> JsonRpcResult<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> JsonRpcResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        if let Ok(mut parsed_code_map) = self.parsed_code.lock() {
            let parsed_code = ParsedCode::new(params.text_document.text);
            self.update_sender
                .send(Ok(AppUpdate {
                    tree: parsed_code.tree.clone(),
                    code: parsed_code.code.clone(),
                }))
                .unwrap();
            parsed_code_map.insert(params.text_document.uri, parsed_code);
        } else {
            self.update_sender
                .send(Err("Error getting lock!".to_string()))
                .unwrap();
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;
        if let Ok(mut parsed_code_map) = self.parsed_code.lock() {
            if let Some(parsed_code) = parsed_code_map.get_mut(&params.text_document.uri) {
                parsed_code.edit(&params.content_changes);
                self.update_sender.send(Ok(AppUpdate {
                    tree: parsed_code.tree.clone(),
                    code: parsed_code.code.clone(),
                })).unwrap();
            } else {
                self.update_sender
                    .send(Err("Error getting parsed code!".to_string()))
                    .unwrap();
            }
        } else {
            self.update_sender
                .send(Err("Error getting lock!".to_string()))
                .unwrap();
        }
    }

    async fn completion(&self, _: CompletionParams) -> JsonRpcResult<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }

    async fn hover(&self, _: HoverParams) -> JsonRpcResult<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
            range: None,
        }))
    }
}

fn main() {
    let rt = Runtime::new().expect("Unable to create Runtime");

    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    let (tx, rx) = watch::channel(Err("not yet initialized!".to_string()));

    let sender = Arc::new(tx);

    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                let stdin = tokio::io::stdin();
                let stdout = tokio::io::stdout();
                let (service, socket) = LspService::new(|client| Backend {
                    client,
                    parsed_code: Mutex::new(HashMap::new()),
                    update_sender: sender.clone(),
                });
                Server::new(stdin, stdout, socket).serve(service).await;
            }
        })
    });

    let mut options = eframe::NativeOptions::default();
    eframe::run_native(
        "FVF Debugger",
        options,
        Box::new(|_cc| Box::new(MyApp::new_with_update_receiver(rx))),
    );
}
