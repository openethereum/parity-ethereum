pub use parity_ipfs_api::start_server;

#[derive(Debug, PartialEq, Clone)]
pub struct Configuration {
    pub enabled: bool,
    pub port: u16,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            enabled: false,
            port: 5001,
        }
    }
}
