use std::sync::Arc;

use tokio::sync::RwLock;
use tower_lsp_server::{
    Client, LanguageServer,
    jsonrpc::{Error, Result},
    ls_types::{
        ColorInformation, ColorProviderCapability, DidChangeTextDocumentParams,
        DocumentColorParams, InitializeParams, InitializeResult, InitializedParams, MessageType,
        OneOf, Range, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
        TextDocumentSyncKind, TextDocumentSyncOptions, VersionedTextDocumentIdentifier,
        WorkspaceFoldersServerCapabilities, WorkspaceServerCapabilities,
    },
};

use crate::{
    adapters::{
        config::Config,
        controllers::{highlight::Highlight, search::Search},
        gateways::{color_provider::ColorProvider, ripgrep::RipGrepSearcher},
        presenters::{ColorPresenter, PositionPresenter},
    },
    entities::{Color, ColorType, Colors, Position, State, TodoResult},
    use_cases::ports::{Colorer, Conversion, RegexSearcher, Searcher},
};

struct Protected<S, G, H> {
    state: S,
    grep: G,
    highlighter: H,
}

impl<S, G, H> Protected<S, G, H> {
    pub fn new(state: S, grep: G, highlighter: H) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            state,
            grep,
            highlighter,
        }))
    }
}

pub struct Backend<S, C>
where
    S: Searcher,
    C: Colorer,
{
    client: Client,
    #[allow(clippy::type_complexity)]
    protected: Arc<RwLock<Protected<State, Search<S>, Highlight<C>>>>,
}

impl<S, C> Backend<S, C>
where
    S: RegexSearcher,
    C: Colorer,
{
    async fn update_colors(&self, config: &Config) {
        self.protected
            .write()
            .await
            .highlighter
            .update_palette(config.highlights.clone());
    }

    async fn update_regex(&self) -> TodoResult<()> {
        // clone the keys while holding a read lock, then acquire a write lock
        // only for the `grep.update_regex` call to avoid borrow conflicts
        let read = self.protected.read().await;
        let owned_keys: Vec<String> = read.highlighter.colors().keys().cloned().collect();
        drop(read);

        let keys_ref: Vec<&str> = owned_keys.iter().map(|s| s.as_str()).collect();
        self.protected.write().await.grep.update_regex(&keys_ref)?;

        Ok(())
    }

    async fn init(&self, params: InitializeParams) -> Result<()> {
        let config = Config::parse_json(params.initialization_options.unwrap_or_default());
        self.client
            .log_message(MessageType::LOG, format!("{config:?}"))
            .await;
        self.update_colors(&config).await;
        self.update_regex()
            .await
            .map_err(|e| Error::invalid_params(format!("{e:?}")))?;

        match params.workspace_folders {
            Some(ref folders) => {
                if folders.is_empty() {
                    return Err(Error::invalid_params("Workspace folders are empty"));
                }

                let uri = &folders[0].uri;
                let intial_state = self
                    .protected
                    .read()
                    .await
                    .grep
                    .recurssive_search(uri.path().as_str());

                self.protected.write().await.state.extend(intial_state);

                Ok(())
            }
            None => Err(Error::invalid_params("No workspace folder to operate on")),
        }
    }
}

impl<S, C> LanguageServer for Backend<S, C>
where
    S: RegexSearcher + Send + Sync + 'static,
    C: Colorer + Send + Sync + 'static,
{
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        const LSP_NAME: &str = "Todo Tree";

        self.init(params).await?;

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: LSP_NAME.into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
            capabilities: ServerCapabilities {
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                color_provider: Some(ColorProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
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

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let DidChangeTextDocumentParams {
            text_document,
            content_changes,
        } = params;
        let VersionedTextDocumentIdentifier { uri, .. } = text_document;

        if let Some(last_change) = content_changes.last() {
            let file_path = uri.path().as_str();
            let new_matches = self
                .protected
                .read()
                .await
                .grep
                .search_in_text(&last_change.text);

            match new_matches {
                Some(new_matches) => self
                    .protected
                    .write()
                    .await
                    .state
                    .replace(file_path.to_owned(), new_matches),
                None => self.protected.write().await.state.remove(file_path),
            }
        }
    }

    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        let protected = self.protected.read().await;

        let highlights = protected
            .state
            .get(params.text_document.uri.path().as_str())
            .into_iter()
            .flat_map(|file_state| {
                file_state
                    .rows()
                    .iter()
                    .map(|(row, (row_meta, row_matches))| {
                        let intervals =
                            protected.highlighter.color_intervals(row_matches, row_meta);

                        intervals.into_iter().enumerate().map(
                            |(i, (color_patch_start, color_patch_end))| {
                                let start_pos = Position::new(*row, color_patch_start);
                                let end_pos = Position::new(*row, color_patch_end);
                                let color = protected
                                    .highlighter
                                    .highlight(row_matches[i].keyword(), ColorType::Background)
                                    .unwrap();

                                ColorInformation {
                                    range: Range {
                                        start: PositionPresenter::convert(start_pos),
                                        end: PositionPresenter::convert(end_pos),
                                    },
                                    color: ColorPresenter::convert(color),
                                }
                            },
                        )
                    })
            })
            .flatten()
            .collect();

        Ok(highlights)
    }
}

pub fn new_server(client: Client, config: Config) -> Backend<RipGrepSearcher, ColorProvider> {
    let state = State::default();

    let key_words = config.highlights.keys().collect::<Vec<_>>();
    let searcher = Search::new(
        RipGrepSearcher::try_from_key_words(&key_words).expect("Could not initialize searcher"),
    );

    let background_colors = config
        .highlights
        .into_iter()
        .map(|(key_word, user_colors)| {
            let background = Color::new(
                user_colors.background.r,
                user_colors.background.g,
                user_colors.background.b,
                user_colors.background.a,
            );
            let colors = Colors::new(background);

            (key_word, colors)
        })
        .collect();
    let highlighter = Highlight::new(ColorProvider::new(background_colors));

    let protected = Protected::new(state, searcher, highlighter);

    Backend { client, protected }
}
