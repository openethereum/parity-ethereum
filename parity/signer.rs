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
use util::panics::{PanicHandler, ForwardPanic};
use die::*;
use rpc_apis;

#[cfg(feature = "ethcore-signer")]
use ethcore_signer as signer;
#[cfg(feature = "ethcore-signer")]
pub use ethcore_signer::Server as SignerServer;
#[cfg(not(feature = "ethcore-signer"))]
pub struct SignerServer;

pub struct Configuration {
	pub enabled: bool,
	pub port: u16,
}

pub struct Dependencies {
	pub panic_handler: Arc<PanicHandler>,
	pub apis: Arc<rpc_apis::Dependencies>,
}

pub fn start(conf: Configuration, deps: Dependencies) -> Option<SignerServer> {
	if !conf.enabled {
		None
	} else {
		Some(do_start(conf, deps))
	}
}

#[cfg(feature = "ethcore-signer")]
fn do_start(conf: Configuration, deps: Dependencies) -> SignerServer {
	let addr = format!("127.0.0.1:{}", conf.port).parse().unwrap_or_else(|_| {
		die!("Invalid port specified: {}", conf.port)
	});

	let start_result = {
		let server = signer::ServerBuilder::new(daps.apis.signer_queue.clone());
		let server = rpc_apis::setup_rpc(server, deps.apis, rpc_apis::ApiSet::SafeContext);
		server.start(addr)
	};

	match start_result {
		Err(signer::ServerError::IoError(err)) => die_with_io_error("Trusted Signer", err),
		Err(e) => die!("Trusted Signer: {:?}", e),
		Ok(server) => {
			deps.panic_handler.forward_from(&server);
			server
		},
	}
}

#[cfg(not(feature = "ethcore-signer"))]
fn do_start(conf: Configuration) -> ! {
	die!("Your Parity version has been compiled without Trusted Signer support.")
}


