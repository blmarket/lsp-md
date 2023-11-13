use std::sync::Mutex;

use dashmap::DashMap;
use ropey::Rope;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::document::{
    find_similar, find_similar2, query_section_titles, Document, BertModels,
};

pub struct Backend {
    client: Client,
    encoder: Mutex<BertModels>,
    document_map: DashMap<String, Rope>,
    section_map: DashMap<String, Document>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(
        &self,
        _: InitializeParams,
    ) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            offset_encoding: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                references_provider: Some(OneOf::Left(true)),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["dummy.do_something".to_string()],
                    work_done_progress_options: Default::default(),
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

    async fn references(
        &self,
        params: ReferenceParams,
    ) -> Result<Option<Vec<Location>>> {
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "references: {:?}",
                    params.text_document_position.position
                ),
            )
            .await;

        let uri = params.text_document_position.text_document.uri;
        let entry = self.section_map.get(&uri.to_string()).unwrap();

        let ranges = find_similar(
            entry.value(),
            &self.encoder,
            params.text_document_position.position,
        );
        self.client
            .log_message(
                MessageType::INFO,
                format!("references:: result: {:?}", ranges),
            )
            .await;

        Ok(ranges
            .into_iter()
            .map(|r| Some(Location::new(uri.clone(), r)))
            .collect::<Option<Vec<Location>>>())
    }

    async fn code_lens(
        &self,
        params: CodeLensParams,
    ) -> Result<Option<Vec<CodeLens>>> {
        let uri = params.text_document.uri;

        let entry = self.section_map.get(&uri.to_string()).unwrap();
        let section_titles = query_section_titles(entry.value());

        let res = section_titles
            .into_iter()
            .flat_map(|r| {
                vec![CodeLens {
                    range: r,
                    command: Some(Command {
                        title: "Search similar documents".to_string(),
                        command: "dummy.do_something".to_string(),
                        arguments: Some(vec![json!(Location::new(
                            uri.clone(),
                            r
                        ))]),
                    }),
                    data: None,
                }]
            })
            .collect();

        Ok(Some(res))
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_workspace_folders(
        &self,
        _: DidChangeWorkspaceFoldersParams,
    ) {
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
    }

    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<Value>> {
        self.client
            .log_message(
                MessageType::INFO,
                format!("command executed!: {:?}", params),
            )
            .await;

        match params.command.as_str() {
            "dummy.do_something" => {
                self.client
                    .log_message(MessageType::INFO, "dummy.do_something")
                    .await;
                dbg!(&params);
                let loc: Location =
                    serde_json::from_value(params.arguments[0].to_owned())
                        .unwrap();
                self.client
                    .log_message(MessageType::INFO, format!("loc: {:?}", &loc))
                    .await;
                Ok(Some(json!(find_similar2(
                    loc.uri.clone(),
                    self.section_map.get(loc.uri.as_str()).unwrap().value(),
                    &self.encoder,
                    loc.range.start
                ))))
            },
            _ => {
                self.client
                    .log_message(MessageType::INFO, "unknown command")
                    .await;
                Ok(None)
            },
        }
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
    #[allow(dead_code)]
    version: i32,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Backend {
            client,
            encoder: Mutex::new(BertModels::default()),
            document_map: DashMap::new(),
            section_map: DashMap::new(),
        }
    }

    async fn on_change(&self, params: TextDocumentItem) {
        let rope = ropey::Rope::from_str(&params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
        self.section_map.insert(
            params.uri.to_string(),
            Document::parse(params.text).unwrap(),
        );
    }
}
