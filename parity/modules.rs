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

use client_traits::{BlockChainClient, ChainNotify};
use sync::{self, SyncConfig, NetworkConfiguration, Params, ConnectionFilter};
use snapshot::SnapshotService;
use ethcore_private_tx::PrivateStateDB;
use light::Provider;
use parity_runtime::Executor;

pub use sync::{EthSync, SyncProvider, ManageNetwork, PrivateTxHandler};
use ethcore_logger::Config as LogConfig;

pub type SyncModules = (
	Arc<dyn SyncProvider>,
	Arc<dyn ManageNetwork>,
	Arc<dyn ChainNotify>,
	mpsc::Sender<sync::PriorityTask>,
);

pub fn sync(
	config: SyncConfig,
	executor: Executor,
	network_config: NetworkConfiguration,
	chain: Arc<dyn BlockChainClient>,
	snapshot_service: Arc<dyn SnapshotService>,
	private_tx_handler: Option<Arc<dyn PrivateTxHandler>>,
	private_state: Option<Arc<PrivateStateDB>>,
	provider: Arc<dyn Provider>,
	_log_settings: &LogConfig,
	connection_filter: Option<Arc<dyn ConnectionFilter>>,
) -> Result<SyncModules, sync::Error> {
	let eth_sync = EthSync::new(Params {
		config,
		executor,
		chain,
		provider,
		snapshot_service,
		private_tx_handler,
		private_state,
		network_config,
	},
	connection_filter)?;

	Ok((
		eth_sync.clone() as Arc<dyn SyncProvider>,
		eth_sync.clone() as Arc<dyn ManageNetwork>,
		eth_sync.clone() as Arc<dyn ChainNotify>,
		eth_sync.priority_tasks()
	))
}
