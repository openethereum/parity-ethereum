// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use std::sync::Arc;
use hash::keccak;
use bigint::prelude::U256;
use io::{IoHandler, IoContext, IoChannel};
use ethcore::client::{BlockChainClient, Client};
use ethcore::service::ClientIoMessage;
use ethcore::spec::Spec;
use ethcore::miner::MinerService;
use ethcore::transaction::*;
use ethcore::account_provider::AccountProvider;
use ethkey::{KeyPair, Secret};
use super::helpers::*;
use {SyncConfig, Address};

struct TestIoHandler {
	client: Arc<Client>,
}

impl IoHandler<ClientIoMessage> for TestIoHandler {
	fn message(&self, _io: &IoContext<ClientIoMessage>, net_message: &ClientIoMessage) {
		match *net_message {
			ClientIoMessage::NewMessage(ref message) => if let Err(e) = self.client.engine().handle_message(message) {
				panic!("Invalid message received: {}", e);
			},
			_ => {} // ignore other messages
		}
	}
}

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
	let ap = Arc::new(AccountProvider::transient_provider());
	ap.insert_account(s0.secret().clone(), "").unwrap();
	ap.insert_account(s1.secret().clone(), "").unwrap();

	let chain_id = Spec::new_test_round().chain_id();
	let mut net = TestNet::with_spec_and_accounts(2, SyncConfig::default(), Spec::new_test_round, Some(ap));
	let io_handler0: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler { client: net.peer(0).chain.clone() });
	let io_handler1: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler { client: net.peer(1).chain.clone() });
	// Push transaction to both clients. Only one of them gets lucky to produce a block.
	net.peer(0).chain.miner().set_engine_signer(s0.address(), "".to_owned()).unwrap();
	net.peer(1).chain.miner().set_engine_signer(s1.address(), "".to_owned()).unwrap();
	net.peer(0).chain.engine().register_client(Arc::downgrade(&net.peer(0).chain) as _);
	net.peer(1).chain.engine().register_client(Arc::downgrade(&net.peer(1).chain) as _);
	net.peer(0).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler1)));
	net.peer(1).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler0)));
	// exchange statuses
	net.sync();
	// Trigger block proposal
	net.peer(0).chain.miner().import_own_transaction(&*net.peer(0).chain, new_tx(s0.secret(), 0.into(), chain_id)).unwrap();
	net.peer(1).chain.miner().import_own_transaction(&*net.peer(1).chain, new_tx(s1.secret(), 0.into(), chain_id)).unwrap();
	// Sync a block
	net.sync();
	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 1);
	assert_eq!(net.peer(1).chain.chain_info().best_block_number, 1);

	net.peer(0).chain.miner().import_own_transaction(&*net.peer(0).chain, new_tx(s0.secret(), 1.into(), chain_id)).unwrap();
	net.peer(1).chain.miner().import_own_transaction(&*net.peer(1).chain, new_tx(s1.secret(), 1.into(), chain_id)).unwrap();
	// Move to next proposer step.
	net.peer(0).chain.engine().step();
	net.peer(1).chain.engine().step();
	net.sync();
	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 2);
	assert_eq!(net.peer(1).chain.chain_info().best_block_number, 2);

	// Fork the network with equal height.
	net.peer(0).chain.miner().import_own_transaction(&*net.peer(0).chain, new_tx(s0.secret(), 2.into(), chain_id)).unwrap();
	net.peer(1).chain.miner().import_own_transaction(&*net.peer(1).chain, new_tx(s1.secret(), 2.into(), chain_id)).unwrap();
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
	net.peer(0).chain.miner().import_own_transaction(&*net.peer(0).chain, new_tx(s0.secret(), 3.into(), chain_id)).unwrap();
	net.peer(1).chain.miner().import_own_transaction(&*net.peer(1).chain, new_tx(s1.secret(), 3.into(), chain_id)).unwrap();
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
	net.peer(1).chain.miner().import_own_transaction(&*net.peer(1).chain, new_tx(s1.secret(), 4.into(), chain_id)).unwrap();
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

#[test]
fn tendermint() {
	let s0 = KeyPair::from_secret_slice(&keccak("1")).unwrap();
	let s1 = KeyPair::from_secret_slice(&keccak("0")).unwrap();
	let ap = Arc::new(AccountProvider::transient_provider());
	ap.insert_account(s0.secret().clone(), "").unwrap();
	ap.insert_account(s1.secret().clone(), "").unwrap();

	let chain_id = Spec::new_test_tendermint().chain_id();
	let mut net = TestNet::with_spec_and_accounts(2, SyncConfig::default(), Spec::new_test_tendermint, Some(ap));
	let io_handler0: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler { client: net.peer(0).chain.clone() });
	let io_handler1: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler { client: net.peer(1).chain.clone() });
	// Push transaction to both clients. Only one of them issues a proposal.
	net.peer(0).chain.miner().set_engine_signer(s0.address(), "".to_owned()).unwrap();
	trace!(target: "poa", "Peer 0 is {}.", s0.address());
	net.peer(1).chain.miner().set_engine_signer(s1.address(), "".to_owned()).unwrap();
	trace!(target: "poa", "Peer 1 is {}.", s1.address());
	net.peer(0).chain.engine().register_client(Arc::downgrade(&net.peer(0).chain) as _);
	net.peer(1).chain.engine().register_client(Arc::downgrade(&net.peer(1).chain) as _);
	net.peer(0).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler0)));
	net.peer(1).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler1)));
	// Exhange statuses
	net.sync();
	// Propose
	net.peer(0).chain.miner().import_own_transaction(&*net.peer(0).chain, new_tx(s0.secret(), 0.into(), chain_id)).unwrap();
	net.sync();
	// Propose timeout, synchronous for now
	net.peer(0).chain.engine().step();
	net.peer(1).chain.engine().step();
	// Prevote, precommit and commit
	net.sync();

	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 1);
	assert_eq!(net.peer(1).chain.chain_info().best_block_number, 1);

	net.peer(1).chain.miner().import_own_transaction(&*net.peer(1).chain, new_tx(s1.secret(), 0.into(), chain_id)).unwrap();
	// Commit timeout
	net.peer(0).chain.engine().step();
	net.peer(1).chain.engine().step();
	// Propose
	net.sync();
	// Propose timeout
	net.peer(0).chain.engine().step();
	net.peer(1).chain.engine().step();
	// Prevote, precommit and commit
	net.sync();
	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 2);
	assert_eq!(net.peer(1).chain.chain_info().best_block_number, 2);

	net.peer(0).chain.miner().import_own_transaction(&*net.peer(0).chain, new_tx(s0.secret(), 1.into(), chain_id)).unwrap();
	net.peer(1).chain.miner().import_own_transaction(&*net.peer(1).chain, new_tx(s1.secret(), 1.into(), chain_id)).unwrap();
	// Peers get disconnected.
	// Commit
	net.peer(0).chain.engine().step();
	net.peer(1).chain.engine().step();
	// Propose
	net.peer(0).chain.engine().step();
	net.peer(1).chain.engine().step();
	net.peer(0).chain.miner().import_own_transaction(&*net.peer(0).chain, new_tx(s0.secret(), 2.into(), chain_id)).unwrap();
	net.peer(1).chain.miner().import_own_transaction(&*net.peer(1).chain, new_tx(s1.secret(), 2.into(), chain_id)).unwrap();
	// Send different prevotes
	net.sync();
	// Prevote timeout
	net.peer(0).chain.engine().step();
	net.peer(1).chain.engine().step();
	// Precommit and commit
	net.sync();
	// Propose timeout
	net.peer(0).chain.engine().step();
	net.peer(1).chain.engine().step();
	net.sync();
	let ci0 = net.peer(0).chain.chain_info();
	let ci1 = net.peer(1).chain.chain_info();
	assert_eq!(ci0.best_block_number, 3);
	assert_eq!(ci1.best_block_number, 3);
	assert_eq!(ci0.best_block_hash, ci1.best_block_hash);
}
