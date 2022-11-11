use std::collections::HashMap;
use std::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tree_sitter::{Parser, Language};

struct Backend {
    client: Client,
    parser: Mutex<HashMap<Url,Parser>>,
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

        if let Ok(mut hm) = self.parser.lock() {
            if !hm.contains_key(&params.text_document.uri){
                let mut parser = Parser::new();
                extern "C" { fn tree_sitter_fvf() -> Language; }
                let language = unsafe { tree_sitter_fvf() };
                parser.set_language(language).unwrap();

                if let Some(tree) = parser.parse(params.text_document.text, None) {
                    let root_node = tree.root_node();
                    eprintln!("{:?}", root_node);
                }

                hm.insert(params.text_document.uri, parser);
            }

        } else {
            eprintln!("Could not lock parser!");
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

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) =
        LspService::new(|client| Backend { client, parser: Mutex::new(HashMap::new()) });
    Server::new(stdin, stdout, socket).serve(service).await;
}