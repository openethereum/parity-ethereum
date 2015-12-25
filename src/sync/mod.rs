use std::sync::{Arc};
use eth::{BlockChainClient};
use util::network::{ProtocolHandler, NetworkService, HandlerIo, TimerToken, PeerId};
use sync::chain::{ChainSync, SyncIo};

mod chain;
mod range_collection;

#[cfg(test)]
mod tests;


pub fn new(_service: &mut NetworkService, eth_client: Arc<BlockChainClient+Send+Sized>) -> EthSync {
	EthSync {
		chain: eth_client,
		sync: ChainSync::new(),
	}
}

pub struct EthSync {
	chain: Arc<BlockChainClient+Send+Sized>,
	sync: ChainSync
}

pub use self::chain::SyncStatus;

impl EthSync {
	pub fn is_syncing(&self) -> bool {
		self.sync.is_syncing()
	}

	pub fn status(&self) -> SyncStatus {
		self.sync.status()
	}

	pub fn stop_network(&mut self, io: &mut HandlerIo) {
		self.sync.abort(&mut SyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()));
	}

	pub fn start_network(&mut self, io: &mut HandlerIo) {
		self.sync.restart(&mut SyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()));
	}
}

impl ProtocolHandler for EthSync {
	fn initialize(&mut self, io: &mut HandlerIo) {
		self.sync.restart(&mut SyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()));
		io.register_timer(1000).unwrap();
	}

	fn read(&mut self, io: &mut HandlerIo, peer: &PeerId, packet_id: u8, data: &[u8]) {
		self.sync.on_packet(&mut SyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()), peer, packet_id, data);
	}

	fn connected(&mut self, io: &mut HandlerIo, peer: &PeerId) {
		self.sync.on_peer_connected(&mut SyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()), peer);
	}

	fn disconnected(&mut self, io: &mut HandlerIo, peer: &PeerId) {
		self.sync.on_peer_aborting(&mut SyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()), peer);
	}

	fn timeout(&mut self, io: &mut HandlerIo, _timer: TimerToken) {
		self.sync.maintain_sync(&mut SyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()));
	}
}


