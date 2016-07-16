// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use ethsync::{EthSync, SyncProvider, ManageNetwork, SyncConfig, NetworkConfiguration};
use std::sync::Arc;
use ethcore::client::{ChainNotify, BlockChainClient};
use ethcore;

#[cfg(feature="ipc")]
fn sync(
	sync_cfg: SyncConfig,
	net_cfg: NetworkConfiguration,
	client: Arc<BlockChainClient>)
	-> Result<(Arc<SyncProvider>, Arc<ManageNetwork>, Arc<ChainNotify>), ethcore::error::Error>
{

}

#[cfg(not(feature="ipc"))]
fn sync(
	sync_cfg: SyncConfig,
	net_cfg: NetworkConfiguration,
	client: Arc<BlockChainClient>)
	-> Result<(Arc<SyncProvider>, Arc<ManageNetwork>, Arc<ChainNotify>), ethcore::error::Error>
{
	let eth_sync = try!(EthSync::new(sync_cfg, client, net_cfg).map_err(|e| ethcore::error::Error::Util(e)));
	Ok((eth_sync.clone() as Arc<SyncProvider>, eth_sync.clone() as Arc<ManageNetwork>, eth_sync.clone() as Arc<ChainNotify>))
}
