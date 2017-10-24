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

//! Parity micro-service helpers

use nanoipc;
use ipc;
use std;
use std::sync::Arc;
use hypervisor::HypervisorServiceClient;
use hypervisor::service::IpcModuleId;
use ctrlc::CtrlC;
use std::sync::atomic::{AtomicBool, Ordering};
use nanoipc::{IpcInterface, GuardedSocket, NanoSocket};
use ipc::WithSocket;
use ethcore_logger::{Config as LogConfig, setup_log};
use docopt::Docopt;

#[derive(Debug)]
pub enum BootError {
	ReadArgs(std::io::Error),
	DecodeArgs(ipc::binary::BinaryError),
	DependencyConnect(nanoipc::SocketError),
}

pub fn host_service<T: ?Sized + Send + Sync + 'static>(addr: &str, stop_guard: Arc<AtomicBool>, service: Arc<T>) where T: IpcInterface {
	let socket_url = addr.to_owned();
	std::thread::spawn(move || {
		let mut worker = nanoipc::Worker::<T>::new(&service);
		worker.add_reqrep(&socket_url).unwrap();

		while !stop_guard.load(Ordering::SeqCst) {
			worker.poll();
		}
	});
}

pub fn payload<B: ipc::BinaryConvertable>() -> Result<B, BootError> {
	use std::io;
	use std::io::Read;

	let mut buffer = Vec::new();
	io::stdin().read_to_end(&mut buffer).map_err(BootError::ReadArgs)?;

	ipc::binary::deserialize::<B>(&buffer).map_err(BootError::DecodeArgs)
}

pub fn register(hv_url: &str, control_url: &str, module_id: IpcModuleId) -> GuardedSocket<HypervisorServiceClient<NanoSocket>>{
	let hypervisor_client = nanoipc::fast_client::<HypervisorServiceClient<_>>(hv_url).unwrap();
	hypervisor_client.handshake().unwrap();
	hypervisor_client.module_ready(module_id, control_url.to_owned());

	hypervisor_client
}

pub fn dependency<C: WithSocket<NanoSocket>>(url: &str)
	-> Result<GuardedSocket<C>, BootError>
{
	nanoipc::generic_client::<C>(url).map_err(BootError::DependencyConnect)
}

pub fn main_thread() -> Arc<AtomicBool> {
	let stop = Arc::new(AtomicBool::new(false));
	let ctrc_stop = stop.clone();
	CtrlC::set_handler(move || {
		ctrc_stop.store(true, Ordering::Relaxed);
	});
	stop
}

pub fn setup_cli_logger(svc_name: &str) {
	let usage = format!("
Ethcore {} service
Usage:
  parity {} [options]

 Options:
  -l --logging LOGGING     Specify the logging level. Must conform to the same
                           format as RUST_LOG.
  --log-file FILENAME      Specify a filename into which logging should be
                           directed.
  --no-color               Don't use terminal color codes in output.
", svc_name, svc_name);

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

	let args: Args = Docopt::new(usage)
		.and_then(|d| d.decode())
		.unwrap_or_else(|e| e.exit());
	setup_log(&args.log_settings()).expect("Log initialization failure");
}
