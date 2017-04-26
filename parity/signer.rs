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
use std::path::{Path, PathBuf};

use ansi_term::Colour;
use rpc;
use rpc_apis;
use parity_rpc;
use path::restrict_permissions_owner;


pub const CODES_FILENAME: &'static str = "authcodes";

pub struct NewToken {
	pub token: String,
	pub url: String,
	pub message: String,
}

pub fn new_service(ws_conf: &rpc::WsConfiguration, ui_conf: &rpc::UiConfiguration) -> rpc_apis::SignerService {
	let signer_path = ws_conf.signer_path.clone();
	let signer_enabled = ui_conf.enabled;

	rpc_apis::SignerService::new(move || {
		generate_new_token(&signer_path).map_err(|e| format!("{:?}", e))
	}, signer_enabled)
}

pub fn codes_path(path: &Path) -> PathBuf {
	let mut p = path.to_owned();
	p.push(CODES_FILENAME);
	let _ = restrict_permissions_owner(&p, true, false);
	p
}

pub fn execute(ws_conf: rpc::WsConfiguration, ui_conf: rpc::UiConfiguration) -> Result<String, String> {
	Ok(generate_token_and_url(&ws_conf, &ui_conf)?.message)
}

pub fn generate_token_and_url(ws_conf: &rpc::WsConfiguration, ui_conf: &rpc::UiConfiguration) -> Result<NewToken, String> {
	let code = generate_new_token(&ws_conf.signer_path).map_err(|err| format!("Error generating token: {:?}", err))?;
	let auth_url = format!("http://{}:{}/#/auth?token={}", ui_conf.interface, ui_conf.port, code);
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

fn generate_new_token(path: &Path) -> io::Result<String> {
	let path = codes_path(path);
	let mut codes = parity_rpc::AuthCodes::from_file(&path)?;
	codes.clear_garbage();
	let code = codes.generate_new()?;
	codes.to_file(&path)?;
	trace!("New key code created: {}", Colour::White.bold().paint(&code[..]));
	Ok(code)
}
