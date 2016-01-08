use std::sync::Arc;
use client::BlockChainClient;
use util::network::{ProtocolHandler, NetworkService, HandlerIo, TimerToken, PeerId, PacketId, Message, Error as NetworkError};
use sync::chain::ChainSync;

mod chain;
mod range_collection;

#[cfg(test)]
mod tests;

pub fn new(_service: &mut NetworkService, eth_client: Arc<BlockChainClient + Send + Sized>) -> EthSync {
	EthSync {
		chain: eth_client,
		sync: ChainSync::new(),
	}
}

pub trait SyncIo {
	fn disable_peer(&mut self, peer_id: &PeerId);
	fn respond(&mut self, packet_id: PacketId, data: Vec<u8>) -> Result<(), NetworkError>;
	fn send(&mut self, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), NetworkError>;
	fn chain<'s>(&'s mut self) -> &'s mut BlockChainClient;
}

pub struct NetSyncIo<'s, 'h> where 'h:'s {
	network: &'s mut HandlerIo<'h>,
	chain: &'s mut BlockChainClient
}

impl<'s, 'h> NetSyncIo<'s, 'h> {
	pub fn new(network: &'s mut HandlerIo<'h>, chain: &'s mut BlockChainClient) -> NetSyncIo<'s,'h> {
		NetSyncIo {
			network: network,
			chain: chain,
		}
	}
}

impl<'s, 'h> SyncIo for NetSyncIo<'s, 'h> {
	fn disable_peer(&mut self, peer_id: &PeerId) {
		self.network.disable_peer(*peer_id);
	}

	fn respond(&mut self, packet_id: PacketId, data: Vec<u8>) -> Result<(), NetworkError>{
		self.network.respond(packet_id, data)
	}

	fn send(&mut self, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), NetworkError>{
		self.network.send(peer_id, packet_id, data)
	}

	fn chain<'a>(&'a mut self) -> &'a mut BlockChainClient {
		self.chain
	}
}

pub struct EthSync {
	chain: Arc<BlockChainClient + Send + Sized>,
	sync: ChainSync
}

pub use self::chain::SyncStatus;

impl EthSync {
	pub fn new(chain: Arc<BlockChainClient + Send + Sized>) -> EthSync {
		EthSync {
			chain: chain,
			sync: ChainSync::new(),
		}
	}

	pub fn is_syncing(&self) -> bool {
		self.sync.is_syncing()
	}

	pub fn status(&self) -> SyncStatus {
		self.sync.status()
	}

	pub fn stop_network(&mut self, io: &mut HandlerIo) {
		self.sync.abort(&mut NetSyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()));
	}

	pub fn start_network(&mut self, io: &mut HandlerIo) {
		self.sync.restart(&mut NetSyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()));
	}
}

impl ProtocolHandler for EthSync {
	fn initialize(&mut self, io: &mut HandlerIo) {
		self.sync.restart(&mut NetSyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()));
		io.register_timer(1000).unwrap();
	}

	fn read(&mut self, io: &mut HandlerIo, peer: &PeerId, packet_id: u8, data: &[u8]) {
		self.sync.on_packet(&mut NetSyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()), peer, packet_id, data);
	}

	fn connected(&mut self, io: &mut HandlerIo, peer: &PeerId) {
		self.sync.on_peer_connected(&mut NetSyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()), peer);
	}

	fn disconnected(&mut self, io: &mut HandlerIo, peer: &PeerId) {
		self.sync.on_peer_aborting(&mut NetSyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()), peer);
	}

	fn timeout(&mut self, io: &mut HandlerIo, _timer: TimerToken) {
		self.sync.maintain_sync(&mut NetSyncIo::new(io, Arc::get_mut(&mut self.chain).unwrap()));
	}

	fn message(&mut self, _io: &mut HandlerIo, _message: &Message) {
	}
}


