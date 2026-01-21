use tower_lsp_server::Client;

use crate::adapters::{
    config::Config,
    controllers::lsp::{Backend, new_server},
    gateways::{color_provider::ColorProvider, ripgrep::RipGrepSearcher},
};

pub fn init_server(client: Client) -> Backend<RipGrepSearcher, ColorProvider> {
    new_server(client, Config::default())
}
