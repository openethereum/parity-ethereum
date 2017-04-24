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
use engines::{Engine, Seal};
use futures::Future;
use miner::MinerService;
use native_contracts::test_contracts::ValidatorSet;
use snapshot::{PoaSnapshot, StateRebuilder};
use snapshot::tests::helpers as snapshot_helpers;
use spec::Spec;
use tests::helpers;
use transaction::{Transaction, Action, SignedTransaction};

use util::{Address, Hashable};

const PASS: &'static str = "";
const TRANSITION_BLOCK: usize = 2; // block at which the contract becomes activated.

macro_rules! secret {
	($e: expr) => { Secret::from_slice(&$e.sha3()).expect(format!("sha3({}) not valid secret.", $e).as_str()) }
}

lazy_static! {
	// contract address.
	static ref CONTRACT_ADDR: Address = Address::from_str("0000000000000000000000000000000000000005").unwrap();
	// secret: `sha3(1)`, and initial validator.
	static ref RICH_ADDR: Address = Address::from_str("7d577a597b2742b498cb5cf0c26cdcd726d39e6e").unwrap();
	// rich address' secret.
	static ref RICH_SECRET: Secret = secret!("1");
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
struct Transition(usize, Vec<Address>);

// create a chain with the given transitions and some blocks beyond that transition.
fn make_chain(accounts: Arc<AccountProvider>, blocks_beyond: usize, transitions: Vec<Transition>) -> Arc<Client> {
	let client = helpers::generate_dummy_client_with_spec_and_accounts(
		Spec::new_test_validator_contract, Some(accounts.clone()));

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

		let contract = ValidatorSet::new(*CONTRACT_ADDR);

		// apply all transitions.
		for Transition(num, new_set) in transitions {
			if num < TRANSITION_BLOCK {
				panic!("Bad test: issued epoch change before transition to contract.");
			}

			for number in client.chain_info().best_block_number + 1 .. num as u64 {
				push_block(&cur_signers, number, vec![]);
			}

			trace!(target: "snapshot", "applying set transition at block #{}", num);
			contract.set_validators(&exec, new_set.clone()).wait().unwrap();
			let pending: Vec<_> = client.ready_transactions()
				.into_iter()
				.map(|x| x.transaction)
				.collect();
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
fn make_transition_chain() {
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

	let client = make_chain(provider, 10, vec![
		Transition(5, vec![addrs[2], addrs[3], addrs[5], addrs[7]]),
		Transition(9, vec![addrs[0], addrs[1], addrs[4], addrs[6]]),
	]);

	assert_eq!(client.chain_info().best_block_number, 19);
	let _reader = snapshot_helpers::snap(&*client);
}
