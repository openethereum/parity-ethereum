// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Block chunker and rebuilder tests.

use devtools::RandomTempPath;

use blockchain::generator::{ChainGenerator, ChainIterator, BlockFinalizer};
use blockchain::BlockChain;
use snapshot::{BlockChunker, BlockRebuilder};
use views::BlockView;

use util::Hashable;

fn chunk_and_restore(amount: usize) {
	let mut canon_chain = ChainGenerator::default();
	let mut finalizer = BlockFinalizer::default();
	let genesis = canon_chain.generate(&mut finalizer).unwrap();
	let genesis_hash = BlockView::new(&genesis).header_view().sha3();

	let orig_path = RandomTempPath::new();
	let new_path = RandomTempPath::new();
	let bc = BlockChain::new(Default::default(), &genesis, orig_path.as_path());

	// build the blockchain.
	for _ in 0..amount {
		let block = canon_chain.generate(&mut finalizer).unwrap();
		bc.insert_block(&block, vec![]);
	}

	// snapshot it.

	// restore it.

	// and test it.
}

#[test]
fn chunk_and_restore_10k() { chunk_and_restore(10_000) }

#[test]
fn chunk_and_restore_40k() { chunk_and_restore(40_000) }