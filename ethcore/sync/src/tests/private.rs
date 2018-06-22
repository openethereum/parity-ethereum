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

use std::sync::Arc;
use hash::keccak;
use io::{IoHandler, IoChannel};
use ethcore::client::{BlockChainClient, BlockId, ClientIoMessage};
use ethcore::spec::Spec;
use ethcore::miner::MinerService;
use ethcore::CreateContractAddress;
use transaction::{Transaction, Action};
use ethcore::executive::{contract_address};
use ethcore::test_helpers::{push_block_with_transactions};
use ethcore_private_tx::{Provider, ProviderConfig, NoopEncryptor, Importer};
use ethcore::account_provider::AccountProvider;
use ethkey::{KeyPair};
use tests::helpers::{TestNet, TestIoHandler};
use rustc_hex::FromHex;
use SyncConfig;

fn seal_spec() -> Spec {
	let spec_data = include_str!("../res/private_spec.json");
	Spec::load(&::std::env::temp_dir(), spec_data.as_bytes()).unwrap()
}

#[test]
fn send_private_transaction() {
	// Setup two clients
	let s0 = KeyPair::from_secret_slice(&keccak("1")).unwrap();
	let s1 = KeyPair::from_secret_slice(&keccak("0")).unwrap();
	let ap = Arc::new(AccountProvider::transient_provider());
	ap.insert_account(s0.secret().clone(), &"".into()).unwrap();
	ap.insert_account(s1.secret().clone(), &"".into()).unwrap();

	let mut net = TestNet::with_spec_and_accounts(2, SyncConfig::default(), seal_spec, Some(ap.clone()));
	let client0 = net.peer(0).chain.clone();
	let client1 = net.peer(1).chain.clone();
	let io_handler0: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler::new(net.peer(0).chain.clone()));
	let io_handler1: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler::new(net.peer(1).chain.clone()));

	net.peer(0).miner.set_author(s0.address(), Some("".into())).unwrap();
	net.peer(1).miner.set_author(s1.address(), Some("".into())).unwrap();
	net.peer(0).chain.engine().register_client(Arc::downgrade(&net.peer(0).chain) as _);
	net.peer(1).chain.engine().register_client(Arc::downgrade(&net.peer(1).chain) as _);
	net.peer(0).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler0)));
	net.peer(1).chain.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler1)));

	let (address, _) = contract_address(CreateContractAddress::FromSenderAndNonce, &s0.address(), &0.into(), &[]);
	let chain_id = client0.signing_chain_id();

	// Exhange statuses
	net.sync();

	// Setup private providers
	let validator_config = ProviderConfig{
		validator_accounts: vec![s1.address()],
		signer_account: None,
		passwords: vec!["".into()],
	};

	let signer_config = ProviderConfig{
		validator_accounts: Vec::new(),
		signer_account: Some(s0.address()),
		passwords: vec!["".into()],
	};

	let pm0 = Arc::new(Provider::new(
			client0.clone(),
			net.peer(0).miner.clone(),
			ap.clone(),
			Box::new(NoopEncryptor::default()),
			signer_config,
			IoChannel::to_handler(Arc::downgrade(&io_handler0)),
	));
	pm0.add_notify(net.peers[0].clone());

	let pm1 = Arc::new(Provider::new(
			client1.clone(),
			net.peer(1).miner.clone(),
			ap.clone(),
			Box::new(NoopEncryptor::default()),
			validator_config,
			IoChannel::to_handler(Arc::downgrade(&io_handler1)),
	));
	pm1.add_notify(net.peers[1].clone());

	// Create and deploy contract
	let private_contract_test = "6060604052341561000f57600080fd5b60d88061001d6000396000f30060606040526000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff1680630c55699c146046578063bc64b76d14607457600080fd5b3415605057600080fd5b60566098565b60405180826000191660001916815260200191505060405180910390f35b3415607e57600080fd5b6096600480803560001916906020019091905050609e565b005b60005481565b8060008160001916905550505600a165627a7a723058206acbdf4b15ca4c2d43e1b1879b830451a34f1e9d02ff1f2f394d8d857e79d2080029".from_hex().unwrap();
	let mut private_create_tx = Transaction::default();
	private_create_tx.action = Action::Create;
	private_create_tx.data = private_contract_test;
	private_create_tx.gas = 200000.into();
	let private_create_tx_signed = private_create_tx.sign(&s0.secret(), None);
	let validators = vec![s1.address()];
	let (public_tx, _) = pm0.public_creation_transaction(BlockId::Latest, &private_create_tx_signed, &validators, 0.into()).unwrap();
	let public_tx = public_tx.sign(&s0.secret(), chain_id);

	let public_tx_copy = public_tx.clone();
	push_block_with_transactions(&client0, &[public_tx]);
	push_block_with_transactions(&client1, &[public_tx_copy]);

	net.sync();

	//Create private transaction for modifying state
	let mut private_tx = Transaction::default();
	private_tx.action = Action::Call(address.clone());
	private_tx.data = "bc64b76d2a00000000000000000000000000000000000000000000000000000000000000".from_hex().unwrap(); //setX(42)
	private_tx.gas = 120000.into();
	private_tx.nonce = 1.into();
	let private_tx = private_tx.sign(&s0.secret(), None);
	assert!(pm0.create_private_transaction(private_tx).is_ok());

	//send private transaction message to validator
	net.sync();

	let validator_handler = net.peer(1).private_tx_handler.clone();
	let received_private_transactions = validator_handler.txs.lock().clone();
	assert_eq!(received_private_transactions.len(), 1);

	//process received private transaction message
	let private_transaction = received_private_transactions[0].clone();
	assert!(pm1.import_private_transaction(&private_transaction).is_ok());

	//send signed response
	net.sync();

	let sender_handler = net.peer(0).private_tx_handler.clone();
	let received_signed_private_transactions = sender_handler.signed_txs.lock().clone();
	assert_eq!(received_signed_private_transactions.len(), 1);

	//process signed response
	let signed_private_transaction = received_signed_private_transactions[0].clone();
	assert!(pm0.import_signed_private_transaction(&signed_private_transaction).is_ok());
	let local_transactions = net.peer(0).miner.local_transactions();
	assert_eq!(local_transactions.len(), 1);
}
