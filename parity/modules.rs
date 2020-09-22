// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::{mpsc, Arc};

use ethcore::{client::BlockChainClient, snapshot::SnapshotService};
use std::collections::BTreeSet;
use sync::{self, ConnectionFilter, NetworkConfiguration, Params, SyncConfig};
use types::BlockNumber;

pub use ethcore::client::ChainNotify;
use ethcore_logger::Config as LogConfig;
pub use sync::{EthSync, ManageNetwork, SyncProvider};

pub type SyncModules = (
    Arc<dyn SyncProvider>,
    Arc<dyn ManageNetwork>,
    Arc<dyn ChainNotify>,
    mpsc::Sender<sync::PriorityTask>,
);

pub fn sync(
    config: SyncConfig,
    network_config: NetworkConfiguration,
    chain: Arc<dyn BlockChainClient>,
    forks: BTreeSet<BlockNumber>,
    snapshot_service: Arc<dyn SnapshotService>,
    _log_settings: &LogConfig,
    connection_filter: Option<Arc<dyn ConnectionFilter>>,
) -> Result<SyncModules, sync::Error> {
    let eth_sync = EthSync::new(
        Params {
            config,
            chain,
            forks,
            snapshot_service,
            network_config,
        },
        connection_filter,
    )?;

    Ok((
        eth_sync.clone() as Arc<dyn SyncProvider>,
        eth_sync.clone() as Arc<dyn ManageNetwork>,
        eth_sync.clone() as Arc<dyn ChainNotify>,
        eth_sync.priority_tasks(),
    ))
}
