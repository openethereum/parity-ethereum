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

use tests::helpers::TestNet;

use ethcore::client::{BlockInfo, BlockId, EachBlockWith};

mod test_net;

#[test]
fn basic_sync() {
	let mut net = TestNet::light(1, 2);
	net.peer(1).chain().add_blocks(5000, EachBlockWith::Nothing);
	net.peer(2).chain().add_blocks(6000, EachBlockWith::Nothing);

	net.sync();

	assert!(net.peer(0).light_chain().block_header(BlockId::Number(6000)).is_some());
}

#[test]
fn fork_post_cht() {
	const CHAIN_LENGTH: u64 = 50; // shouldn't be longer than ::light::cht::size();

	let mut net = TestNet::light(1, 2);

	// peer 2 is on a higher TD chain.
	net.peer(1).chain().add_blocks(CHAIN_LENGTH as usize, EachBlockWith::Nothing);
	net.peer(2).chain().add_blocks(CHAIN_LENGTH as usize + 1, EachBlockWith::Uncle);

	// get the light peer on peer 1's chain.
	for id in (0..CHAIN_LENGTH).map(|x| x + 1).map(BlockId::Number) {
		let (light_peer, full_peer) = (net.peer(0), net.peer(1));
		let light_chain = light_peer.light_chain();
		let header = full_peer.chain().block_header(id).unwrap().decode().expect("decoding failure");
		let _  = light_chain.import_header(header);
		light_chain.flush_queue();
		light_chain.import_verified();
		assert!(light_chain.block_header(id).is_some());
	}

	net.sync();

	for id in (0..CHAIN_LENGTH).map(|x| x + 1).map(BlockId::Number) {
		assert_eq!(
			net.peer(0).light_chain().block_header(id).unwrap(),
			net.peer(2).chain().block_header(id).unwrap()
		);
	}
}
