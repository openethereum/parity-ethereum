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

//! Parity sync service

extern crate ethcore_ipc_nano as nanoipc;
extern crate ethcore_ipc_hypervisor as hypervisor;
extern crate ethcore_ipc as ipc;
extern crate ctrlc;
#[macro_use] extern crate log;
extern crate ethsync;
extern crate rustc_serialize;
extern crate docopt;
extern crate ethcore;
extern crate ethcore_util as util;

use std::sync::Arc;
use hypervisor::{HypervisorServiceClient, SYNC_MODULE_ID, HYPERVISOR_IPC_URL};
use ctrlc::CtrlC;
use std::sync::atomic::{AtomicBool, Ordering};
use docopt::Docopt;
use ethcore::client::{RemoteClient, ChainNotify};
use ethsync::{SyncProvider, SyncConfig, EthSync, ManageNetwork, NetworkConfiguration};
use std::thread;
use util::numbers::{U256, H256};
use std::str::FromStr;
use nanoipc::IpcInterface;

const USAGE: &'static str = "
Ethcore sync service
Usage:
  sync <client-url> <network-id> <listen-address> <nat-enabled> <discovery-enabled> <ideal-peers> <config-path> <allow-non-reserved> [options]

Options:
  --public-address IP      Public address.
  --boot-nodes LIST        List of boot nodes.
  --reserved-nodes LIST    List of reserved peers,
  --secret HEX             Use node key hash
  --udp-port               UDP port
";

#[derive(Debug, RustcDecodable)]
struct Args {
	arg_network_id: String,
	arg_listen_address: String,
	arg_nat_enabled: bool,
	arg_discovery_enabled: bool,
	arg_ideal_peers: u32,
	arg_config_path: String,
	arg_client_url: String,
	arg_allow_non_reserved: bool,
	flag_public_address: Option<String>,
	flag_secret: Option<String>,
	flag_boot_nodes: Vec<String>,
	flag_reserved_nodes: Vec<String>,
	flag_udp_port: Option<u16>,
}

impl Args {
	pub fn into_config(self) -> (SyncConfig, NetworkConfiguration, String) {
		let mut sync_config = SyncConfig::default();
		sync_config.network_id = U256::from_str(&self.arg_network_id).unwrap();

		let network_config = NetworkConfiguration {
			udp_port: self.flag_udp_port,
			nat_enabled: self.arg_nat_enabled,
			boot_nodes: self.flag_boot_nodes,
			listen_address: Some(self.arg_listen_address),
			public_address: self.flag_public_address,
			use_secret: self.flag_secret.as_ref().map(|s| H256::from_str(s).unwrap()),
			discovery_enabled: self.arg_discovery_enabled,
			ideal_peers: self.arg_ideal_peers,
			config_path: Some(self.arg_config_path),
			reserved_nodes: self.flag_reserved_nodes,
			allow_non_reserved: self.arg_allow_non_reserved,
		};

		(sync_config, network_config, self.arg_client_url)
	}
}

fn run_service<T: ?Sized + Send + Sync + 'static>(addr: &str, stop_guard: Arc<AtomicBool>, service: Arc<T>) where T: IpcInterface {
	let socket_url = addr.to_owned();
	std::thread::spawn(move || {
		let mut worker = nanoipc::Worker::<T>::new(&service);
		worker.add_reqrep(&socket_url).unwrap();

		while !stop_guard.load(Ordering::Relaxed) {
			worker.poll();
		}
	});
}

fn main() {
	let args: Args = Docopt::new(USAGE)
		.and_then(|d| d.decode())
		.unwrap_or_else(|e| e.exit());
	let (sync_config, network_config, client_url) = args.into_config();
	let remote_client = nanoipc::init_client::<RemoteClient<_>>(&client_url).unwrap();

	remote_client.handshake().unwrap();

	let stop = Arc::new(AtomicBool::new(false));
	let sync = EthSync::new(sync_config, remote_client.service().clone(), network_config).unwrap();

	run_service("ipc:///tmp/parity-sync.ipc", stop.clone(), sync.clone() as Arc<SyncProvider>);
	run_service("ipc:///tmp/parity-manage-net.ipc", stop.clone(), sync.clone() as Arc<ManageNetwork>);
	run_service("ipc:///tmp/parity-sync-notify.ipc", stop.clone(), sync.clone() as Arc<ChainNotify>);

	let hypervisor_client = nanoipc::init_client::<HypervisorServiceClient<_>>(HYPERVISOR_IPC_URL).unwrap();
	hypervisor_client.handshake().unwrap();
	hypervisor_client.module_ready(SYNC_MODULE_ID);

	let terminate_stop = stop.clone();
	CtrlC::set_handler(move || {
		terminate_stop.store(true, Ordering::Relaxed);
	});

	while !stop.load(Ordering::Relaxed) {
		thread::park_timeout(std::time::Duration::from_millis(1000));
	}
}
