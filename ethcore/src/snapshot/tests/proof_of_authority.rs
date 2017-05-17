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

//! PoA block chunker and rebuilder tests.

use std::cell::RefCell;
use std::sync::Arc;
use std::str::FromStr;

use account_provider::AccountProvider;
use client::{Client, BlockChainClient, MiningBlockChainClient};
use ethkey::Secret;
use engines::Seal;
use futures::Future;
use miner::MinerService;
use native_contracts::test_contracts::ValidatorSet;
use snapshot::tests::helpers as snapshot_helpers;
use spec::Spec;
use tests::helpers;
use transaction::{Transaction, Action, SignedTransaction};

use util::{Address, Hashable};
use util::kvdb;

const PASS: &'static str = "";
const TRANSITION_BLOCK_1: usize = 2; // block at which the contract becomes activated.
const TRANSITION_BLOCK_2: usize = 6; // block at which the second contract activates.

macro_rules! secret {
	($e: expr) => { Secret::from_slice(&$e.sha3()).expect(format!("sha3({}) not valid secret.", $e).as_str()) }
}

lazy_static! {
	// contract addresses.
	static ref CONTRACT_ADDR_1: Address = Address::from_str("0000000000000000000000000000000000000005").unwrap();
	static ref CONTRACT_ADDR_2: Address = Address::from_str("0000000000000000000000000000000000000006").unwrap();
	// secret: `sha3(1)`, and initial validator.
	static ref RICH_ADDR: Address = Address::from_str("7d577a597b2742b498cb5cf0c26cdcd726d39e6e").unwrap();
	// rich address' secret.
	static ref RICH_SECRET: Secret = secret!("1");
}


/// Contract code used here: https://gist.github.com/rphmeier/2de14fd365a969e3a9e10d77eb9a1e37
/// Account with secrets "1".sha3() is initially the validator.
/// Transitions to the contract at block 2, initially same validator set.
/// Create a new Spec with BasicAuthority which uses a contract at address 5 to determine the current validators using `getValidators`.
/// `native_contracts::test_contracts::ValidatorSet` provides a native wrapper for the ABi.
fn spec_fixed_to_contract() -> Spec {
	let data = include_bytes!("test_validator_contract.json");
	Spec::load(&data[..]).unwrap()
}

// creates an account provider, filling it with accounts from all the given
// secrets and password `PASS`.
// returns addresses corresponding to secrets.
fn make_accounts(secrets: &[Secret]) -> (Arc<AccountProvider>, Vec<Address>) {
	let provider = AccountProvider::transient_provider();

	let addrs = secrets.iter()
		.cloned()
		.map(|s| provider.insert_account(s, PASS).unwrap())
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
	let client = helpers::generate_dummy_client_with_spec_and_accounts(
		spec_fixed_to_contract, Some(accounts.clone()));

	let mut cur_signers = vec![*RICH_ADDR];
	{
		let engine = client.engine();
		engine.register_client(Arc::downgrade(&client));
	}

	{
		// push a block with given number, signed by one of the signers, with given transactions.
		let push_block = |signers: &[Address], n, txs: Vec<SignedTransaction>| {
			use block::IsBlock;

			let engine = client.engine();
			let idx = n as usize % signers.len();
			engine.set_signer(accounts.clone(), signers[idx], PASS.to_owned());

			trace!(target: "snapshot", "Pushing block #{}, {} txs, author={}", n, txs.len(), signers[idx]);

			let mut open_block = client.prepare_open_block(signers[idx], (5_000_000.into(), 5_000_000.into()), Vec::new());
			for tx in txs {
				open_block.push_transaction(tx, None).unwrap();
			}
			let block = open_block.close_and_lock();
			let seal = match engine.generate_seal(block.block()) {
				Seal::Regular(seal) => seal,
				_ => panic!("Unable to generate seal for dummy chain block #{}", n),
			};
			let block = block.seal(&*engine, seal).unwrap();

			client.import_sealed_block(block).unwrap();
		};

		// execution callback for native contract: push transaction to be sealed.
		let nonce = RefCell::new(client.engine().account_start_nonce());
		let exec = |addr, data| {
			let mut nonce = nonce.borrow_mut();
			let transaction = Transaction {
				nonce: *nonce,
				gas_price: 0.into(),
				gas: 1_000_000.into(),
				action: Action::Call(addr),
				value: 0.into(),
				data: data,
			}.sign(&*RICH_SECRET, client.signing_network_id());

			client.miner().import_own_transaction(&*client, transaction.into()).unwrap();

			*nonce = *nonce + 1.into();
			Ok(Vec::new())
		};

		let contract_1 = ValidatorSet::new(*CONTRACT_ADDR_1);
		let contract_2 = ValidatorSet::new(*CONTRACT_ADDR_2);

		// apply all transitions.
		for transition in transitions {
			let (num, manual, new_set) = match transition {
				Transition::Manual(num, new_set) => (num, true, new_set),
				Transition::Implicit(num, new_set) => (num, false, new_set),
			};

			if num < TRANSITION_BLOCK_1 {
				panic!("Bad test: issued epoch change before transition to contract.");
			}

			for number in client.chain_info().best_block_number + 1 .. num as u64 {
				push_block(&cur_signers, number, vec![]);
			}

			let pending = if manual {
				trace!(target: "snapshot", "applying set transition at block #{}", num);
				let contract = match num >= TRANSITION_BLOCK_2 {
					true => &contract_2,
					false => &contract_1,
				};

				contract.set_validators(&exec, new_set.clone()).wait().unwrap();
				client.ready_transactions()
					.into_iter()
					.map(|x| x.transaction)
					.collect()
			} else {
				Vec::new()
			};

			push_block(&cur_signers, num as u64, pending);
			cur_signers = new_set;
		}

		// make blocks beyond.
		for number in (client.chain_info().best_block_number..).take(blocks_beyond) {
			push_block(&cur_signers, number + 1, vec![]);
		}
	}

	client
}

#[test]
fn fixed_to_contract() {
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

	assert!(provider.has_account(*RICH_ADDR).unwrap());

	let client = make_chain(provider, 1, vec![
		Transition::Manual(3, vec![addrs[2], addrs[3], addrs[5], addrs[7]]),
		Transition::Manual(4, vec![addrs[0], addrs[1], addrs[4], addrs[6]]),
	]);

	assert_eq!(client.chain_info().best_block_number, 5);
	let reader = snapshot_helpers::snap(&*client);

	let new_db = kvdb::in_memory(::db::NUM_COLUMNS.unwrap_or(0));
	let spec = spec_fixed_to_contract();

	snapshot_helpers::restore(Arc::new(new_db), &*spec.engine, &**reader, &spec.genesis_block()).unwrap();
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

	assert!(provider.has_account(*RICH_ADDR).unwrap());

	let client = make_chain(provider, 2, vec![
		Transition::Manual(3, vec![addrs[2], addrs[3], addrs[5], addrs[7]]),
		Transition::Manual(4, vec![addrs[0], addrs[1], addrs[4], addrs[6]]),
		Transition::Implicit(5, vec![addrs[0]]),
		Transition::Manual(8, vec![addrs[2], addrs[4], addrs[6], addrs[7]]),
	]);

	assert_eq!(client.chain_info().best_block_number, 10);
	let reader = snapshot_helpers::snap(&*client);
	let new_db = kvdb::in_memory(::db::NUM_COLUMNS.unwrap_or(0));
	let spec = spec_fixed_to_contract();

	snapshot_helpers::restore(Arc::new(new_db), &*spec.engine, &**reader, &spec.genesis_block()).unwrap();
}
