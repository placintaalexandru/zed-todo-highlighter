use tower_lsp_server::Client;

use crate::adapters::{
    controllers::lsp::Backend,
    gateways::{highlighter::Highlighter, ripgrep::RipGrepSearcher},
};

pub fn init_server(client: Client) -> Backend<RipGrepSearcher, Highlighter> {
    Backend::new(client, RipGrepSearcher::default(), Highlighter::default())
}
