// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use hash::keccak;
use ethereum_types::{U256, Address};
use io::{IoHandler, IoChannel};
use ethcore::client::{ChainInfo, ClientIoMessage};
use ethcore::engines;
use ethcore::spec::Spec;
use ethcore::miner::{self, MinerService};
use ethkey::{KeyPair, Secret};
use types::transaction::{Action, PendingTransaction, Transaction};
use super::helpers::*;
use SyncConfig;

fn new_tx(secret: &Secret, nonce: U256, chain_id: u64) -> PendingTransaction {
	let signed = Transaction {
		nonce: nonce.into(),
		gas_price: 0.into(),
		gas: 21000.into(),
		action: Action::Call(Address::default()),
		value: 0.into(),
		data: Vec::new(),
	}.sign(secret, Some(chain_id));
	PendingTransaction::new(signed, None)
}

#[test]
fn authority_round() {
	let s0 = KeyPair::from_secret_slice(&keccak("1")).unwrap();
	let s1 = KeyPair::from_secret_slice(&keccak("0")).unwrap();

	let chain_id = Spec::new_test_round().chain_id();
	let mut net = TestNet::with_spec(2, SyncConfig::default(), Spec::new_test_round);
	let io_handler0: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler::new(net.peer(0).chain.clone()));
	let io_handler1: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler::new(net.peer(1).chain.clone()));
	// Push transaction to both clients. Only one of them gets lucky to produce a block.
	net.peer(0).miner.set_author(miner::Author::Sealer(engines::signer::from_keypair(s0.clone())));
	net.peer(1).miner.set_author(miner::Author::Sealer(engines::signer::from_keypair(s1.clone())));
	net.peer(0).chain.engine().register_client(Arc::downgrade(&net.peer(0).chain) as _);
	net.peer(1).chain.engine().register_client(Arc::downgrade(&net.peer(1).chain) as _);
	net.peer(0).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler1)));
	net.peer(1).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler0)));
	// exchange statuses
	net.sync();
	// Trigger block proposal
	net.peer(0).miner.import_own_transaction(&*net.peer(0).chain, new_tx(s0.secret(), 0.into(), chain_id)).unwrap();
	net.peer(1).miner.import_own_transaction(&*net.peer(1).chain, new_tx(s1.secret(), 0.into(), chain_id)).unwrap();
	// Sync a block
	net.sync();
	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 1);
	assert_eq!(net.peer(1).chain.chain_info().best_block_number, 1);

	net.peer(0).miner.import_own_transaction(&*net.peer(0).chain, new_tx(s0.secret(), 1.into(), chain_id)).unwrap();
	net.peer(1).miner.import_own_transaction(&*net.peer(1).chain, new_tx(s1.secret(), 1.into(), chain_id)).unwrap();
	// Move to next proposer step.
	net.peer(0).chain.engine().step();
	net.peer(1).chain.engine().step();
	net.sync();
	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 2);
	assert_eq!(net.peer(1).chain.chain_info().best_block_number, 2);

	// Fork the network with equal height.
	net.peer(0).miner.import_own_transaction(&*net.peer(0).chain, new_tx(s0.secret(), 2.into(), chain_id)).unwrap();
	net.peer(1).miner.import_own_transaction(&*net.peer(1).chain, new_tx(s1.secret(), 2.into(), chain_id)).unwrap();
	// Let both nodes build one block.
	net.peer(0).chain.engine().step();
	let early_hash = net.peer(0).chain.chain_info().best_block_hash;
	net.peer(1).chain.engine().step();
	net.peer(0).chain.engine().step();
	net.peer(1).chain.engine().step();
	let ci0 = net.peer(0).chain.chain_info();
	let ci1 = net.peer(1).chain.chain_info();
	assert_eq!(ci0.best_block_number, 3);
	assert_eq!(ci1.best_block_number, 3);
	assert!(ci0.best_block_hash != ci1.best_block_hash);
	// Reorg to the chain with earlier view.
	net.sync();
	let ci0 = net.peer(0).chain.chain_info();
	let ci1 = net.peer(1).chain.chain_info();
	assert_eq!(ci0.best_block_number, 3);
	assert_eq!(ci1.best_block_number, 3);
	assert_eq!(ci0.best_block_hash, ci1.best_block_hash);
	assert_eq!(ci1.best_block_hash, early_hash);

	// Selfish miner
	net.peer(0).miner.import_own_transaction(&*net.peer(0).chain, new_tx(s0.secret(), 3.into(), chain_id)).unwrap();
	net.peer(1).miner.import_own_transaction(&*net.peer(1).chain, new_tx(s1.secret(), 3.into(), chain_id)).unwrap();
	// Node 0 is an earlier primary.
	net.peer(0).chain.engine().step();
	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 4);
	net.peer(0).chain.engine().step();
	net.peer(0).chain.engine().step();
	net.peer(0).chain.engine().step();
	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 4);
	// Node 1 makes 2 blocks, but is a later primary on the first one.
	net.peer(1).chain.engine().step();
	net.peer(1).chain.engine().step();
	net.peer(1).miner.import_own_transaction(&*net.peer(1).chain, new_tx(s1.secret(), 4.into(), chain_id)).unwrap();
	net.peer(1).chain.engine().step();
	net.peer(1).chain.engine().step();
	assert_eq!(net.peer(1).chain.chain_info().best_block_number, 5);
	// Reorg to the longest chain one not ealier view one.
	net.sync();
	let ci0 = net.peer(0).chain.chain_info();
	let ci1 = net.peer(1).chain.chain_info();
	assert_eq!(ci0.best_block_number, 5);
	assert_eq!(ci1.best_block_number, 5);
	assert_eq!(ci0.best_block_hash, ci1.best_block_hash);
}
