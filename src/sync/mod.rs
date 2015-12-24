use std::sync::{Arc};
use eth::{BlockChainClient};
use util::network::{ProtocolHandler, NetworkService, HandlerIo, TimerToken, PeerId};
use sync::chain::{ChainSync, SyncIo};

mod chain;


pub fn new(service: &mut NetworkService, eth_cleint: Arc<BlockChainClient>) {

}

struct EthSync {
	idle: bool,
	chain: Arc<BlockChainClient+Send+Sized>,
	sync: ChainSync
}

impl ProtocolHandler for EthSync {
	fn initialize(&mut self, io: &mut HandlerIo) {
		io.register_timer(1000);
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

	fn timeout(&mut self, io: &mut HandlerIo, timer: TimerToken) {
	}
}


