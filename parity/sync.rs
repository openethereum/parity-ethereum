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

use nanoipc;
use ipc;
use std;
use std::sync::Arc;
use hypervisor::{HypervisorServiceClient, SYNC_MODULE_ID, HYPERVISOR_IPC_URL};
use ctrlc::CtrlC;
use std::sync::atomic::{AtomicBool, Ordering};
use docopt::Docopt;
use ethcore::client::{RemoteClient, ChainNotify};
use ethsync::{SyncProvider, EthSync, ManageNetwork, ServiceConfiguration};
use std::thread;
use nanoipc::IpcInterface;
use modules::service_urls;
use ethcore_logger::{Config as LogConfig, setup_log};

const USAGE: &'static str = "
Ethcore sync service
Usage:
  parity sync [options]

 Options:
  -l --logging LOGGING     Specify the logging level. Must conform to the same
                           format as RUST_LOG.
  --log-file FILENAME      Specify a filename into which logging should be
                           directed.
  --no-color               Don't use terminal color codes in output.
";

#[derive(Debug, RustcDecodable)]
struct Args {
	flag_logging: Option<String>,
	flag_log_file: Option<String>,
	flag_no_color: bool,
}

impl Args {
	pub fn log_settings(&self) -> LogConfig {
		LogConfig {
			color: self.flag_no_color || cfg!(windows),
			mode: self.flag_logging.clone(),
			file: self.flag_log_file.clone(),
		}
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

pub fn main() {
	use std::io::{self, Read};

	let args: Args = Docopt::new(USAGE)
		.and_then(|d| d.decode())
		.unwrap_or_else(|e| e.exit());

	setup_log(&args.log_settings()).expect("Log initialization failure");

	let mut buffer = Vec::new();
	io::stdin().read_to_end(&mut buffer).expect("Failed to read initialisation payload");
	let service_config = ipc::binary::deserialize::<ServiceConfiguration>(&buffer).expect("Failed deserializing initialisation payload");

	let remote_client = nanoipc::init_client::<RemoteClient<_>>(service_urls::CLIENT).unwrap();

	remote_client.handshake().unwrap();

	let stop = Arc::new(AtomicBool::new(false));
	let sync = EthSync::new(service_config.sync, remote_client.service().clone(), service_config.net).unwrap();

	run_service(service_urls::SYNC, stop.clone(), sync.clone() as Arc<SyncProvider>);
	run_service(service_urls::NETWORK_MANAGER, stop.clone(), sync.clone() as Arc<ManageNetwork>);
	run_service(service_urls::SYNC_NOTIFY, stop.clone(), sync.clone() as Arc<ChainNotify>);

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
