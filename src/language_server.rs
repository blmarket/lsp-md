use std::sync::Mutex;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::document::{
    extract_keywords, find_by_keyword, find_similar, query_section_titles,
    BertModel, CodeFormatter, Document, LspRangeFormat,
};

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
struct KeywordQuery {
    pub uri: Url,
    pub keyword: String,
}

pub struct Backend {
    client: Client,
    encoder: Mutex<BertModel>,
    document_map: DashMap<String, Document>,
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
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),
                document_range_formatting_provider: Some(OneOf::Left(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        String::from("lsp_md/searchSimilar"),
                        String::from("lsp_md/keywords"),
                        String::from("lsp_md/findByKeyword"),
                    ],
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

    async fn code_lens(
        &self,
        params: CodeLensParams,
    ) -> Result<Option<Vec<CodeLens>>> {
        let uri = params.text_document.uri;

        let entry = self.document_map.get(&uri.to_string()).unwrap();
        let section_titles = query_section_titles(entry.value());

        let res = section_titles
            .into_iter()
            .flat_map(|r| {
                vec![
                    CodeLens {
                        range: r,
                        command: Some(Command {
                            title: "Similar docs".to_string(),
                            command: "lsp_md/searchSimilar".to_string(),
                            arguments: Some(vec![json!(Location::new(
                                uri.clone(),
                                r
                            ))]),
                        }),
                        data: None,
                    },
                    CodeLens {
                        range: r,
                        command: Some(Command {
                            title: "Keywords".to_string(),
                            command: "lsp_md/keywords".to_string(),
                            arguments: Some(vec![json!(Location::new(
                                uri.clone(),
                                r
                            ))]),
                        }),
                        data: None,
                    },
                ]
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
        match params.command.as_str() {
            "lsp_md/searchSimilar" => {
                let loc: Location =
                    serde_json::from_value(params.arguments[0].to_owned())
                        .unwrap();
                Ok(Some(json!(find_similar(
                    loc.uri.clone(),
                    self.document_map.get(loc.uri.as_str()).unwrap().value(),
                    &self.encoder,
                    &loc.range.start
                ))))
            },
            "lsp_md/keywords" => {
                let loc: Location =
                    serde_json::from_value(params.arguments[0].to_owned())
                        .unwrap();
                Ok(Some(json!(extract_keywords(
                    self.document_map.get(loc.uri.as_str()).unwrap().value(),
                    &self.encoder,
                    &loc.range.start,
                )
                .unwrap())))
            },
            "lsp_md/findByKeyword" => {
                let query: KeywordQuery =
                    serde_json::from_value(params.arguments[0].to_owned())
                        .unwrap();
                let doc = self.document_map.get(query.uri.as_str()).unwrap();
                let resp = find_by_keyword(
                    query.uri,
                    &self.encoder,
                    doc.value(),
                    &query.keyword,
                );
                Ok(Some(json!(resp)))
            },
            _ => {
                self.client
                    .log_message(MessageType::INFO, "unknown command")
                    .await;
                Ok(None)
            },
        }
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.to_string();
        let Some(doc) = self.document_map.get(&uri) else {
            return Ok(None);
        };

        Ok(CodeFormatter::new(doc.value()).format(params.range))
    }
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
            encoder: Mutex::new(BertModel::default()),
            document_map: DashMap::new(),
        }
    }

    async fn on_change(&self, params: TextDocumentItem) {
        self.document_map.insert(
            params.uri.to_string(),
            Document::parse(&params.text).unwrap(),
        );
    }
}
