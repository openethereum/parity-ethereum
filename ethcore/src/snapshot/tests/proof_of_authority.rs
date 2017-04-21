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
use spec::Spec;
use tests::helpers;
use transaction::{Transaction, Action};

use util::{Address, Hashable};

const PASS: &'static str = "";
const TRANSITION_BLOCK: usize = 100; // block at which the contract becomes activated.

lazy_static! {
	// contract address.
	static ref CONTRACT_ADDR: Address = Address::from_str("0x0000000000000000000000000000000000000005").unwrap();
	// secret: `sha3(0)`, and initial validator.
	static ref RICH_ADDR: Address = Address::from_str("0x7d577a597b2742b498cb5cf0c26cdcd726d39e6e").unwrap();
	// rich address' secret.
	static ref RICH_SECRET: Secret = Secret::from_slice(&"0".sha3()).unwrap();
}

// creates an account provider, filling it with accounts from all the given
// secrets and password `PASS`.
fn make_account_provider(secrets: Vec<Secret>) -> AccountProvider {
	let provider = AccountProvider::transient_provider();

	for secret in secrets {
		provider.insert_account(secret, PASS).unwrap();
	}

	provider
}

// validator transition. block number and new validators. must be after `TRANSITION_BLOCK`.
// all addresses in the set must be in the account provider.
struct Transition(usize, Vec<Address>);

// create a chain with the given transitions and some blocks beyond that transition.
fn make_chain(secrets: Vec<Secret>, transitions: Vec<Transition>, blocks_beyond: usize) -> Arc<Client> {
	let accounts = Arc::new(make_account_provider(secrets));
	let client = helpers::generate_dummy_client_with_spec_and_accounts(
		Spec::new_test_validator_contract, Some(accounts.clone()));

	let mut number = 1;
	let mut cur_signers = vec![*RICH_ADDR];

	{
		// push a block with given number, signed by one of the signers, with given transactions.
		let push_block = |signers: &[Address], n, txs| {
			use block::IsBlock;

			let engine = client.engine();
			let idx = signers.len() % n as usize;
			engine.set_signer(accounts.clone(), signers[idx], PASS.to_owned());

			let mut open_block = client.prepare_open_block(signers[idx], (5_000_000.into(), 5_000_000.into()), Vec::new());
			for tx in txs {
				open_block.push_transaction(tx, None);
			}
			let block = open_block.close_and_lock();
			let seal = match engine.generate_seal(block.block()) {
				Seal::Regular(seal) => seal,
				_ => panic!("Unable to generate seal for dummy chain."),
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
				action: Action::Call(*CONTRACT_ADDR),
				value: 0.into(),
				data: data,
			}.sign(&*RICH_SECRET, client.signing_network_id());

			client.miner().import_own_transaction(&*client, transaction.into());

			*nonce = *nonce + 1.into();
			Ok(Vec::new())
		};

		let contract = ValidatorSet::new(*CONTRACT_ADDR);

		// apply all transitions.
		for Transition(num, new_set) in transitions {
			if num < TRANSITION_BLOCK {
				panic!("Bad test: issued epoch change before transition to contract.");
			}

			while number < num - 1 {
				push_block(&cur_signers, number, vec![]);
				number += 1;
			}

			contract.set_validators(&exec, new_set.clone()).wait();
			let pending: Vec<_> = client.ready_transactions()
				.into_iter()
				.map(|x| x.transaction)
				.collect();
			push_block(&cur_signers, number, pending);

			number = num;
		}

		// make blocks beyond.
		for num in (number..).take(blocks_beyond) {
			push_block(&cur_signers, num, vec![]);
		}
	}

	client
}
