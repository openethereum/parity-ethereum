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
use util::*;
use rlp::*;
use io::{IoHandler, IoChannel};
use ethcore::service::ClientIoMessage;
use ethcore::client::{MiningBlockChainClient, BlockChainClient, Client, BlockId};
use ethcore::spec::Spec;
use ethcore::miner::MinerService;
use ethcore::transaction::*;
use ethcore::private_transactions::{VerificationAccount, ProviderConfig, PublicSigningAccount};
use ethcore::account_provider::AccountProvider;
use ethkey::KeyPair;
use tests::helpers::*;
use ethcore::engines::Engine;
use bytes::Bytes;
use bigint::prelude::U256;
use bigint::hash::H256;
use ethcore::CreateContractAddress;
use rustc_hex::FromHex;
use SyncConfig;

fn push_block_with_transactions(client: &Arc<Client>, engine: &Engine, transactions: &[SignedTransaction]) {
	let block_number = client.chain_info().best_block_number as u64 + 1;

	let mut b = client.prepare_open_block(Address::default(), (0.into(), 5000000.into()), Bytes::new());
	b.set_difficulty(U256::from(0x20000));
	b.set_timestamp(block_number * 10);

	for t in transactions {
		b.push_transaction(t.clone(), None).unwrap();
	}
	let b = b.close_and_lock().seal(engine, vec![]).unwrap();

	if let Err(e) = client.import_block(b.rlp_bytes()) {
		panic!("error importing block which is valid by definition: {:?}", e);
	}

	client.flush_queue();
	client.import_verified_blocks();
}

fn contract_address(sender: &Address, nonce: &U256) -> Address {
	let mut stream = RlpStream::new_list(2);
	stream.append(sender);
	stream.append(nonce);
	From::from(keccak(stream.as_raw()))
}

fn get_seal_spec() -> Spec {
	let spec_data = r#"
	{
		"name": "PrivateTransactions",
		"engine": {
			"instantSeal": null
		},
		"params": {
			"gasLimitBoundDivisor": "0x0400",
			"accountStartNonce": "0x0",
			"maximumExtraDataSize": "0x20",
			"minGasLimit": "0x1388",
			"networkID" : "0x11"
		},
		"genesis": {
			"seal": {
				"generic": "0x0"
			},
			"difficulty": "0x20000",
			"author": "0x0000000000000000000000000000000000000000",
			"timestamp": "0x00",
			"parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
			"extraData": "0x",
			"gasLimit": "0x989680"
		},
		"accounts": {
			"0000000000000000000000000000000000000001": { "balance": "1", "builtin": { "name": "ecrecover", "pricing": { "linear": { "base": 3000, "word": 0 } } } },
			"0000000000000000000000000000000000000002": { "balance": "1", "builtin": { "name": "sha256", "pricing": { "linear": { "base": 60, "word": 12 } } } },
			"0000000000000000000000000000000000000003": { "balance": "1", "builtin": { "name": "ripemd160", "pricing": { "linear": { "base": 600, "word": 120 } } } },
			"0000000000000000000000000000000000000004": { "balance": "1", "builtin": { "name": "identity", "pricing": { "linear": { "base": 15, "word": 3 } } } }
		}
	}
	"#;

	Spec::load(::std::env::temp_dir(), spec_data.as_bytes()).unwrap()
}

#[test]
fn send_private_transaction() {
	// Setup two clients
	let s0 = KeyPair::from_secret_slice(&keccak("1")).unwrap();
	let s1 = KeyPair::from_secret_slice(&keccak("0")).unwrap();
	let ap = Arc::new(AccountProvider::transient_provider());
	ap.insert_account(s0.secret().clone(), "").unwrap();
	ap.insert_account(s1.secret().clone(), "").unwrap();

	let mut net = TestNet::with_spec_and_accounts(2, SyncConfig::default(), get_seal_spec, Some(ap.clone()));
	let client0 = net.peer(0).chain.clone();
	let client1 = net.peer(1).chain.clone();
	let io_handler0: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler { client: client0.clone() });
	let io_handler1: Arc<IoHandler<ClientIoMessage>> = Arc::new(TestIoHandler { client: client1.clone() });

	client0.miner().set_engine_signer(s0.address(), "".to_owned()).unwrap();
	client1.miner().set_engine_signer(s1.address(), "".to_owned()).unwrap();
	client0.engine().register_client(Arc::downgrade(&client0) as _);
	client1.engine().register_client(Arc::downgrade(&client1) as _);
	client0.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler0)));
	client1.set_io_channel(IoChannel::to_handler(Arc::downgrade(&io_handler1)));

	let address = contract_address(&s0.address(), &0.into());
	let chain_id = client0.signing_chain_id();

	// Exhange statuses
	net.sync();

	// Setup providers
	let provider0 = client0.private_transactions_provider().clone();
	let provider1 = client1.private_transactions_provider().clone();

	let mut validator_config = ProviderConfig{
		validator_accounts: Vec::new(),
		signer_account: PublicSigningAccount::default(),
	};
	validator_config.validator_accounts.push(VerificationAccount::new(s1.address(), Some("".into())));
	provider1.set_config(validator_config);

	let signer_config = ProviderConfig{
		validator_accounts: Vec::new(),
		signer_account: PublicSigningAccount::new(s1.address(), Some("".into())),
	};
	provider0.set_config(signer_config);

	provider0.register_account_provider(Arc::downgrade(&ap));
	provider1.register_account_provider(Arc::downgrade(&ap));

	let test_res = provider0.test_encryption();

	// Create contract
	let private_contract_test = "6060604052341561000f57600080fd5b60d88061001d6000396000f30060606040526000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff1680630c55699c146046578063bc64b76d14607457600080fd5b3415605057600080fd5b60566098565b60405180826000191660001916815260200191505060405180910390f35b3415607e57600080fd5b6096600480803560001916906020019091905050609e565b005b60005481565b8060008160001916905550505600a165627a7a723058206acbdf4b15ca4c2d43e1b1879b830451a34f1e9d02ff1f2f394d8d857e79d2080029".from_hex().unwrap();
	let mut private_create_tx = Transaction::default();
	private_create_tx.action = Action::Create;
	private_create_tx.data = private_contract_test;
	private_create_tx.gas = 200000.into();
	let private_create_tx_signed = private_create_tx.sign(&s0.secret(), None);
	let validators = vec![s1.address()];
	let public_tx = provider0.public_creation_transaction(BlockId::Pending, &private_create_tx_signed, &validators, 0.into(), 0.into()).unwrap();
	let public_tx = public_tx.sign(&s0.secret(), chain_id);
	let public_tx_copy = public_tx.clone();
	push_block_with_transactions(&client0, client0.engine(), &[public_tx]);
	push_block_with_transactions(&client1, client1.engine(), &[public_tx_copy]);

	net.sync();

	//Create private transaction for modifying state
	let mut private_tx = Transaction::default();
	private_tx.action = Action::Call(address.clone());
	private_tx.data = "bc64b76d2a00000000000000000000000000000000000000000000000000000000000000".from_hex().unwrap(); //setX(42)
	private_tx.gas = 120000.into();
	private_tx.nonce = 1.into();
	let private_tx = private_tx.sign(&s0.secret(), None);
	assert!(provider0.create_private_transaction(private_tx, &address).is_ok());

	//Exchange with signature and create corresponding public transaction
	net.sync();

	let future_transactions = client0.miner().future_transactions();
	assert_eq!(future_transactions.len(), 1);
}
