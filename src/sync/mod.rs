/// Blockchain sync module
/// Implements ethereum protocol version 63 as specified here:
/// https://github.com/ethereum/wiki/wiki/Ethereum-Wire-Protocol
///
/// Usage example:
///
/// ```rust
/// extern crate ethcore_util as util;
/// extern crate ethcore;
/// use std::env;
/// use std::sync::Arc;
/// use util::network::NetworkService;
/// use ethcore::client::Client;
/// use ethcore::sync::EthSync;
/// use ethcore::ethereum;
///
/// fn main() {
/// 	let mut service = NetworkService::start().unwrap();
/// 	let dir = env::temp_dir();
/// 	let client = Arc::new(Client::new(ethereum::new_frontier(), &dir).unwrap());
/// 	EthSync::register(&mut service, client);
/// }
/// ```

use std::ops::*;
use std::sync::*;
use client::Client;
use util::network::{NetworkProtocolHandler, NetworkService, NetworkContext, PeerId, NetworkIoMessage};
use util::TimerToken;
use util::Bytes;
use sync::chain::ChainSync;
use sync::io::NetSyncIo;

mod chain;
mod io;
mod range_collection;

#[cfg(test)]
mod tests;

/// Message type for external events
pub enum SyncMessage {
	/// New block has been imported into the blockchain
	NewChainBlock(Bytes),
	/// A block is ready 
	BlockVerified,
}

/// TODO [arkpar] Please document me
pub type NetSyncMessage = NetworkIoMessage<SyncMessage>;

/// Ethereum network protocol handler
pub struct EthSync {
	/// Shared blockchain client. TODO: this should evetually become an IPC endpoint
	chain: Arc<RwLock<Client>>,
	/// Sync strategy
	sync: ChainSync
}

pub use self::chain::SyncStatus;

impl EthSync {
	/// Creates and register protocol with the network service
	pub fn register(service: &mut NetworkService<SyncMessage>, chain: Arc<RwLock<Client>>) {
		let sync = Box::new(EthSync {
			chain: chain,
			sync: ChainSync::new(),
		});
		service.register_protocol(sync, "eth", &[62u8, 63u8]).expect("Error registering eth protocol handler");
	}

	/// Get sync status
	pub fn status(&self) -> SyncStatus {
		self.sync.status()
	}

	/// Stop sync
	pub fn stop(&mut self, io: &mut NetworkContext<SyncMessage>) {
		self.sync.abort(&mut NetSyncIo::new(io, self.chain.write().unwrap().deref_mut()));
	}

	/// Restart sync
	pub fn restart(&mut self, io: &mut NetworkContext<SyncMessage>) {
		self.sync.restart(&mut NetSyncIo::new(io, self.chain.write().unwrap().deref_mut()));
	}
}

impl NetworkProtocolHandler<SyncMessage> for EthSync {
	fn initialize(&mut self, io: &mut NetworkContext<SyncMessage>) {
		self.sync.restart(&mut NetSyncIo::new(io, self.chain.write().unwrap().deref_mut()));
		io.register_timer(1000).unwrap();
	}

	fn read(&mut self, io: &mut NetworkContext<SyncMessage>, peer: &PeerId, packet_id: u8, data: &[u8]) {
		self.sync.on_packet(&mut NetSyncIo::new(io, self.chain.write().unwrap().deref_mut()) , *peer, packet_id, data);
	}

	fn connected(&mut self, io: &mut NetworkContext<SyncMessage>, peer: &PeerId) {
		self.sync.on_peer_connected(&mut NetSyncIo::new(io, self.chain.write().unwrap().deref_mut()), *peer);
	}

	fn disconnected(&mut self, io: &mut NetworkContext<SyncMessage>, peer: &PeerId) {
		self.sync.on_peer_aborting(&mut NetSyncIo::new(io, self.chain.write().unwrap().deref_mut()), *peer);
	}

	fn timeout(&mut self, io: &mut NetworkContext<SyncMessage>, _timer: TimerToken) {
		self.sync.maintain_sync(&mut NetSyncIo::new(io, self.chain.write().unwrap().deref_mut()));
	}

	fn message(&mut self, _io: &mut NetworkContext<SyncMessage>, _message: &SyncMessage) {
	}
}


