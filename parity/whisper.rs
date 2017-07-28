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

use std::sync::Arc;
use std::io;

use ethsync::{AttachedProtocol, ManageNetwork};
use parity_rpc::Metadata;
use parity_whisper::message::Message;
use parity_whisper::net::{self as whisper_net, Network as WhisperNetwork};
use parity_whisper::rpc::{WhisperClient, PoolHandle, FilterManager};

/// Whisper config.
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
	pub enabled: bool,
	pub target_message_pool_size: usize,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			enabled: false,
			target_message_pool_size: 10 * 1024 * 1024,
		}
	}
}

/// Standard pool handle.
pub struct NetPoolHandle {
	/// Pool handle.
	handle: Arc<WhisperNetwork<Arc<FilterManager>>>,
	/// Network manager.
	net: Arc<ManageNetwork>,
}

impl PoolHandle for NetPoolHandle {
	fn relay(&self, message: Message) -> bool {
		let mut res = false;
		let mut message = Some(message);
		self.net.with_proto_context(whisper_net::PROTOCOL_ID, &mut move |ctx| {
			if let Some(message) = message.take() {
				res = self.handle.post_message(message, ctx);
			}
		});
		res
	}

	fn pool_status(&self) -> whisper_net::PoolStatus {
		self.handle.pool_status()
	}
}

/// Factory for standard whisper RPC.
pub struct RpcFactory {
	net: Arc<WhisperNetwork<Arc<FilterManager>>>,
	manager: Arc<FilterManager>,
}

impl RpcFactory {
	pub fn make_handler(&self, net: Arc<ManageNetwork>) -> WhisperClient<NetPoolHandle, Metadata> {
		let handle = NetPoolHandle { handle: self.net.clone(), net: net };
		WhisperClient::new(handle, self.manager.clone())
	}
}

/// Sets up whisper protocol and RPC handler.
///
/// Will target the given pool size.
#[cfg(not(feature = "ipc"))]
pub fn setup(target_pool_size: usize, protos: &mut Vec<AttachedProtocol>)
	-> io::Result<Option<RpcFactory>>
{
	let manager = Arc::new(FilterManager::new()?);
	let net = Arc::new(WhisperNetwork::new(target_pool_size, manager.clone()));

	protos.push(AttachedProtocol {
		handler: net.clone() as Arc<_>,
		packet_count: whisper_net::PACKET_COUNT,
		versions: whisper_net::SUPPORTED_VERSIONS,
		protocol_id: whisper_net::PROTOCOL_ID,
	});

	// parity-only extensions to whisper.
	protos.push(AttachedProtocol {
		handler: Arc::new(whisper_net::ParityExtensions),
		packet_count: whisper_net::PACKET_COUNT,
		versions: whisper_net::SUPPORTED_VERSIONS,
		protocol_id: whisper_net::PARITY_PROTOCOL_ID,
	});

	let factory = RpcFactory { net: net, manager: manager };

	Ok(Some(factory))
}

// TODO: make it possible to attach generic protocols in IPC.
#[cfg(feature = "ipc")]
pub fn setup(_target_pool_size: usize, _protos: &mut Vec<AttachedProtocol>)
	-> io::Result<Option<RpcFactory>>
{
	Ok(None)
}
