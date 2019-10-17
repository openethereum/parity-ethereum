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

//! PoA block chunker and rebuilder tests.

use std::cell::RefCell;
use std::sync::Arc;
use std::str::FromStr;

use accounts::AccountProvider;
use client_traits::{BlockChainClient, ChainInfo};
use common_types::transaction::{Transaction, Action, SignedTransaction};
use ethabi_contract::use_contract;
use ethcore::{
	client::Client,
	test_helpers::{self, generate_dummy_client_with_spec},
	miner::{self, MinerService},
};
use ethereum_types::Address;
use parity_crypto::publickey::Secret;
use keccak_hash::keccak;
use lazy_static::lazy_static;
use log::trace;
use spec::Spec;
use tempdir::TempDir;

use crate::helpers as snapshot_helpers;

use_contract!(test_validator_set, "../../res/contracts/test_validator_set.json");

const PASS: &'static str = "";
const TRANSITION_BLOCK_1: usize = 2; // block at which the contract becomes activated.
const TRANSITION_BLOCK_2: usize = 10; // block at which the second contract activates.

macro_rules! secret {
	($e: expr) => { Secret::from(keccak($e).0) }
}

lazy_static! {
	// contract addresses.
	static ref CONTRACT_ADDR_1: Address = Address::from_str("0000000000000000000000000000000000000005").unwrap();
	static ref CONTRACT_ADDR_2: Address = Address::from_str("0000000000000000000000000000000000000006").unwrap();
	// secret: `keccak(1)`, and initial validator.
	static ref RICH_ADDR: Address = Address::from_str("7d577a597b2742b498cb5cf0c26cdcd726d39e6e").unwrap();
	// rich address' secret.
	static ref RICH_SECRET: Secret = secret!("1");
}

/// Contract code used here: https://gist.github.com/anonymous/2a43783647e0f0dfcc359bd6fd81d6d9
/// Account with secrets keccak("1") is initially the validator.
/// Transitions to the contract at block 2, initially same validator set.
/// Create a new Spec with AuthorityRound which uses a contract at address 5 to determine the current validators using `getValidators`.
/// `test_validator_set::ValidatorSet` provides a native wrapper for the ABi.
fn spec_fixed_to_contract() -> Spec {
	let data = include_bytes!("test_validator_contract.json");
	let tempdir = TempDir::new("").unwrap();
	Spec::load(&tempdir.path(), &data[..]).unwrap()
}

// creates an account provider, filling it with accounts from all the given
// secrets and password `PASS`.
// returns addresses corresponding to secrets.
fn make_accounts(secrets: &[Secret]) -> (Arc<AccountProvider>, Vec<Address>) {
	let provider = AccountProvider::transient_provider();

	let addrs = secrets.iter()
		.cloned()
		.map(|s| provider.insert_account(s, &PASS.into()).unwrap())
		.collect();

	(Arc::new(provider), addrs)
}

// validator transition. block number and new validators. must be after `TRANSITION_BLOCK`.
// all addresses in the set must be in the account provider.
enum Transition {
	// manual transition via transaction
	Manual(usize, Vec<Address>),
	// implicit transition via multi-set
	Implicit(usize, Vec<Address>),
}

// create a chain with the given transitions and some blocks beyond that transition.
fn make_chain(accounts: Arc<AccountProvider>, blocks_beyond: usize, transitions: Vec<Transition>) -> Arc<Client> {
	let client = generate_dummy_client_with_spec(spec_fixed_to_contract);

	let mut cur_signers = vec![*RICH_ADDR];
	{
		let engine = client.engine();
		engine.register_client(Arc::downgrade(&client) as _);
	}

	{
		// push a block with given number, signed by one of the signers, with given transactions.
		let push_block = |signers: &[Address], n, txs: Vec<SignedTransaction>| {
			let idx = n as usize % signers.len();
			trace!(target: "snapshot", "Pushing block #{}, {} txs, author={}",
				n, txs.len(), signers[idx]);

			let signer = Box::new((accounts.clone(), signers[idx], PASS.into()));
			client.miner().set_author(miner::Author::Sealer(signer));
			client.miner().import_external_transactions(&*client,
				txs.into_iter().map(Into::into).collect());

			client.engine().step();

			assert_eq!(client.chain_info().best_block_number, n);
		};

		// execution callback for native contract: push transaction to be sealed.
		let nonce = RefCell::new(client.engine().account_start_nonce(0));

		// create useless transactions vector so we don't have to dig in
		// and force sealing.
		let make_useless_transactions = || {
			let mut nonce = nonce.borrow_mut();
			let transaction = Transaction {
				nonce: *nonce,
				gas_price: 1.into(),
				gas: 21_000.into(),
				action: Action::Call(Address::zero()),
				value: 1.into(),
				data: Vec::new(),
			}.sign(&*RICH_SECRET, client.signing_chain_id());

			*nonce = *nonce + 1;
			vec![transaction]
		};

		// apply all transitions.
		for transition in transitions {
			let (num, manual, new_set) = match transition {
				Transition::Manual(num, new_set) => (num, true, new_set),
				Transition::Implicit(num, new_set) => (num, false, new_set),
			};

			if num < TRANSITION_BLOCK_1 {
				panic!("Bad test: issued epoch change before transition to contract.");
			}

			if (num as u64) < client.chain_info().best_block_number {
				panic!("Bad test: issued epoch change before previous transition finalized.");
			}

			for number in client.chain_info().best_block_number + 1 .. num as u64 {
				push_block(&cur_signers, number, make_useless_transactions());
			}

			let pending = if manual {
				trace!(target: "snapshot", "applying set transition at block #{}", num);
				let address = match num >= TRANSITION_BLOCK_2 {
					true => &CONTRACT_ADDR_2 as &Address,
					false => &CONTRACT_ADDR_1 as &Address,
				};

				let data = test_validator_set::functions::set_validators::encode_input(new_set.clone());
				let mut nonce = nonce.borrow_mut();
				let transaction = Transaction {
					nonce: *nonce,
					gas_price: 0.into(),
					gas: 1_000_000.into(),
					action: Action::Call(*address),
					value: 0.into(),
					data,
				}.sign(&*RICH_SECRET, client.signing_chain_id());

				*nonce = *nonce + 1;
				vec![transaction]
			} else {
				make_useless_transactions()
			};

			// push transition block.
			push_block(&cur_signers, num as u64, pending);

			// push blocks to finalize transition
			for finalization_count in 1.. {
				if finalization_count * 2 > cur_signers.len() { break }
				push_block(&cur_signers, (num + finalization_count) as u64, make_useless_transactions());
			}

			cur_signers = new_set;
		}

		// make blocks beyond.
		for number in (client.chain_info().best_block_number..).take(blocks_beyond) {
			push_block(&cur_signers, number + 1, make_useless_transactions());
		}
	}

	client
}

#[test]
fn fixed_to_contract_only() {
	let (provider, addrs) = make_accounts(&[
		RICH_SECRET.clone(),
		secret!("foo"),
		secret!("bar"),
		secret!("test"),
		secret!("signer"),
		secret!("crypto"),
		secret!("wizard"),
		secret!("dog42"),
	]);

	assert!(provider.has_account(*RICH_ADDR));

	let client = make_chain(provider, 3, vec![
		Transition::Manual(3, vec![addrs[2], addrs[3], addrs[5], addrs[7]]),
		Transition::Manual(6, vec![addrs[0], addrs[1], addrs[4], addrs[6]]),
	]);

	// 6, 7, 8 prove finality for transition at 6.
	// 3 beyond gets us to 11.
	assert_eq!(client.chain_info().best_block_number, 11);
	let (reader, _tempdir) = snapshot_helpers::snap(&*client);

	let new_db = test_helpers::new_db();
	let spec = spec_fixed_to_contract();

	// ensure fresh engine's step matches.
	for _ in 0..11 { spec.engine.step() }
	snapshot_helpers::restore(new_db, &*spec.engine, &*reader, &spec.genesis_block()).unwrap();
}

#[test]
fn fixed_to_contract_to_contract() {
	let (provider, addrs) = make_accounts(&[
		RICH_SECRET.clone(),
		secret!("foo"),
		secret!("bar"),
		secret!("test"),
		secret!("signer"),
		secret!("crypto"),
		secret!("wizard"),
		secret!("dog42"),
	]);

	assert!(provider.has_account(*RICH_ADDR));

	let client = make_chain(provider, 3, vec![
		Transition::Manual(3, vec![addrs[2], addrs[3], addrs[5], addrs[7]]),
		Transition::Manual(6, vec![addrs[0], addrs[1], addrs[4], addrs[6]]),
		Transition::Implicit(10, vec![addrs[0]]),
		Transition::Manual(13, vec![addrs[2], addrs[4], addrs[6], addrs[7]]),
	]);

	assert_eq!(client.chain_info().best_block_number, 16);
	let (reader, _tempdir) = snapshot_helpers::snap(&*client);
	let new_db = test_helpers::new_db();
	let spec = spec_fixed_to_contract();

	for _ in 0..16 { spec.engine.step() }
	snapshot_helpers::restore(new_db, &*spec.engine, &*reader, &spec.genesis_block()).unwrap();
}
