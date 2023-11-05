use std::collections::HashMap;

use dashmap::DashMap;
use nrs_language_server::chumsky::{
    parse, type_inference, Func, ImCompleteSemanticToken, ParserResult,
};
use nrs_language_server::completion::completion;
use nrs_language_server::jump_definition::get_definition;
use nrs_language_server::reference::get_reference;
use nrs_language_server::semantic_token::{semantic_token_from_ast, LEGEND_TYPE};
use ropey::Rope;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    ast_map: DashMap<String, HashMap<String, Func>>,
    document_map: DashMap<String, Rope>,
    semantic_token_map: DashMap<String, Vec<ImCompleteSemanticToken>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            offset_encoding: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                references_provider: Some(OneOf::Left(true)),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(true),
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: params.text_document.text,
            version: params.text_document.version,
        })
        .await
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            version: params.text_document.version,
        })
        .await
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        self.client
            .log_message(MessageType::INFO, "references")
            .await;
        let reference_list = || -> Option<Vec<Location>> {
            let uri = params.text_document_position.text_document.uri;
            let loc = Location::new(
                uri.clone(),
                Range::new(Position::new(0, 0), Position::new(0, 1)),
            );
            Some(vec![loc])
            // let ast = self.ast_map.get(&uri.to_string())?;
            // let rope = self.document_map.get(&uri.to_string())?;

            // let position = params.text_document_position.position;
            // let char = rope.try_line_to_char(position.line as usize).ok()?;
            // let offset = char + position.character as usize;
            // let reference_list = get_reference(&ast, offset, false);
            // let mut ret = reference_list
            //     .into_iter()
            //     .filter_map(|(_, range)| {
            //         let start_position = offset_to_position(range.start, &rope)?;
            //         let end_position = offset_to_position(range.end, &rope)?;

            //         let range = Range::new(start_position, end_position);

            //         Some(Location::new(uri.clone(), range))
            //     })
            //     .collect::<Vec<_>>();
            // Some(ret)
        }();
        self.client
            .log_message(
                MessageType::INFO,
                format!("references: {:?}", reference_list),
            )
            .await;
        Ok(reference_list)
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        self.client
            .log_message(MessageType::INFO, "command executed!")
            .await;

        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let uri = params.text_document.uri;

        let code_lenses = vec![CodeLens {
            range: Range {
                start: Position {
                    line: 4,
                    character: 0,
                },
                end: Position {
                    line: 4,
                    character: 0,
                },
            },
            command: Some(Command {
                title: "Search similar documents".to_string(),
                command: "dummy.do_something".to_string(),
                arguments: Some(vec![Value::String(uri.to_string())]),
            }),
            data: None,
        }];
        Ok(Some(code_lenses))
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct InlayHintParams {
    path: String,
}

enum CustomNotification {}
impl Notification for CustomNotification {
    type Params = InlayHintParams;
    const METHOD: &'static str = "custom/notification";
}
struct TextDocumentItem {
    uri: Url,
    text: String,
    version: i32,
}

impl Backend {
    async fn on_change(&self, params: TextDocumentItem) {
        let rope = ropey::Rope::from_str(&params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        ast_map: DashMap::new(),
        document_map: DashMap::new(),
        semantic_token_map: DashMap::new(),
    })
    .finish();

    serde_json::json!({"test": 20});
    Server::new(stdin, stdout, socket).serve(service).await;
}

fn offset_to_position(offset: usize, rope: &Rope) -> Option<Position> {
    let line = rope.try_char_to_line(offset).ok()?;
    let first_char_of_line = rope.try_line_to_char(line).ok()?;
    let column = offset - first_char_of_line;
    Some(Position::new(line as u32, column as u32))
}
