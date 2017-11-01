use std::sync::{Arc, Weak};

use api::{EthSync, NetworkConfiguration, Params};
use ethcore::client::{BlockChainClient, TestBlockChainClient};
use ethcore::spec::Spec;
use network::NetworkError;
use test_snapshot::TestSnapshotService;

/// Test wrapper around EthSync
pub struct TestSync;

impl TestSync {
    /// Creates a new EthSync with default TestBlockChainClient & TestSnapshotService
    pub fn new() -> Result<Arc<EthSync>, NetworkError> {
        let client = Arc::new(TestBlockChainClient::new());
        Self::new_with_client(client)
    }

    /// Creates new EthSync w/ Kovan TestBlockChainClient & default TestSnapshotService
    pub fn new_kovan() -> Result<Arc<EthSync>, NetworkError> {
        let spec = Spec::new_test_kovan();
        let client = Arc::new(TestBlockChainClient::new_with_spec(spec));
        Self::new_with_client(client)
    }

    /// Creates new EthSync w/ provided BlockChainClient configuration
    pub fn new_with_client(client: Arc<TestBlockChainClient>) -> Result<Arc<EthSync>, NetworkError> {
        let params = Params {
            config: Default::default(),
            chain: client.clone(),
            snapshot_service: Arc::new(TestSnapshotService::new()),
            provider: client.clone(),
            network_config: NetworkConfiguration::new(),
            attached_protos: Vec::new(),
        };

        EthSync::new(params, None)
    }
}
