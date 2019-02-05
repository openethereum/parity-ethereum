// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::{Arc, mpsc};

use ethcore::client::BlockChainClient;
use sync::{self, AttachedProtocol, SyncConfig, NetworkConfiguration, Params, ConnectionFilter};
use ethcore::snapshot::SnapshotService;
use light::Provider;

pub use sync::{EthSync, SyncProvider, ManageNetwork, PrivateTxHandler};
pub use ethcore::client::ChainNotify;
use ethcore_logger::Config as LogConfig;

pub type SyncModules = (
	Arc<SyncProvider>,
	Arc<ManageNetwork>,
	Arc<ChainNotify>,
	mpsc::Sender<sync::PriorityTask>,
);

pub fn sync(
	config: SyncConfig,
	network_config: NetworkConfiguration,
	chain: Arc<BlockChainClient>,
	snapshot_service: Arc<SnapshotService>,
	private_tx_handler: Option<Arc<PrivateTxHandler>>,
	provider: Arc<Provider>,
	_log_settings: &LogConfig,
	attached_protos: Vec<AttachedProtocol>,
	connection_filter: Option<Arc<ConnectionFilter>>,
) -> Result<SyncModules, sync::Error> {
	let eth_sync = EthSync::new(Params {
		config,
		chain,
		provider,
		snapshot_service,
		private_tx_handler,
		network_config,
		attached_protos,
	},
	connection_filter)?;

	Ok((
		eth_sync.clone() as Arc<SyncProvider>,
		eth_sync.clone() as Arc<ManageNetwork>,
		eth_sync.clone() as Arc<ChainNotify>,
		eth_sync.priority_tasks()
	))
}
