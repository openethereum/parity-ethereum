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

use std::io;
use std::sync::Arc;
use std::path::PathBuf;
use ansi_term::Colour;
use io::{ForwardPanic, PanicHandler};
use util::path::restrict_permissions_owner;
use rpc_apis;
use ethcore_signer as signer;
use helpers::replace_home;
pub use ethcore_signer::Server as SignerServer;

const CODES_FILENAME: &'static str = "authcodes";

#[derive(Debug, PartialEq)]
pub struct Configuration {
	pub enabled: bool,
	pub port: u16,
	pub interface: String,
	pub signer_path: String,
	pub skip_origin_validation: bool,
}

impl Default for Configuration {
	fn default() -> Self {
		Configuration {
			enabled: true,
			port: 8180,
			interface: "127.0.0.1".into(),
			signer_path: replace_home("$HOME/.parity/signer"),
			skip_origin_validation: false,
		}
	}
}

pub struct Dependencies {
	pub panic_handler: Arc<PanicHandler>,
	pub apis: Arc<rpc_apis::Dependencies>,
}

pub fn start(conf: Configuration, deps: Dependencies) -> Result<Option<SignerServer>, String> {
	if !conf.enabled {
		Ok(None)
	} else {
		Ok(Some(try!(do_start(conf, deps))))
	}
}

fn codes_path(path: String) -> PathBuf {
	let mut p = PathBuf::from(path);
	p.push(CODES_FILENAME);
	let _ = restrict_permissions_owner(&p);
	p
}

pub fn new_token(path: String) -> Result<String, String> {
	generate_new_token(path)
		.map(|code| format!("This key code will authorise your System Signer UI: {}", Colour::White.bold().paint(code)))
		.map_err(|err| format!("Error generating token: {:?}", err))
}

pub fn generate_new_token(path: String) -> io::Result<String> {
	let path = codes_path(path);
	let mut codes = try!(signer::AuthCodes::from_file(&path));
	let code = try!(codes.generate_new());
	try!(codes.to_file(&path));
	trace!("New key code created: {}", Colour::White.bold().paint(&code[..]));
	Ok(code)
}

fn do_start(conf: Configuration, deps: Dependencies) -> Result<SignerServer, String> {
	let addr = try!(format!("{}:{}", conf.interface, conf.port)
		.parse()
		.map_err(|_| format!("Invalid port specified: {}", conf.port)));

	let start_result = {
		let server = signer::ServerBuilder::new(
			deps.apis.signer_service.queue(),
			codes_path(conf.signer_path),
		);
		if conf.skip_origin_validation {
			warn!("{}", Colour::Red.bold().paint("*** INSECURE *** Running Trusted Signer with no origin validation."));
			info!("If you do not intend this, exit now.");
		}
		let server = server.skip_origin_validation(conf.skip_origin_validation);
		let server = rpc_apis::setup_rpc(server, deps.apis, rpc_apis::ApiSet::SafeContext);
		server.start(addr)
	};

	match start_result {
		Err(signer::ServerError::IoError(err)) => Err(format!("Trusted Signer Error: {}", err)),
		Err(e) => Err(format!("Trusted Signer Error: {:?}", e)),
		Ok(server) => {
			deps.panic_handler.forward_from(&server);
			Ok(server)
		},
	}
}


