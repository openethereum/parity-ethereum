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
use std::net::SocketAddr;
use ethcore::client::Client;
use ethsync::EthSync;
use ethminer::Miner;
use util::RotatingLogger;
use util::keys::store::{AccountService};
use die::*;

#[cfg(feature = "rpc")]
pub use ethcore_rpc::Server as RpcServer;
#[cfg(feature = "rpc")]
use ethcore_rpc::{RpcServerError, RpcServer as Server};

#[cfg(not(feature = "rpc"))]
pub struct RpcServer;

#[cfg(not(feature = "rpc"))]
pub fn setup_rpc_server(
	_client: Arc<Client>,
	_sync: Arc<EthSync>,
	_secret_store: Arc<AccountService>,
	_miner: Arc<Miner>,
	_url: &SocketAddr,
	_cors_domain: Option<String>,
	_apis: Vec<&str>,
	_logger: Arc<RotatingLogger>,
) -> ! {
	die!("Your Parity version has been compiled without JSON-RPC support.")
}

#[cfg(feature = "rpc")]
pub fn setup_rpc_server(
	client: Arc<Client>,
	sync: Arc<EthSync>,
	secret_store: Arc<AccountService>,
	miner: Arc<Miner>,
	url: &SocketAddr,
	cors_domain: Option<String>,
	apis: Vec<&str>,
	logger: Arc<RotatingLogger>,
) -> RpcServer {
	use ethcore_rpc::v1::*;

	let server = Server::new();
	for api in apis.into_iter() {
		match api {
			"web3" => server.add_delegate(Web3Client::new().to_delegate()),
			"net" => server.add_delegate(NetClient::new(&sync).to_delegate()),
			"eth" => {
				server.add_delegate(EthClient::new(&client, &sync, &secret_store, &miner).to_delegate());
				server.add_delegate(EthFilterClient::new(&client, &miner).to_delegate());
			},
			"personal" => server.add_delegate(PersonalClient::new(&secret_store).to_delegate()),
			"ethcore" => server.add_delegate(EthcoreClient::new(&miner, logger.clone()).to_delegate()),
			_ => {
				die!("{}: Invalid API name to be enabled.", api);
			},
		}
	}
	let start_result = server.start_http(url, cors_domain);
	match start_result {
		Err(RpcServerError::IoError(err)) => die_with_io_error(err),
		Err(e) => die!("{:?}", e),
		Ok(server) => server,
	}
}

