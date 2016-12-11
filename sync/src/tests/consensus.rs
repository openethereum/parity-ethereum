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

use util::*;
use ethcore::client::BlockChainClient;
use ethcore::spec::Spec;
use ethcore::miner::MinerService;
use ethcore::transaction::*;
use ethcore::account_provider::AccountProvider;
use ethkey::KeyPair;
use super::helpers::*;
use SyncConfig;

#[test]
fn test_authority_round() {
	::env_logger::init().ok();

	let s1 = KeyPair::from_secret("1".sha3()).unwrap();
	let s2 = KeyPair::from_secret("0".sha3()).unwrap();
	let spec_factory = || {
		let spec = Spec::new_test_round();
		let account_provider = AccountProvider::transient_provider();
		account_provider.insert_account(s1.secret().clone(), "").unwrap();
		account_provider.insert_account(s2.secret().clone(), "").unwrap();
		spec.engine.register_account_provider(Arc::new(account_provider));
		spec
	};
	let mut net = TestNet::new_with_spec(2, SyncConfig::default(), spec_factory);
	let mut net = &mut *net;
	// Push transaction to both clients. Only one of them gets lucky to mine a block.
	net.peer(0).chain.miner().set_author(s1.address());
	net.peer(0).chain.engine().set_signer(s1.address(), "".to_owned());
	net.peer(1).chain.miner().set_author(s2.address());
	net.peer(1).chain.engine().set_signer(s2.address(), "".to_owned());
	let tx1 = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 21000.into(),
		action: Action::Call(Address::default()),
		value: 0.into(),
		data: Vec::new(),
	}.sign(s1.secret(), None);
	// exhange statuses
	net.sync_steps(5);
	net.peer(0).chain.miner().import_own_transaction(&net.peer(0).chain, tx1).unwrap();
	net.sync();
	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 1);
	assert_eq!(net.peer(1).chain.chain_info().best_block_number, 1);

	let tx2 = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 21000.into(),
		action: Action::Call(Address::default()),
		value: 0.into(),
		data: Vec::new(),
	}.sign(s2.secret(), None);
	net.peer(1).chain.miner().import_own_transaction(&net.peer(1).chain, tx2).unwrap();
	net.peer(1).chain.engine().step();
	net.peer(1).chain.miner().update_sealing(&net.peer(1).chain);
	net.sync();
	assert_eq!(net.peer(0).chain.chain_info().best_block_number, 2);
	assert_eq!(net.peer(1).chain.chain_info().best_block_number, 2);
}

