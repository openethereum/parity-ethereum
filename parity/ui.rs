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

use ansi_term::Colour;
use dir::default_data_path;
use parity_ui_server as ui;
use helpers::replace_home;
use rpc;
use rpc_apis;
use path::restrict_permissions_owner;

pub const CODES_FILENAME: &'static str = "authcodes";

#[derive(Debug, PartialEq, Clone)]
pub struct Configuration {
	pub enabled: bool,
	pub port: u16,
	pub interface: String,
	pub signer_path: String,
	pub skip_origin_validation: bool,
}

impl From<Configuration> for rpc::HttpConfiguration {
	fn from(conf: Configuration) -> Self {
		rpc::HttpConfiguration {
			enabled: conf.enabled,
			interface: conf.interface,
			port: conf.port,
			apis: rpc_apis::ApiSet::SafeContext,
			cors: None,
			// TODO [ToDr] ?
			hosts: if conf.skip_origin_validation { None } else { Some(vec![]) },
			threads: None,
		}
	}
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

pub struct NewToken {
	pub token: String,
	pub url: String,
	pub message: String,
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
	let mut codes = ui::AuthCodes::from_file(&path)?;
	codes.clear_garbage();
	let code = codes.generate_new()?;
	codes.to_file(&path)?;
	trace!("New key code created: {}", Colour::White.bold().paint(&code[..]));
	Ok(code)
}
