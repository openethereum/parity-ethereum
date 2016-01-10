use client::BlockChainClient;
use util::network::{HandlerIo, PeerId, PacketId,};
use util::error::UtilError;

pub trait SyncIo {
	fn disable_peer(&mut self, peer_id: &PeerId);
	fn respond(&mut self, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError>;
	fn send(&mut self, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError>;
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

	fn respond(&mut self, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError>{
		self.network.respond(packet_id, data)
	}

	fn send(&mut self, peer_id: PeerId, packet_id: PacketId, data: Vec<u8>) -> Result<(), UtilError>{
		self.network.send(peer_id, packet_id, data)
	}

	fn chain<'a>(&'a mut self) -> &'a mut BlockChainClient {
		self.chain
	}
}


