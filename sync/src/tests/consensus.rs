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

use util::*;
use io::{IoHandler, IoContext, IoChannel};
use ethcore::client::{BlockChainClient, Client, MiningBlockChainClient};
use ethcore::service::{ClientIoMessage};
use ethcore::spec::Spec;
use ethcore::miner::MinerService;
use ethcore::transaction::*;
use ethcore::account_provider::AccountProvider;
use ethkey::KeyPair;
use super::helpers::*;
use SyncConfig;

struct TestIoHandler {
	client: Arc<Client>,
}

impl IoHandler<ClientIoMessage> for TestIoHandler {
	fn message(&self, _io: &IoContext<ClientIoMessage>, net_message: &ClientIoMessage) {
		match *net_message {
			ClientIoMessage::UpdateSealing => self.client.update_sealing(),
			ClientIoMessage::SubmitSeal(ref hash, ref seal) => self.client.submit_seal(*hash, seal.clone()),
			ClientIoMessage::BroadcastMessage(ref message) => self.client.broadcast_message(message.clone()),
			ClientIoMessage::NewMessage(ref message) => if let Err(e) = self.client.engine().handle_message(message) {
				panic!("Invalid message received: {}", e);
			},
			_ => {} // ignore other messages
		}
	}
}

#[test]
fn authority_round() {
	let s0 = KeyPair::from_secret("1".sha3()).unwrap();
	let s1 = KeyPair::from_secret("0".sha3()).unwrap();
	let spec_factory = || {
		let spec = Spec::new_test_round();
		let account_provider = AccountProvider::transient_provider();
		account_provider.insert_account(s0.secret().clone(), "").unwrap();
		account_provider.insert_account(s1.secret().clone(), "").unwrap();
		spec.engine.register_account_provider(Arc::new(account_provider));
		spec
	};
	let mut net = TestNet::with_spec(2, SyncConfig::default(), spec_factory);
	let mut net = &mut *net;
	let io_handler0: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler { client: net.peer(0).chain.clone() });
	let io_handler1: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler { client: net.peer(1).chain.clone() });
	// Push transaction to both clients. Only one of them gets lucky to produce a block.
	net.peer(0).chain.miner().set_engine_signer(s0.address(), "".to_owned()).unwrap();
	net.peer(1).chain.miner().set_engine_signer(s1.address(), "".to_owned()).unwrap();
	net.peer(0).chain.engine().register_message_channel(IoChannel::to_handler(Arc::downgrade(&io_handler0)));
	net.peer(1).chain.engine().register_message_channel(IoChannel::to_handler(Arc::downgrade(&io_handler1)));
	net.peer(0).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler1)));
	net.peer(1).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler1)));
	let tx0 = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 21000.into(),
		action: Action::Call(Address::default()),
		value: 0.into(),
		data: Vec::new(),
	}.sign(s0.secret(), None);
	// exchange statuses
	net.sync();
	net.peer(0).chain.miner().import_own_transaction(&*net.peer(0).chain, tx0).unwrap();
	net.sync();
	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 1);
	assert_eq!(net.peer(1).chain.chain_info().best_block_number, 1);

	let tx1 = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 21000.into(),
		action: Action::Call(Address::default()),
		value: 0.into(),
		data: Vec::new(),
	}.sign(s1.secret(), None);
	net.peer(1).chain.miner().import_own_transaction(&*net.peer(1).chain, tx1).unwrap();
	net.peer(1).chain.engine().step();
	net.sync();
	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 2);
	assert_eq!(net.peer(1).chain.chain_info().best_block_number, 2);
}

#[test]
fn tendermint() {
	::env_logger::init().ok();

	let s0 = KeyPair::from_secret("1".sha3()).unwrap();
	let s1 = KeyPair::from_secret("0".sha3()).unwrap();
	let spec_factory = || {
		let spec = Spec::new_test_tendermint();
		let account_provider = AccountProvider::transient_provider();
		account_provider.insert_account(s0.secret().clone(), "").unwrap();
		account_provider.insert_account(s1.secret().clone(), "").unwrap();
		spec.engine.register_account_provider(Arc::new(account_provider));
		spec
	};
	let mut net = TestNet::with_spec(2, SyncConfig::default(), spec_factory);
	let mut net = &mut *net;
	let io_handler0: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler { client: net.peer(0).chain.clone() });
	let io_handler1: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler { client: net.peer(1).chain.clone() });
	// Push transaction to both clients. Only one of them issues a proposal.
	net.peer(0).chain.miner().set_engine_signer(s0.address(), "".to_owned()).unwrap();
	net.peer(1).chain.miner().set_engine_signer(s1.address(), "".to_owned()).unwrap();
	net.peer(0).chain.engine().register_message_channel(IoChannel::to_handler(Arc::downgrade(&io_handler0)));
	net.peer(1).chain.engine().register_message_channel(IoChannel::to_handler(Arc::downgrade(&io_handler1)));
	net.peer(0).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler1)));
	net.peer(1).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler1)));
	let tx0 = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 21000.into(),
		action: Action::Call(Address::default()),
		value: 0.into(),
		data: Vec::new(),
	}.sign(s0.secret(), None);
	// exhange statuses
	net.sync_steps(5);
	net.peer(0).chain.miner().import_own_transaction(&*net.peer(0).chain, tx0).unwrap();
	// Propose
	net.sync();
	// Propose timeout
	net.peer(1).chain.engine().step();
	net.peer(0).chain.engine().step();
	// Precommit
	net.sync();

	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 1);
	assert_eq!(net.peer(1).chain.chain_info().best_block_number, 1);

	let tx1 = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 21000.into(),
		action: Action::Call(Address::default()),
		value: 0.into(),
		data: Vec::new(),
	}.sign(s1.secret(), None);
	net.peer(1).chain.miner().import_own_transaction(&*net.peer(1).chain, tx1).unwrap();
	net.peer(1).chain.engine().step();
	net.sync();
	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 2);
	assert_eq!(net.peer(1).chain.chain_info().best_block_number, 2);
}
