// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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
use parity_ipfs_api::{self, AccessControlAllowOrigin, Host, Listening};
use parity_ipfs_api::error::ServerError;
use ethcore::client::BlockChainClient;

#[derive(Debug, PartialEq, Clone)]
pub struct Configuration {
	pub enabled: bool,
	pub port: u16,
	pub interface: String,
	pub cors: Option<Vec<String>>,
	pub hosts: Option<Vec<String>>,
}

impl Default for Configuration {
	fn default() -> Self {
		Configuration {
			enabled: false,
			port: 5001,
			interface: "127.0.0.1".into(),
			cors: Some(vec![]),
			hosts: Some(vec![]),
		}
	}
}

pub fn start_server(conf: Configuration, client: Arc<BlockChainClient>) -> Result<Option<Listening>, ServerError> {
	if !conf.enabled {
		return Ok(None);
	}

	let cors = conf.cors.map(|cors| cors.into_iter().map(AccessControlAllowOrigin::from).collect());
	let hosts = conf.hosts.map(|hosts| hosts.into_iter().map(Host::from).collect());

	parity_ipfs_api::start_server(
		conf.port,
		conf.interface,
		cors.into(),
		hosts.into(),
		client
	).map(Some)
}
