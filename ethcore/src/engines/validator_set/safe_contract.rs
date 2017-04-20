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

/// Validator set maintained in a contract, updated using `getValidators` method.

use std::sync::Weak;
use futures::Future;
use native_contracts::ValidatorSet as Provider;

use util::*;
use util::cache::MemoryLruCache;

use basic_types::LogBloom;
use client::{Client, BlockChainClient};
use engines::Call;
use header::Header;
use ids::BlockId;
use log_entry::LogEntry;

use super::ValidatorSet;
use super::simple_list::SimpleList;

const MEMOIZE_CAPACITY: usize = 500;

// TODO: ethabi should be able to generate this.
const EVENT_NAME: &'static [u8] = &*b"ValidatorsChanged(bytes32,uint256,address[])";

lazy_static! {
	static ref EVENT_NAME_HASH: H256 = EVENT_NAME.sha3();
}

/// The validator contract should have the following interface:
pub struct ValidatorSafeContract {
	pub address: Address,
	validators: RwLock<MemoryLruCache<H256, SimpleList>>,
	provider: Provider,
	client: RwLock<Option<Weak<Client>>>, // TODO [keorn]: remove
}

fn encode_proof(nonce: U256, validators: &[Address]) -> Bytes {
	use rlp::RlpStream;

	let mut stream = RlpStream::new_list(2);
	stream.append(&nonce).append_list(validators);
	stream.drain().to_vec()
}

impl ValidatorSafeContract {
	pub fn new(contract_address: Address) -> Self {
		ValidatorSafeContract {
			address: contract_address,
			validators: RwLock::new(MemoryLruCache::new(MEMOIZE_CAPACITY)),
			provider: Provider::new(contract_address),
			client: RwLock::new(None),
		}
	}

	/// Queries the state and gets the set of validators.
	fn get_list(&self, caller: &Call) -> Option<SimpleList> {
		match self.provider.get_validators(caller).wait() {
			Ok(new) => {
				debug!(target: "engine", "Set of validators obtained: {:?}", new);
				Some(SimpleList::new(new))
			},
			Err(s) => {
				debug!(target: "engine", "Set of validators could not be updated: {}", s);
				None
			},
		}
	}

	/// Queries for the current validator set transition nonce.
	fn get_nonce(&self, caller: &Call) -> Option<::util::U256> {
		match self.provider.transition_nonce(caller).wait() {
			Ok(nonce) => Some(nonce),
			Err(s) => {
				debug!(target: "engine", "Unable to fetch transition nonce: {}", s);
				None
			}
		}
	}

	// Whether the header matches the expected bloom.
	//
	// The expected log should have 3 topics:
	//   1. ETHABI-encoded log name.
	//   2. the block's parent hash.
	//   3. the "nonce": n for the nth transition in history.
	//
	// We can only search for the first 2, since we don't have the third
	// just yet.
	//
	// The parent hash is included to prevent
	// malicious actors from brute forcing other logs that would
	// produce the same bloom.
	//
	// The log data is an array of all new validator addresses.
	fn expected_bloom(&self, header: &Header) -> LogBloom {
		LogEntry {
			address: self.address,
			topics: vec![*EVENT_NAME_HASH, *header.parent_hash()],
			data: Vec::new(), // irrelevant for bloom.
		}.bloom()
	}
}

impl ValidatorSet for ValidatorSafeContract {
	fn default_caller(&self, id: BlockId) -> Box<Call> {
		let client = self.client.read().clone();
		Box::new(move |addr, data| client.as_ref()
			.and_then(Weak::upgrade)
			.ok_or("No client!".into())
			.and_then(|c| c.call_contract(id, addr, data)))
	}

	fn is_epoch_end(&self, header: &Header, _block: Option<&[u8]>, receipts: Option<&[::receipt::Receipt]>)
		-> ::engines::EpochChange
	{
		let bloom = self.expected_bloom(header);
		let header_bloom = header.log_bloom();

		if &bloom & header_bloom != bloom { return ::engines::EpochChange::No }

		match receipts {
			None => ::engines::EpochChange::Unsure(::engines::Unsure::NeedsReceipts),
			Some(receipts) => {
				let check_log = |log: &LogEntry| {
					log.address == self.address &&
						log.topics.len() == 3 &&
						log.topics[0] == *EVENT_NAME_HASH &&
						log.topics[1] == *header.parent_hash()
						// don't have anything to compare nonce to yet.
				};

				let event = Provider::contract(&self.provider)
					.event("ValidatorsChanged".into())
					.expect("Contract known ahead of time to have `ValidatorsChanged` event; qed");

				// iterate in reverse because only the _last_ change in a given
				// block actually has any effect.
				// the contract should only increment the nonce once.
				let mut decoded_events = receipts.iter()
					.rev()
					.filter(|r| &bloom & &r.log_bloom == bloom)
					.flat_map(|r| r.logs.iter())
					.filter(move |l| check_log(l))
					.filter_map(|log| {
						let topics = log.topics.iter().map(|x| x.0.clone()).collect();
						match event.decode_log(topics, log.data.clone()) {
							Ok(decoded) => Some(decoded),
							Err(_) => None,
						}
					});

				match decoded_events.next() {
					None => ::engines::EpochChange::No,
					Some(matched_event) => {
						// decode log manually until the native contract generator is
						// good enough to do it for us.
						let &(_, _, ref nonce_token) = &matched_event.params[2];
						let &(_, _, ref validators_token) = &matched_event.params[3];

						let nonce: Option<U256> = nonce_token.clone().to_uint()
							.map(H256).map(Into::into);
						let validators = validators_token.clone().to_array()
							.and_then(|a| a.into_iter()
								.map(|x| x.to_address().map(H160))
								.collect::<Option<Vec<_>>>()
							);

						match (nonce, validators) {
							(Some(nonce), Some(validators)) => {
								let proof = encode_proof(nonce, &validators);
								let new_epoch = nonce.low_u64();
								::engines::EpochChange::Yes(new_epoch, proof)
							}
							_ => {
								debug!(target: "engine", "Successfully decoded log turned out to be bad.");
								::engines::EpochChange::No
							}
						}
					}
				}
			}
		}
	}

	// the proof we generate is an RLP list containing two parts.
	//   (nonce, validators)
	fn epoch_proof(&self, _header: &Header, caller: &Call) -> Result<Vec<u8>, String> {
		match (self.get_nonce(caller), self.get_list(caller)) {
			(Some(nonce), Some(list)) => Ok(encode_proof(nonce, &list.into_inner())),
			_ => Err("Caller insufficient to generate validator proof.".into()),
		}
	}

	fn epoch_set(&self, _header: &Header, proof: &[u8]) -> Result<(u64, SimpleList), ::error::Error> {
		use rlp::UntrustedRlp;

		let rlp = UntrustedRlp::new(proof);
		let nonce: u64 = rlp.val_at(0)?;
		let validators: Vec<Address> = rlp.list_at(1)?;

		Ok((nonce, SimpleList::new(validators)))
	}

	fn contains_with_caller(&self, block_hash: &H256, address: &Address, caller: &Call) -> bool {
		let mut guard = self.validators.write();
		let maybe_existing = guard
			.get_mut(block_hash)
			.map(|list| list.contains(block_hash, address));
		maybe_existing
			.unwrap_or_else(|| self
				.get_list(caller)
				.map_or(false, |list| {
					let contains = list.contains(block_hash, address);
					guard.insert(block_hash.clone(), list);
					contains
				 }))
	}

	fn get_with_caller(&self, block_hash: &H256, nonce: usize, caller: &Call) -> Address {
		let mut guard = self.validators.write();
		let maybe_existing = guard
			.get_mut(block_hash)
			.map(|list| list.get(block_hash, nonce));
		maybe_existing
			.unwrap_or_else(|| self
				.get_list(caller)
				.map_or_else(Default::default, |list| {
					let address = list.get(block_hash, nonce);
					guard.insert(block_hash.clone(), list);
					address
				 }))
	}

	fn count_with_caller(&self, block_hash: &H256, caller: &Call) -> usize {
		let mut guard = self.validators.write();
		let maybe_existing = guard
			.get_mut(block_hash)
			.map(|list| list.count(block_hash));
		maybe_existing
			.unwrap_or_else(|| self
				.get_list(caller)
				.map_or_else(usize::max_value, |list| {
					let address = list.count(block_hash);
					guard.insert(block_hash.clone(), list);
					address
				 }))
	}

	fn register_contract(&self, client: Weak<Client>) {
		trace!(target: "engine", "Setting up contract caller.");
		*self.client.write() = Some(client);
	}
}

#[cfg(test)]
mod tests {
	use util::*;
	use types::ids::BlockId;
	use spec::Spec;
	use account_provider::AccountProvider;
	use transaction::{Transaction, Action};
	use client::{BlockChainClient, EngineClient};
	use ethkey::Secret;
	use miner::MinerService;
	use tests::helpers::{generate_dummy_client_with_spec_and_accounts, generate_dummy_client_with_spec_and_data};
	use super::super::ValidatorSet;
	use super::{ValidatorSafeContract, EVENT_NAME_HASH};

	#[test]
	fn fetches_validators() {
		let client = generate_dummy_client_with_spec_and_accounts(Spec::new_validator_safe_contract, None);
		let vc = Arc::new(ValidatorSafeContract::new(Address::from_str("0000000000000000000000000000000000000005").unwrap()));
		vc.register_contract(Arc::downgrade(&client));
		let last_hash = client.best_block_header().hash();
		assert!(vc.contains(&last_hash, &Address::from_str("7d577a597b2742b498cb5cf0c26cdcd726d39e6e").unwrap()));
		assert!(vc.contains(&last_hash, &Address::from_str("82a978b3f5962a5b0957d9ee9eef472ee55b42f1").unwrap()));
	}

	#[test]
	fn knows_validators() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let s0 = Secret::from_slice(&"1".sha3()).unwrap();
		let v0 = tap.insert_account(s0.clone(), "").unwrap();
		let v1 = tap.insert_account(Secret::from_slice(&"0".sha3()).unwrap(), "").unwrap();
		let network_id = Spec::new_validator_safe_contract().network_id();
		let client = generate_dummy_client_with_spec_and_accounts(Spec::new_validator_safe_contract, Some(tap));
		client.engine().register_client(Arc::downgrade(&client));
		let validator_contract = Address::from_str("0000000000000000000000000000000000000005").unwrap();

		client.miner().set_engine_signer(v1, "".into()).unwrap();
		// Remove "1" validator.
		let tx = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 500_000.into(),
			action: Action::Call(validator_contract),
			value: 0.into(),
			data: "bfc708a000000000000000000000000082a978b3f5962a5b0957d9ee9eef472ee55b42f1".from_hex().unwrap(),
		}.sign(&s0, Some(network_id));
		client.miner().import_own_transaction(client.as_ref(), tx.into()).unwrap();
		client.update_sealing();
		assert_eq!(client.chain_info().best_block_number, 1);
		// Add "1" validator back in.
		let tx = Transaction {
			nonce: 1.into(),
			gas_price: 0.into(),
			gas: 500_000.into(),
			action: Action::Call(validator_contract),
			value: 0.into(),
			data: "4d238c8e00000000000000000000000082a978b3f5962a5b0957d9ee9eef472ee55b42f1".from_hex().unwrap(),
		}.sign(&s0, Some(network_id));
		client.miner().import_own_transaction(client.as_ref(), tx.into()).unwrap();
		client.update_sealing();
		// The transaction is not yet included so still unable to seal.
		assert_eq!(client.chain_info().best_block_number, 1);

		// Switch to the validator that is still there.
		client.miner().set_engine_signer(v0, "".into()).unwrap();
		client.update_sealing();
		assert_eq!(client.chain_info().best_block_number, 2);
		// Switch back to the added validator, since the state is updated.
		client.miner().set_engine_signer(v1, "".into()).unwrap();
		let tx = Transaction {
			nonce: 2.into(),
			gas_price: 0.into(),
			gas: 21000.into(),
			action: Action::Call(Address::default()),
			value: 0.into(),
			data: Vec::new(),
		}.sign(&s0, Some(network_id));
		client.miner().import_own_transaction(client.as_ref(), tx.into()).unwrap();
		client.update_sealing();
		// Able to seal again.
		assert_eq!(client.chain_info().best_block_number, 3);

		// Check syncing.
		let sync_client = generate_dummy_client_with_spec_and_data(Spec::new_validator_safe_contract, 0, 0, &[]);
		sync_client.engine().register_client(Arc::downgrade(&sync_client));
		for i in 1..4 {
			sync_client.import_block(client.block(BlockId::Number(i)).unwrap().into_inner()).unwrap();
		}
		sync_client.flush_queue();
		assert_eq!(sync_client.chain_info().best_block_number, 3);
	}

	#[test]
	fn detects_bloom() {
		use header::Header;
		use engines::{EpochChange, Unsure};
		use log_entry::LogEntry;

		let client = generate_dummy_client_with_spec_and_accounts(Spec::new_validator_safe_contract, None);
		let engine = client.engine().clone();
		let validator_contract = Address::from_str("0000000000000000000000000000000000000005").unwrap();

		let last_hash = client.best_block_header().hash();
		let mut new_header = Header::default();
		new_header.set_parent_hash(last_hash);

		// first, try without the parent hash.
		let mut event = LogEntry {
			address: validator_contract,
			topics: vec![*EVENT_NAME_HASH],
			data: Vec::new(),
		};

		new_header.set_log_bloom(event.bloom());
		assert_eq!(engine.is_epoch_end(&new_header, None, None), EpochChange::No);

		// with the last hash, it should need the receipts.
		event.topics.push(last_hash);
		new_header.set_log_bloom(event.bloom());
		assert_eq!(engine.is_epoch_end(&new_header, None, None),
			EpochChange::Unsure(Unsure::NeedsReceipts));
	}
}
