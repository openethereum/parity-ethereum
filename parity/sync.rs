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

use std;
use std::sync::Arc;
use hypervisor::{SYNC_MODULE_ID, HYPERVISOR_IPC_URL};
use ethcore::client::{RemoteClient, ChainNotify};
use ethsync::{SyncProvider, EthSync, ManageNetwork, ServiceConfiguration};
use std::thread;
use modules::service_urls;
use boot;

pub fn main() {
	boot::setup_cli_logger("sync");

	let service_config: ServiceConfiguration = boot::payload()
		.unwrap_or_else(|e| panic!("Fatal: error reading boot arguments ({:?})", e));

	let remote_client = dependency!(RemoteClient, &service_urls::with_base(&service_config.io_path, service_urls::CLIENT));

	let stop = boot::main_thread();
	let sync = EthSync::new(service_config.sync, remote_client.service().clone(), service_config.net).unwrap();

	let _ = boot::register(
		&service_urls::with_base(&service_config.io_path, HYPERVISOR_IPC_URL),
		SYNC_MODULE_ID
	);

	boot::host_service(
		&service_urls::with_base(&service_config.io_path, service_urls::SYNC),
		stop.clone(),
		sync.clone() as Arc<SyncProvider>
	);
	boot::host_service(
		&service_urls::with_base(&service_config.io_path, service_urls::NETWORK_MANAGER),
		stop.clone(),
		sync.clone() as Arc<ManageNetwork>
	);
	boot::host_service(
		&service_urls::with_base(&service_config.io_path, service_urls::SYNC_NOTIFY),
		stop.clone(),
		sync.clone() as Arc<ChainNotify>
	);

	while !stop.load(::std::sync::atomic::Ordering::Relaxed) {
		thread::park_timeout(std::time::Duration::from_millis(1000));
	}
}
