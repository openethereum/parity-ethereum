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

//! Parity micro-service helpers

use nanoipc;
use ipc;
use std;
use std::sync::Arc;
use hypervisor::{HypervisorServiceClient, HYPERVISOR_IPC_URL};
use hypervisor::service::IpcModuleId;
use ctrlc::CtrlC;
use std::sync::atomic::{AtomicBool, Ordering};
use nanoipc::{IpcInterface, GuardedSocket, NanoSocket};
use ipc::WithSocket;

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

		while !stop_guard.load(Ordering::Relaxed) {
			worker.poll();
		}
	});
}

pub fn payload<B: ipc::BinaryConvertable>() -> Result<B, BootError> {
	use std::io;
	use std::io::Read;

	let mut buffer = Vec::new();
	try!(
		io::stdin().read_to_end(&mut buffer)
			.map_err(|io_err| BootError::ReadArgs(io_err))
	);

	ipc::binary::deserialize::<B>(&buffer)
		.map_err(|binary_error| BootError::DecodeArgs(binary_error))
}

pub fn register(module_id: IpcModuleId) -> GuardedSocket<HypervisorServiceClient<NanoSocket>>{
	let hypervisor_client = nanoipc::init_client::<HypervisorServiceClient<_>>(HYPERVISOR_IPC_URL).unwrap();
	hypervisor_client.handshake().unwrap();
	hypervisor_client.module_ready(module_id);

	hypervisor_client
}

pub fn dependency<C: WithSocket<NanoSocket>>(url: &str)
	-> Result<GuardedSocket<C>, BootError>
{
	nanoipc::init_client::<C>(url).map_err(|socket_err| BootError::DependencyConnect(socket_err))
}

pub fn main_thread() -> Arc<AtomicBool> {
	let stop = Arc::new(AtomicBool::new(false));
	let ctrc_stop = stop.clone();
	CtrlC::set_handler(move || {
		ctrc_stop.store(true, Ordering::Relaxed);
	});
	stop
}
