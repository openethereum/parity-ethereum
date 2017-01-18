// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

use ethcore::client::{BlockId, EachBlockWith};

mod test_net;

#[test]
fn basic_sync() {
	::env_logger::init().ok();

	let mut net = TestNet::light(1, 2);
	net.peer(1).chain().add_blocks(5000, EachBlockWith::Nothing);
	net.peer(2).chain().add_blocks(6000, EachBlockWith::Nothing);

	net.sync();

	assert!(net.peer(0).light_chain().get_header(BlockId::Number(12000)).is_some())
}
