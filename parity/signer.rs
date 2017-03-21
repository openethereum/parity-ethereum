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

use std::io;
use std::path::PathBuf;
use std::sync::Arc;

pub use ethcore_signer::Server as SignerServer;

use ansi_term::Colour;
use dir::default_data_path;
use ethcore_rpc::informant::RpcStats;
use ethcore_rpc;
use ethcore_signer as signer;
use helpers::replace_home;
use io::{ForwardPanic, PanicHandler};
use jsonrpc_core::reactor::{RpcHandler, Remote};
use rpc_apis;
use util::path::restrict_permissions_owner;
use util::H256;

const CODES_FILENAME: &'static str = "authcodes";

#[derive(Debug, PartialEq, Clone)]
pub struct Configuration {
	pub enabled: bool,
	pub port: u16,
	pub interface: String,
	pub signer_path: String,
	pub skip_origin_validation: bool,
}

impl Default for Configuration {
	fn default() -> Self {
		let data_dir = default_data_path();
		Configuration {
			enabled: true,
			port: 8180,
			interface: "127.0.0.1".into(),
			signer_path: replace_home(&data_dir, "$BASE/signer"),
			skip_origin_validation: false,
		}
	}
}

pub struct Dependencies {
	pub panic_handler: Arc<PanicHandler>,
	pub apis: Arc<rpc_apis::Dependencies>,
	pub remote: Remote,
	pub rpc_stats: Arc<RpcStats>,
}

pub struct NewToken {
	pub token: String,
	pub url: String,
	pub message: String,
}

#[derive(Debug, Default, Clone)]
pub struct StandardExtractor;
impl signer::MetaExtractor<ethcore_rpc::Metadata> for StandardExtractor {
	fn extract_metadata(&self, session: &H256) -> ethcore_rpc::Metadata {
		let mut metadata = ethcore_rpc::Metadata::default();
		metadata.origin = ethcore_rpc::Origin::Signer((*session).into());
		metadata
	}
}

pub fn start(conf: Configuration, deps: Dependencies) -> Result<Option<SignerServer>, String> {
	if !conf.enabled {
		Ok(None)
	} else {
		Ok(Some(do_start(conf, deps)?))
	}
}

fn codes_path(path: String) -> PathBuf {
	let mut p = PathBuf::from(path);
	p.push(CODES_FILENAME);
	let _ = restrict_permissions_owner(&p, true, false);
	p
}

pub fn execute(cmd: Configuration) -> Result<String, String> {
	Ok(generate_token_and_url(&cmd)?.message)
}

pub fn generate_token_and_url(conf: &Configuration) -> Result<NewToken, String> {
	let code = generate_new_token(conf.signer_path.clone()).map_err(|err| format!("Error generating token: {:?}", err))?;
	let auth_url = format!("http://{}:{}/#/auth?token={}", conf.interface, conf.port, code);
	// And print in to the console
	Ok(NewToken {
		token: code.clone(),
		url: auth_url.clone(),
		message: format!(
			r#"
Open: {}
to authorize your browser.
Or use the generated token:
{}"#,
			Colour::White.bold().paint(auth_url),
			code
		)
	})
}

pub fn generate_new_token(path: String) -> io::Result<String> {
	let path = codes_path(path);
	let mut codes = signer::AuthCodes::from_file(&path)?;
	codes.clear_garbage();
	let code = codes.generate_new()?;
	codes.to_file(&path)?;
	trace!("New key code created: {}", Colour::White.bold().paint(&code[..]));
	Ok(code)
}

fn do_start(conf: Configuration, deps: Dependencies) -> Result<SignerServer, String> {
	let addr = format!("{}:{}", conf.interface, conf.port)
		.parse()
		.map_err(|_| format!("Invalid port specified: {}", conf.port))?;

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
		let server = server.stats(deps.rpc_stats.clone());
		let apis = rpc_apis::setup_rpc(deps.rpc_stats, deps.apis, rpc_apis::ApiSet::SafeContext);
		let handler = RpcHandler::new(Arc::new(apis), deps.remote);
		server.start_with_extractor(addr, handler, StandardExtractor)
	};

	match start_result {
		Err(signer::ServerError::IoError(err)) => match err.kind() {
			io::ErrorKind::AddrInUse => Err(format!("Trusted UI address {} is already in use, make sure that another instance of an Ethereum client is not running or change the address using the --ui-port and --ui-interface options.", addr)),
			_ => Err(format!("Trusted Signer io error: {}", err)),
		},
		Err(e) => Err(format!("Trusted Signer Error: {:?}", e)),
		Ok(server) => {
			deps.panic_handler.forward_from(&server);
			Ok(server)
		},
	}
}


