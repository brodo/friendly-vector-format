#![deny(clippy::all)]
mod app;
use std::collections::hash_map::Entry::Vacant;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tokio::sync::watch;
use tokio::sync::watch::Sender;
use tree_sitter::{Parser, Language};
use crate::app::{MyApp};


struct Backend {
    client: Client,
    parsers: Mutex<HashMap<Url, Parser>>,
    update_sender: Arc<Sender<String>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }


    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        if let Ok(mut hm) = self.parsers.lock() {
            if let Vacant(e) = hm.entry(params.text_document.uri){
                let mut parser = Parser::new();
                extern "C" { fn tree_sitter_fvf() -> Language; }
                let language = unsafe { tree_sitter_fvf() };
                parser.set_language(language).unwrap();

                if let Some(tree) = parser.parse(params.text_document.text, None) {
                    let root_node = tree.root_node();
                    eprintln!("{:?}", root_node);
                    self.update_sender.send("Parsed successfully!".to_string()).unwrap();
                } else {
                    if let Err(e) = self.update_sender.send("Error getting lock!".to_string()) {
                        eprintln!("Error sending update: {}", e);
                    }

                }
                e.insert( parser);
            }

        } else {
            self.update_sender.send("Error getting lock!".to_string()).unwrap();
        }
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(
                MarkedString::String("You're hovering!".to_string())
            ),
            range: None,
        }))
    }
}

fn main() {
    let rt = Runtime::new().expect("Unable to create Runtime");

    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();



    let (tx, rx) = watch::channel("hello".to_string());

    let sender = Arc::new(tx);

    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                let stdin = tokio::io::stdin();
                let stdout = tokio::io::stdout();
                let (service, socket) =
                    LspService::new(|client| Backend { client, parsers: Mutex::new(HashMap::new()), update_sender: sender.clone() });
                Server::new(stdin, stdout, socket).serve(service).await;
            }
        })
    });



    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "FVF Debugger",
        options,
        Box::new(|_cc| Box::new(MyApp::new_with_ast_receiver(rx))),
    );


}