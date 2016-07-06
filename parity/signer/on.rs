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

extern crate ethcore_signer;
extern crate ansi_term;

use self::ansi_term::Colour::White;
use std::io;
use std::path::PathBuf;
use util::panics::ForwardPanic;
use util::path::restrict_permissions_owner;
use die::*;
use rpc_apis;
use super::{Configuration, Dependencies};
use self::ethcore_signer as signer;

pub use self::ethcore_signer::Server as SignerServer;

const CODES_FILENAME: &'static str = "authcodes";

pub fn start(conf: Configuration, deps: Dependencies) -> Option<SignerServer> {
	if !conf.enabled {
		None
	} else {
		Some(do_start(conf, deps))
	}
}

fn codes_path(path: String) -> PathBuf {
	let mut p = PathBuf::from(path);
	p.push(CODES_FILENAME);
	let _ = restrict_permissions_owner(&p);
	p
}

pub fn new_token(path: String) -> io::Result<()> {
	let path = codes_path(path);
	let mut codes = try!(signer::AuthCodes::from_file(&path));
	let code = try!(codes.generate_new());
	try!(codes.to_file(&path));
	println!("This key code will authorise your System Signer UI: {}", White.bold().paint(code));
	Ok(())
}

fn do_start(conf: Configuration, deps: Dependencies) -> SignerServer {
	let addr = format!("127.0.0.1:{}", conf.port).parse().unwrap_or_else(|_| {
		die!("Invalid port specified: {}", conf.port)
	});

	let start_result = {
		let server = signer::ServerBuilder::new(
			deps.apis.signer_queue.clone(),
			codes_path(conf.signer_path),
		);
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


