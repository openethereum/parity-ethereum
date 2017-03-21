use std::sync::Arc;
use parity_ipfs_api;
use parity_ipfs_api::error::ServerError;
use ethcore::client::BlockChainClient;
use hyper::server::Listening;

#[derive(Debug, PartialEq, Clone)]
pub struct Configuration {
    pub enabled: bool,
    pub port: u16,
    pub interface: String,
    pub cors: Option<Vec<String>>,
    pub hosts: Option<Vec<String>>,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            enabled: false,
            port: 5001,
            interface: "127.0.0.1".into(),
            cors: None,
            hosts: Some(Vec::new()),
        }
    }
}

pub fn start_server(conf: Configuration, client: Arc<BlockChainClient>) -> Result<Option<Listening>, ServerError> {
    if !conf.enabled {
        return Ok(None);
    }

    parity_ipfs_api::start_server(
        conf.port,
        conf.interface,
        conf.cors,
        conf.hosts,
        client
    ).map(Some)
}
