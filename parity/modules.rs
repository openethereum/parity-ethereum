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

use std::sync::Arc;
use ethcore::client::BlockChainClient;
use ethcore;
use hypervisor::Hypervisor;
use ethsync::{SyncConfig, NetworkConfiguration};
#[cfg(not(feature="ipc"))]
use self::no_ipc_deps::*;
#[cfg(feature="ipc")]
use self::ipc_deps::*;

#[cfg(not(feature="ipc"))]
mod no_ipc_deps {
	pub use ethsync::{EthSync, SyncProvider, ManageNetwork};
	pub use ethcore::client::ChainNotify;
}

#[cfg(feature="ipc")]
mod ipc_deps {
	pub use ethsync::{SyncClient, NetworkManagerClient};
	pub use ethcore::client::ChainNotifyClient;
	pub use hypervisor::{SYNC_MODULE_ID, BootArgs};
	pub use nanoipc::{GuardedSocket, NanoSocket, init_client};
	pub use ipc::IpcSocket;
}


#[cfg(feature="ipc")]
pub fn hypervisor() -> Option<Hypervisor> {
	Some(Hypervisor::new())
}

#[cfg(not(feature="ipc"))]
pub fn hypervisor() -> Option<Hypervisor> {
	None
}

#[cfg(feature="ipc")]
fn sync_arguments(sync_cfg: SyncConfig, net_cfg: NetworkConfiguration) -> Vec<String> {
	let mut result = Vec::new();
	result.push("ipc:///tmp/parity-chain.ipc".to_owned());
	result.push(format!("{}", sync_cfg.network_id));
	result.push(format!("{}", net_cfg.listen_address.unwrap_or("0.0.0.0:30303".to_owned())));
	result.push(format!("{}", match net_cfg.nat_enabled { true => "yes".to_owned(), false => "no".to_owned() }));
	result.push(format!("{}", match net_cfg.discovery_enabled { true => "yes".to_owned(), false => "no".to_owned() }));
	result.push(format!("{}", net_cfg.ideal_peers));
	result.push(format!("{}", net_cfg.config_path.unwrap_or(".".to_owned())));
	result.push(format!("{}", match net_cfg.allow_non_reserved { true => "yes".to_owned(), false => "no".to_owned() }));

	result
}

#[cfg(feature="ipc")]
pub fn sync (
	hypervisor_ref: &mut Option<Hypervisor>,
	sync_cfg: SyncConfig,
	net_cfg: NetworkConfiguration,
	_client: Arc<BlockChainClient>)
	-> Result<
		(
			GuardedSocket<SyncClient<NanoSocket>>,
			GuardedSocket<NetworkManagerClient<NanoSocket>>,
			GuardedSocket<ChainNotifyClient<NanoSocket>>
		),
		ethcore::error::Error>
{
	let mut hypervisor = hypervisor_ref.take().expect("There should be hypervisor for ipc configuration");
	hypervisor = hypervisor.module(SYNC_MODULE_ID, "sync", BootArgs::new().cli(sync_arguments(sync_cfg, net_cfg)));

	hypervisor.start();
	hypervisor.wait_for_startup();

	let sync_client = init_client::<SyncClient<_>>("ipc:///tmp/parity-sync.ipc").unwrap();
	let notify_client = init_client::<ChainNotifyClient<_>>("ipc:///tmp/parity-sync-notify.ipc").unwrap();
	let manage_client = init_client::<NetworkManagerClient<_>>("ipc:///tmp/parity-manage-net.ipc").unwrap();

	*hypervisor_ref = Some(hypervisor);
	Ok((sync_client, manage_client, notify_client))
}

#[cfg(not(feature="ipc"))]
pub fn sync(
	_hypervisor: &mut Option<Hypervisor>,
	sync_cfg: SyncConfig,
	net_cfg: NetworkConfiguration,
	client: Arc<BlockChainClient>)
	-> Result<(Arc<SyncProvider>, Arc<ManageNetwork>, Arc<ChainNotify>), ethcore::error::Error>
{
	let eth_sync = try!(EthSync::new(sync_cfg, client, net_cfg).map_err(|e| ethcore::error::Error::Util(e)));
	Ok((eth_sync.clone() as Arc<SyncProvider>, eth_sync.clone() as Arc<ManageNetwork>, eth_sync.clone() as Arc<ChainNotify>))
}
