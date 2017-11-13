// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;

use ethcore::client::BlockChainClient;
use ethsync::{self, AttachedProtocol, SyncConfig, NetworkConfiguration, Params, ConnectionFilter};
use ethcore::snapshot::SnapshotService;
use light::Provider;

pub use ethsync::{EthSync, SyncProvider, ManageNetwork};
pub use ethcore::client::ChainNotify;
use ethcore_logger::Config as LogConfig;

pub type SyncModules = (Arc<SyncProvider>, Arc<ManageNetwork>, Arc<ChainNotify>);

pub fn sync(
	sync_cfg: SyncConfig,
	net_cfg: NetworkConfiguration,
	client: Arc<BlockChainClient>,
	snapshot_service: Arc<SnapshotService>,
	provider: Arc<Provider>,
	_log_settings: &LogConfig,
	attached_protos: Vec<AttachedProtocol>,
	connection_filter: Option<Arc<ConnectionFilter>>,
) -> Result<SyncModules, ethsync::Error> {
	let eth_sync = EthSync::new(Params {
		config: sync_cfg,
		chain: client,
		provider: provider,
		snapshot_service: snapshot_service,
		network_config: net_cfg,
		attached_protos: attached_protos,
	},
	connection_filter)?;

	Ok((eth_sync.clone() as Arc<SyncProvider>, eth_sync.clone() as Arc<ManageNetwork>, eth_sync.clone() as Arc<ChainNotify>))
}
