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

//! Contract for private transactions tests.

extern crate rustc_hex;
extern crate ethcore;
extern crate ethkey;
extern crate keccak_hash as hash;
extern crate ethcore_io;
extern crate ethcore_logger;
extern crate ethcore_private_tx;
extern crate ethcore_transaction;

#[macro_use]
extern crate log;

use std::sync::Arc;
use rustc_hex::FromHex;

use ethcore::CreateContractAddress;
use ethcore::account_provider::AccountProvider;
use ethcore::client::BlockChainClient;
use ethcore::client::BlockId;
use ethcore::executive::{contract_address};
use ethcore::miner::Miner;
use ethcore::test_helpers::{generate_dummy_client, push_block_with_transactions};
use ethcore_transaction::{Transaction, Action};
use ethkey::{Secret, KeyPair, Signature};
use hash::keccak;

use ethcore_private_tx::{NoopEncryptor, Provider, ProviderConfig};

#[test]
fn private_contract() {
	// This uses a simple private contract: contract Test1 { bytes32 public x; function setX(bytes32 _x) { x = _x; } }
	ethcore_logger::init_log();
	let client = generate_dummy_client(0);
	let chain_id = client.signing_chain_id();
	let key1 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000011")).unwrap();
	let _key2 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000012")).unwrap();
	let key3 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000013")).unwrap();
	let key4 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000014")).unwrap();
	let ap = Arc::new(AccountProvider::transient_provider());
	ap.insert_account(key1.secret().clone(), &"".into()).unwrap();
	ap.insert_account(key3.secret().clone(), &"".into()).unwrap();
	ap.insert_account(key4.secret().clone(), &"".into()).unwrap();

	let config = ProviderConfig{
		validator_accounts: vec![key3.address(), key4.address()],
		signer_account: None,
		passwords: vec!["".into()],
	};

	let io = ethcore_io::IoChannel::disconnected();
	let miner = Arc::new(Miner::new_for_tests(&::ethcore::spec::Spec::new_test(), None));
	let pm = Arc::new(Provider::new(
			client.clone(),
			miner,
			ap.clone(),
			Box::new(NoopEncryptor::default()),
			config,
			io,
	));

	let (address, _) = contract_address(CreateContractAddress::FromSenderAndNonce, &key1.address(), &0.into(), &[]);

	trace!("Creating private contract");
	let private_contract_test = "6060604052341561000f57600080fd5b60d88061001d6000396000f30060606040526000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff1680630c55699c146046578063bc64b76d14607457600080fd5b3415605057600080fd5b60566098565b60405180826000191660001916815260200191505060405180910390f35b3415607e57600080fd5b6096600480803560001916906020019091905050609e565b005b60005481565b8060008160001916905550505600a165627a7a723058206acbdf4b15ca4c2d43e1b1879b830451a34f1e9d02ff1f2f394d8d857e79d2080029".from_hex().unwrap();
	let mut private_create_tx = Transaction::default();
	private_create_tx.action = Action::Create;
	private_create_tx.data = private_contract_test;
	private_create_tx.gas = 200000.into();
	let private_create_tx_signed = private_create_tx.sign(&key1.secret(), None);
	let validators = vec![key3.address(), key4.address()];
	let (public_tx, _) = pm.public_creation_transaction(BlockId::Latest, &private_create_tx_signed, &validators, 0.into()).unwrap();
	let public_tx = public_tx.sign(&key1.secret(), chain_id);
	trace!("Transaction created. Pushing block");
	push_block_with_transactions(&client, &[public_tx]);

	trace!("Querying default private state");
	let mut query_tx = Transaction::default();
	query_tx.action = Action::Call(address.clone());
	query_tx.data = "0c55699c".from_hex().unwrap();  // getX
	query_tx.gas = 50000.into();
	query_tx.nonce = 1.into();
	let query_tx = query_tx.sign(&key1.secret(), chain_id);
	let result = pm.private_call(BlockId::Latest, &query_tx).unwrap();
	assert_eq!(&result.output[..], &("0000000000000000000000000000000000000000000000000000000000000000".from_hex().unwrap()[..]));
	assert_eq!(pm.get_validators(BlockId::Latest, &address).unwrap(), validators);

	trace!("Modifying private state");
	let mut private_tx = Transaction::default();
	private_tx.action = Action::Call(address.clone());
	private_tx.data = "bc64b76d2a00000000000000000000000000000000000000000000000000000000000000".from_hex().unwrap(); //setX(42)
	private_tx.gas = 120000.into();
	private_tx.nonce = 1.into();
	let private_tx = private_tx.sign(&key1.secret(), None);
	let private_contract_nonce = pm.get_contract_nonce(&address, BlockId::Latest).unwrap();
	let private_state = pm.execute_private_transaction(BlockId::Latest, &private_tx).unwrap();
	let nonced_state_hash = pm.calculate_state_hash(&private_state, private_contract_nonce);
	let signatures: Vec<_> = [&key3, &key4].iter().map(|k|
		Signature::from(::ethkey::sign(&k.secret(), &nonced_state_hash).unwrap().into_electrum())).collect();
	let public_tx = pm.public_transaction(private_state, &private_tx, &signatures, 1.into(), 0.into()).unwrap();
	let public_tx = public_tx.sign(&key1.secret(), chain_id);
	push_block_with_transactions(&client, &[public_tx]);

	trace!("Querying private state");
	let mut query_tx = Transaction::default();
	query_tx.action = Action::Call(address.clone());
	query_tx.data = "0c55699c".from_hex().unwrap();  // getX
	query_tx.gas = 50000.into();
	query_tx.nonce = 2.into();
	let query_tx = query_tx.sign(&key1.secret(), chain_id);
	let result = pm.private_call(BlockId::Latest, &query_tx).unwrap();
	assert_eq!(&result.output[..], &("2a00000000000000000000000000000000000000000000000000000000000000".from_hex().unwrap()[..]));
	assert_eq!(pm.get_validators(BlockId::Latest, &address).unwrap(), validators);

	// Now try modification with just one signature
	trace!("Modifying private state");
	let mut private_tx = Transaction::default();
	private_tx.action = Action::Call(address.clone());
	private_tx.data = "bc64b76d2b00000000000000000000000000000000000000000000000000000000000000".from_hex().unwrap(); //setX(43)
	private_tx.gas = 120000.into();
	private_tx.nonce = 2.into();
	let private_tx = private_tx.sign(&key1.secret(), None);
	let private_state = pm.execute_private_transaction(BlockId::Latest, &private_tx).unwrap();
	let private_state_hash = keccak(&private_state);
	let signatures: Vec<_> = [&key4].iter().map(|k|
		Signature::from(::ethkey::sign(&k.secret(), &private_state_hash).unwrap().into_electrum())).collect();
	let public_tx = pm.public_transaction(private_state, &private_tx, &signatures, 2.into(), 0.into()).unwrap();
	let public_tx = public_tx.sign(&key1.secret(), chain_id);
	push_block_with_transactions(&client, &[public_tx]);

	trace!("Querying private state");
	let mut query_tx = Transaction::default();
	query_tx.action = Action::Call(address.clone());
	query_tx.data = "0c55699c".from_hex().unwrap();  // getX
	query_tx.gas = 50000.into();
	query_tx.nonce = 3.into();
	let query_tx = query_tx.sign(&key1.secret(), chain_id);
	let result = pm.private_call(BlockId::Latest, &query_tx).unwrap();
	assert_eq!(result.output, "2a00000000000000000000000000000000000000000000000000000000000000".from_hex().unwrap());
}
