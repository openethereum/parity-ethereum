// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::io;
use std::path::{Path, PathBuf};

use ansi_term::Colour::White;
use ethcore_logger::Config as LogConfig;
use rpc;
use rpc_apis;
use parity_rpc;
use path::restrict_permissions_owner;

pub const CODES_FILENAME: &'static str = "authcodes";

pub struct NewToken {
	pub token: String,
	pub message: String,
}

pub fn new_service(ws_conf: &rpc::WsConfiguration, logger_config: &LogConfig) -> rpc_apis::SignerService {
	let logger_config_color = logger_config.color;
	let signer_path = ws_conf.signer_path.clone();
	let signer_enabled = ws_conf.support_token_api;

	rpc_apis::SignerService::new(move || {
		generate_new_token(&signer_path, logger_config_color).map_err(|e| format!("{:?}", e))
	}, signer_enabled)
}

pub fn codes_path(path: &Path) -> PathBuf {
	let mut p = path.to_owned();
	p.push(CODES_FILENAME);
	let _ = restrict_permissions_owner(&p, true, false);
	p
}

pub fn execute(ws_conf: rpc::WsConfiguration, logger_config: LogConfig) -> Result<String, String> {
	Ok(generate_token_and_url(&ws_conf, &logger_config)?.message)
}

pub fn generate_token_and_url(ws_conf: &rpc::WsConfiguration, logger_config: &LogConfig) -> Result<NewToken, String> {
	let code = generate_new_token(&ws_conf.signer_path, logger_config.color).map_err(|err| format!("Error generating token: {:?}", err))?;
	let colored = |s: String| match logger_config.color {
		true => format!("{}", White.bold().paint(s)),
		false => s,
	};

	Ok(NewToken {
		token: code.clone(),
		message: format!(
			r#"
Generated token:
{}
"#,
			colored(code)
		),
	})
}

fn generate_new_token(path: &Path, logger_config_color: bool) -> io::Result<String> {
	let path = codes_path(path);
	let mut codes = parity_rpc::AuthCodes::from_file(&path)?;
	codes.clear_garbage();
	let code = codes.generate_new()?;
	codes.to_file(&path)?;
	trace!("New key code created: {}", match logger_config_color {
		true => format!("{}", White.bold().paint(&code[..])),
		false => format!("{}", &code[..])
	});
	Ok(code)
}
