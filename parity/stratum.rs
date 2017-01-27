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

//! Parity sync service

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use ethcore_stratum::{Stratum as StratumServer, PushWorkHandler, RemoteJobDispatcher, ServiceConfiguration};
use modules::service_urls;
use boot;
use hypervisor::service::IpcModuleId;
use hypervisor::{HYPERVISOR_IPC_URL, ControlService};
use std::net::{SocketAddr, IpAddr};
use std::str::FromStr;
use nanoipc;
use std::thread;
use ethcore::miner::stratum::{STRATUM_SOCKET_NAME, JOB_DISPATCHER_SOCKET_NAME};

pub const MODULE_ID: IpcModuleId = 8000;

#[derive(Default)]
struct StratumControlService {
	pub stop: Arc<AtomicBool>,
}

impl ControlService for StratumControlService {
	fn shutdown(&self) -> bool {
		trace!(target: "hypervisor", "Received shutdown from control service");
		self.stop.store(true, ::std::sync::atomic::Ordering::Relaxed);
		true
	}
}

pub fn main() {
	boot::setup_cli_logger("stratum");

	let service_config: ServiceConfiguration = boot::payload()
		.unwrap_or_else(|e| {
			println!("Fatal: error reading boot arguments ({:?})", e);
			std::process::exit(1)
		});

	let job_dispatcher = dependency!(
		RemoteJobDispatcher,
		&service_urls::with_base(&service_config.io_path, JOB_DISPATCHER_SOCKET_NAME)
	);

	let _ = boot::main_thread();
	let service_stop = Arc::new(AtomicBool::new(false));

	let server =
		StratumServer::start(
			&SocketAddr::new(
				IpAddr::from_str(&service_config.listen_addr)
					.unwrap_or_else(|e|
						println!("Fatal: invalid listen address: '{}' ({:?})", &service_config.listen_addr, e);
						std::process::exit(1)
					),
				service_config.port,
			),
			job_dispatcher.service().clone(),
			service_config.secret
		).unwrap_or_else(
			|e| {
				println!("Fatal: cannot start stratum server({:?})", e);
				std::process::exit(1)
			}
		);

	boot::host_service(
		&service_urls::with_base(&service_config.io_path, STRATUM_SOCKET_NAME),
		service_stop.clone(),
		server.clone() as Arc<PushWorkHandler>
	);

	let hypervisor = boot::register(
		&service_urls::with_base(&service_config.io_path, HYPERVISOR_IPC_URL),
		&service_urls::with_base(&service_config.io_path, service_urls::STRATUM_CONTROL),
		MODULE_ID
	);

	let timer_svc = server.clone();
	let timer_stop = service_stop.clone();
	thread::spawn(move || {
		while !timer_stop.load(Ordering::SeqCst) {
			thread::park_timeout(::std::time::Duration::from_millis(2000));
			// It almost always not doing anything, only greets new peers with a job
			timer_svc.maintain();
		}
	});

	let control_service = Arc::new(StratumControlService::default());
	let as_control = control_service.clone() as Arc<ControlService>;
	let mut worker = nanoipc::Worker::<ControlService>::new(&as_control);
	worker.add_reqrep(
		&service_urls::with_base(&service_config.io_path, service_urls::STRATUM_CONTROL)
	).unwrap();

	while !control_service.stop.load(Ordering::SeqCst) {
		worker.poll();
	}
	service_stop.store(true, Ordering::SeqCst);

	hypervisor.module_shutdown(MODULE_ID);
	trace!(target: "hypervisor", "Stratum process terminated gracefully");
}
