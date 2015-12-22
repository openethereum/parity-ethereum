use std::sync::{Arc};
use eth::{BlockChainClient};
use util::network::{Error as NetworkError, ProtocolHandler, NetworkService, HandlerIo, TimerToken, PeerId};

mod chain;


pub fn new(service: &mut NetworkService, eth_cleint: Arc<BlockChainClient>) {

}

struct EthSync {
	idle: bool,
	chain: Arc<BlockChainClient+Send>
}

impl ProtocolHandler for EthSync {
	fn initialize(&mut self, io: &mut HandlerIo) {
		io.register_timer(1000);
	}

	fn read(&mut self, io: &mut HandlerIo, peer: &PeerId, packet_id: u8, data: &[u8]) {
	}

	fn connected(&mut self, io: &mut HandlerIo, peer: &PeerId) {
	}

	fn disconnected(&mut self, io: &mut HandlerIo, peer: &PeerId) {
	}

	fn timeout(&mut self, io: &mut HandlerIo, timer: TimerToken) {
	}
}


