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
use sync::chain::ChainSync;
use util::{Bytes, TimerToken};
use sync::io::NetSyncIo;

mod chain;
mod io;
mod range_collection;

#[cfg(test)]
mod tests;

const SYNC_TIMER: usize = 0;

/// Message type for external events
#[derive(Clone)]
pub enum SyncMessage {
	/// New block has been imported into the blockchain
	NewChainBlock(Bytes), //TODO: use Cow
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
	sync: RwLock<ChainSync>
}

pub use self::chain::SyncStatus;

impl EthSync {
	/// Creates and register protocol with the network service
	pub fn register(service: &mut NetworkService<SyncMessage>, chain: Arc<RwLock<Client>>) {
		let sync = Arc::new(EthSync {
			chain: chain,
			sync: RwLock::new(ChainSync::new()),
		});
		service.register_protocol(sync.clone(), "eth", &[62u8, 63u8]).expect("Error registering eth protocol handler");
	}

	/// Get sync status
	pub fn status(&self) -> SyncStatus {
		self.sync.read().unwrap().status()
	}

	/// Stop sync
	pub fn stop(&mut self, io: &mut NetworkContext<SyncMessage>) {
		self.sync.write().unwrap().abort(&mut NetSyncIo::new(io, self.chain.write().unwrap().deref_mut()));
	}

	/// Restart sync
	pub fn restart(&mut self, io: &mut NetworkContext<SyncMessage>) {
		self.sync.write().unwrap().restart(&mut NetSyncIo::new(io, self.chain.write().unwrap().deref_mut()));
	}
}

impl NetworkProtocolHandler<SyncMessage> for EthSync {
	fn initialize(&self, io: &NetworkContext<SyncMessage>) {
		io.register_timer(SYNC_TIMER, 1000).unwrap();
	}

	fn read(&self, io: &NetworkContext<SyncMessage>, peer: &PeerId, packet_id: u8, data: &[u8]) {
		self.sync.write().unwrap().on_packet(&mut NetSyncIo::new(io, self.chain.write().unwrap().deref_mut()) , *peer, packet_id, data);
	}

	fn connected(&self, io: &NetworkContext<SyncMessage>, peer: &PeerId) {
		self.sync.write().unwrap().on_peer_connected(&mut NetSyncIo::new(io, self.chain.write().unwrap().deref_mut()), *peer);
	}

	fn disconnected(&self, io: &NetworkContext<SyncMessage>, peer: &PeerId) {
		self.sync.write().unwrap().on_peer_aborting(&mut NetSyncIo::new(io, self.chain.write().unwrap().deref_mut()), *peer);
	}

	fn timeout(&self, io: &NetworkContext<SyncMessage>, timer: TimerToken) {
		if timer == SYNC_TIMER {
			self.sync.write().unwrap().maintain_sync(&mut NetSyncIo::new(io, self.chain.write().unwrap().deref_mut()));
		}
	}
}


