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
use util::*;
use rlp::*;
use io::{IoHandler, IoChannel};
use ethcore::client::PrivateTransactionClient;
use ethcore::service::ClientIoMessage;
use ethcore::spec::Spec;
use ethcore::miner::MinerService;
use ethcore::transaction::*;
use ethcore::account_provider::AccountProvider;
use ethkey::KeyPair;
use tests::helpers::*;
use SyncConfig;

#[test]
pub fn send_private_transaction() {
	// Setup two clients
	let s0 = KeyPair::from_secret_slice(&"1".sha3()).unwrap();
	let s1 = KeyPair::from_secret_slice(&"0".sha3()).unwrap();
	let ap = Arc::new(AccountProvider::transient_provider());
	ap.insert_account(s0.secret().clone(), "").unwrap();
	ap.insert_account(s1.secret().clone(), "").unwrap();

	let chain_id = Spec::new_test_tendermint().chain_id();
	let mut net = TestNet::with_spec_and_accounts(2, SyncConfig::default(), Spec::new_test_tendermint, Some(ap));
	let io_handler0: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler { client: net.peer(0).chain.clone() });
	let io_handler1: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler { client: net.peer(1).chain.clone() });
	net.peer(0).chain.miner().set_engine_signer(s0.address(), "".to_owned()).unwrap();
	net.peer(1).chain.miner().set_engine_signer(s1.address(), "".to_owned()).unwrap();
	net.peer(0).chain.engine().register_client(Arc::downgrade(&net.peer(0).chain));
	net.peer(1).chain.engine().register_client(Arc::downgrade(&net.peer(1).chain));
	net.peer(0).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler0)));
	net.peer(1).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler1)));
	// Exhange statuses
	net.sync();
	// broadcast private transaction	
	let transaction = Transaction {
		nonce: 0.into(),
		gas_price: 0.into(),
		gas: 21000.into(),
		action: Action::Call(Address::default()),
		value: 0.into(),
		data: Vec::new(),
	};
	let signature = net.peer(0).chain.engine().sign(transaction.hash(Some(chain_id)));
	let message = transaction.with_signature(signature.unwrap(), Some(chain_id)).rlp_bytes();
	net.peer(0).chain.broadcast_private_transaction(message.into_vec());
	net.sync();
	net.peer(0).chain.engine().step();
	net.peer(1).chain.engine().step();
	net.sync();

	assert_eq!(net.peer(1).chain.private_transactions().len(), 1);
}
