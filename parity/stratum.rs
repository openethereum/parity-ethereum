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
use ethcore_stratum::{Stratum as StratumServer, PushWorkHandler, RemoteJobDispatcher, ServiceConfiguration};
use std::thread;
use modules::service_urls;
use boot;
use hypervisor::service::IpcModuleId;
use std::net::SocketAddr;
use std::str::FromStr;

const STRATUM_MODULE_ID: IpcModuleId = 8000;

pub fn main() {
	boot::setup_cli_logger("stratum");

	let service_config: ServiceConfiguration = boot::payload()
		.unwrap_or_else(|e| panic!("Fatal: error reading boot arguments ({:?})", e));

	let job_dispatcher = dependency!(RemoteJobDispatcher, service_urls::MINING_JOB_DISPATCHER);

	let stop = boot::main_thread();
	let server =
		StratumServer::start(
			&SocketAddr::from_str(&service_config.listen_addr)
				.unwrap_or_else(|e| panic!("Fatal: invalid listen address ({:?})", e)),
			job_dispatcher.service().clone(),
			service_config.secret
		).unwrap_or_else(
			|e| panic!("Fatal: cannot start stratum server({:?})", e)
		);

	boot::host_service(service_urls::STRATUM, stop.clone(), server.clone() as Arc<PushWorkHandler>);

	let _ = boot::register(STRATUM_MODULE_ID);

	while !stop.load(::std::sync::atomic::Ordering::Relaxed) {
		thread::park_timeout(std::time::Duration::from_millis(1000));
	}
}
