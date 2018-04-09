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

use std::sync::{Weak, Arc};
use hash::keccak;

use ethereum_types::{H256, U256, Address, Bloom};
use parking_lot::RwLock;

use bytes::Bytes;
use memory_cache::MemoryLruCache;
use unexpected::Mismatch;
use rlp::{UntrustedRlp, RlpStream};
use kvdb::DBValue;

use client::EngineClient;
use machine::{AuxiliaryData, Call, EthereumMachine, AuxiliaryRequest};
use header::Header;
use ids::BlockId;
use log_entry::LogEntry;
use receipt::Receipt;

use super::{SystemCall, ValidatorSet};
use super::simple_list::SimpleList;

use_contract!(validator_set, "ValidatorSet", "res/contracts/validator_set.json");

const MEMOIZE_CAPACITY: usize = 500;

// TODO: ethabi should be able to generate this.
const EVENT_NAME: &'static [u8] = &*b"InitiateChange(bytes32,address[])";

lazy_static! {
	static ref EVENT_NAME_HASH: H256 = keccak(EVENT_NAME);
}

// state-dependent proofs for the safe contract:
// only "first" proofs are such.
struct StateProof {
	contract_address: Address,
	header: Header,
	provider: validator_set::ValidatorSet,
}

impl ::engines::StateDependentProof<EthereumMachine> for StateProof {
	fn generate_proof(&self, caller: &Call) -> Result<Vec<u8>, String> {
		prove_initial(&self.provider, self.contract_address, &self.header, caller)
	}

	fn check_proof(&self, machine: &EthereumMachine, proof: &[u8]) -> Result<(), String> {
		let (header, state_items) = decode_first_proof(&UntrustedRlp::new(proof))
			.map_err(|e| format!("proof incorrectly encoded: {}", e))?;
		if &header != &self.header {
			return Err("wrong header in proof".into());
		}

		check_first_proof(machine, &self.provider, self.contract_address, header, &state_items).map(|_| ())
	}
}

/// The validator contract should have the following interface:
pub struct ValidatorSafeContract {
	contract_address: Address,
	validators: RwLock<MemoryLruCache<H256, SimpleList>>,
	provider: validator_set::ValidatorSet,
	client: RwLock<Option<Weak<EngineClient>>>, // TODO [keorn]: remove
}

// first proof is just a state proof call of `getValidators` at header's state.
fn encode_first_proof(header: &Header, state_items: &[Vec<u8>]) -> Bytes {
	let mut stream = RlpStream::new_list(2);
	stream.append(header).begin_list(state_items.len());
	for item in state_items {
		stream.append(item);
	}

	stream.out()
}

// check a first proof: fetch the validator set at the given block.
fn check_first_proof(machine: &EthereumMachine, provider: &validator_set::ValidatorSet, contract_address: Address, old_header: Header, state_items: &[DBValue])
	-> Result<Vec<Address>, String>
{
	use transaction::{Action, Transaction};

	// TODO: match client contract_call_tx more cleanly without duplication.
	const PROVIDED_GAS: u64 = 50_000_000;

	let env_info = ::vm::EnvInfo {
		number: old_header.number(),
		author: *old_header.author(),
		difficulty: *old_header.difficulty(),
		gas_limit: PROVIDED_GAS.into(),
		timestamp: old_header.timestamp(),
		last_hashes: {
			// this will break if we don't inclue all 256 last hashes.
			let mut last_hashes: Vec<_> = (0..256).map(|_| H256::default()).collect();
			last_hashes[255] = *old_header.parent_hash();
			Arc::new(last_hashes)
		},
		gas_used: 0.into(),
	};

	// check state proof using given machine.
	let number = old_header.number();
	provider.functions().get_validators().call(&|data| {
		let from = Address::default();
		let tx = Transaction {
			nonce: machine.account_start_nonce(number),
			action: Action::Call(contract_address),
			gas: PROVIDED_GAS.into(),
			gas_price: U256::default(),
			value: U256::default(),
			data,
		}.fake_sign(from);

		let res = ::state::check_proof(
			state_items,
			*old_header.state_root(),
			&tx,
			machine,
			&env_info,
		);

		match res {
			::state::ProvedExecution::BadProof => Err("Bad proof".into()),
			::state::ProvedExecution::Failed(e) => Err(format!("Failed call: {}", e)),
			::state::ProvedExecution::Complete(e) => Ok(e.output),
		}
	}).map_err(|err| err.to_string())
}

fn decode_first_proof(rlp: &UntrustedRlp) -> Result<(Header, Vec<DBValue>), ::error::Error> {
	let header = rlp.val_at(0)?;
	let state_items = rlp.at(1)?.iter().map(|x| {
		let mut val = DBValue::new();
		val.append_slice(x.data()?);
		Ok(val)
	}).collect::<Result<_, ::error::Error>>()?;

	Ok((header, state_items))
}

// inter-contract proofs are a header and receipts.
// checking will involve ensuring that the receipts match the header and
// extracting the validator set from the receipts.
fn encode_proof(header: &Header, receipts: &[Receipt]) -> Bytes {
	let mut stream = RlpStream::new_list(2);
	stream.append(header).append_list(receipts);
	stream.drain().into_vec()
}

fn decode_proof(rlp: &UntrustedRlp) -> Result<(Header, Vec<Receipt>), ::error::Error> {
	Ok((rlp.val_at(0)?, rlp.list_at(1)?))
}

// given a provider and caller, generate proof. this will just be a state proof
// of `getValidators`.
fn prove_initial(provider: &validator_set::ValidatorSet, contract_address: Address, header: &Header, caller: &Call) -> Result<Vec<u8>, String> {
	use std::cell::RefCell;

	let epoch_proof = RefCell::new(None);
	let res = {
		let caller = |data| {
			let (result, proof) = caller(contract_address, data)?;
			*epoch_proof.borrow_mut() = Some(encode_first_proof(header, &proof));
			Ok(result)
		};

		provider.functions().get_validators().call(&caller)
			.map_err(|err| err.to_string())
	};

	res.map(|validators| {
		let proof = epoch_proof.into_inner().expect("epoch_proof always set after call; qed");

		trace!(target: "engine", "obtained proof for initial set: {} validators, {} bytes",
			validators.len(), proof.len());

		info!(target: "engine", "Signal for switch to contract-based validator set.");
		info!(target: "engine", "Initial contract validators: {:?}", validators);

		proof
	})
}

impl ValidatorSafeContract {
	pub fn new(contract_address: Address) -> Self {
		ValidatorSafeContract {
			contract_address,
			validators: RwLock::new(MemoryLruCache::new(MEMOIZE_CAPACITY)),
			provider: validator_set::ValidatorSet::default(),
			client: RwLock::new(None),
		}
	}

	/// Queries the state and gets the set of validators.
	fn get_list(&self, caller: &Call) -> Option<SimpleList> {
		let contract_address = self.contract_address;
		let caller = move |data| caller(contract_address, data).map(|x| x.0);
		match self.provider.functions().get_validators().call(&caller) {
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
	fn expected_bloom(&self, header: &Header) -> Bloom {
		let topics = vec![*EVENT_NAME_HASH, *header.parent_hash()];

		debug!(target: "engine", "Expected topics for header {}: {:?}",
			header.hash(), topics);

		LogEntry {
			address: self.contract_address,
			topics: topics,
			data: Vec::new(), // irrelevant for bloom.
		}.bloom()
	}

	// check receipts for log event. bloom should be `expected_bloom` for the
	// header the receipts correspond to.
	fn extract_from_event(&self, bloom: Bloom, header: &Header, receipts: &[Receipt]) -> Option<SimpleList> {
		let check_log = |log: &LogEntry| {
			log.address == self.contract_address &&
				log.topics.len() == 2 &&
				log.topics[0] == *EVENT_NAME_HASH &&
				log.topics[1] == *header.parent_hash()
		};

		let event = self.provider.events().initiate_change();
		//// iterate in reverse because only the _last_ change in a given
		//// block actually has any effect.
		//// the contract should only increment the nonce once.
		let mut decoded_events = receipts.iter()
			.rev()
			.filter(|r| r.log_bloom.contains_bloom(&bloom))
			.flat_map(|r| r.logs.iter())
			.filter(move |l| check_log(l))
			.filter_map(|log| {
				event.parse_log((log.topics.clone(), log.data.clone()).into()).ok()
			});

		// only last log is taken into account
		match decoded_events.next() {
			None => None,
			Some(matched_event) => Some(SimpleList::new(matched_event.new_set))
		}
	}
}

impl ValidatorSet for ValidatorSafeContract {
	fn default_caller(&self, id: BlockId) -> Box<Call> {
		let client = self.client.read().clone();
		Box::new(move |addr, data| client.as_ref()
			.and_then(Weak::upgrade)
			.ok_or_else(|| "No client!".into())
			.and_then(|c| {
				match c.as_full_client() {
					Some(c) => c.call_contract(id, addr, data),
					None => Err("No full client!".into()),
				}
			})
			.map(|out| (out, Vec::new()))) // generate no proofs in general
	}

	fn on_epoch_begin(&self, _first: bool, _header: &Header, caller: &mut SystemCall) -> Result<(), ::error::Error> {
		let data = self.provider.functions().finalize_change().input();
		caller(self.contract_address, data)
			.map(|_| ())
			.map_err(::engines::EngineError::FailedSystemCall)
			.map_err(Into::into)
	}

	fn genesis_epoch_data(&self, header: &Header, call: &Call) -> Result<Vec<u8>, String> {
		prove_initial(&self.provider, self.contract_address, header, call)
	}

	fn is_epoch_end(&self, _first: bool, _chain_head: &Header) -> Option<Vec<u8>> {
		None // no immediate transitions to contract.
	}

	fn signals_epoch_end(&self, first: bool, header: &Header, aux: AuxiliaryData)
		-> ::engines::EpochChange<EthereumMachine>
	{
		let receipts = aux.receipts;

		// transition to the first block of a contract requires finality but has no log event.
		if first {
			debug!(target: "engine", "signalling transition to fresh contract.");
			let state_proof = Arc::new(StateProof {
				contract_address: self.contract_address,
				header: header.clone(),
				provider: validator_set::ValidatorSet::default(),
			});
			return ::engines::EpochChange::Yes(::engines::Proof::WithState(state_proof as Arc<_>));
		}

		// otherwise, we're checking for logs.
		let bloom = self.expected_bloom(header);
		let header_bloom = header.log_bloom();

		if &bloom & header_bloom != bloom { return ::engines::EpochChange::No }

		trace!(target: "engine", "detected epoch change event bloom");

		match receipts {
			None => ::engines::EpochChange::Unsure(AuxiliaryRequest::Receipts),
			Some(receipts) => match self.extract_from_event(bloom, header, receipts) {
				None => ::engines::EpochChange::No,
				Some(list) => {
					info!(target: "engine", "Signal for transition within contract. New list: {:?}",
						&*list);

					let proof = encode_proof(&header, receipts);
					::engines::EpochChange::Yes(::engines::Proof::Known(proof))
				}
			},
		}
	}

	fn epoch_set(&self, first: bool, machine: &EthereumMachine, _number: ::header::BlockNumber, proof: &[u8])
		-> Result<(SimpleList, Option<H256>), ::error::Error>
	{
		let rlp = UntrustedRlp::new(proof);

		if first {
			trace!(target: "engine", "Recovering initial epoch set");

			let (old_header, state_items) = decode_first_proof(&rlp)?;
			let number = old_header.number();
			let old_hash = old_header.hash();
			let addresses = check_first_proof(machine, &self.provider, self.contract_address, old_header, &state_items)
				.map_err(::engines::EngineError::InsufficientProof)?;

			trace!(target: "engine", "extracted epoch set at #{}: {} addresses",
				number, addresses.len());

			Ok((SimpleList::new(addresses), Some(old_hash)))
		} else {
			let (old_header, receipts) = decode_proof(&rlp)?;

			// ensure receipts match header.
			// TODO: optimize? these were just decoded.
			let found_root = ::triehash::ordered_trie_root(
				receipts.iter().map(::rlp::encode)
			);
			if found_root != *old_header.receipts_root() {
				return Err(::error::BlockError::InvalidReceiptsRoot(
					Mismatch { expected: *old_header.receipts_root(), found: found_root }
				).into());
			}

			let bloom = self.expected_bloom(&old_header);

			match self.extract_from_event(bloom, &old_header, &receipts) {
				Some(list) => Ok((list, Some(old_header.hash()))),
				None => Err(::engines::EngineError::InsufficientProof("No log event in proof.".into()).into()),
			}
		}
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

	fn register_client(&self, client: Weak<EngineClient>) {
		trace!(target: "engine", "Setting up contract caller.");
		*self.client.write() = Some(client);
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use rustc_hex::FromHex;
	use hash::keccak;
	use ethereum_types::Address;
	use types::ids::BlockId;
	use spec::Spec;
	use account_provider::AccountProvider;
	use transaction::{Transaction, Action};
	use client::{ChainInfo, BlockInfo, ImportBlock};
	use ethkey::Secret;
	use miner::MinerService;
	use test_helpers::{generate_dummy_client_with_spec_and_accounts, generate_dummy_client_with_spec_and_data};
	use super::super::ValidatorSet;
	use super::{ValidatorSafeContract, EVENT_NAME_HASH};

	#[test]
	fn fetches_validators() {
		let client = generate_dummy_client_with_spec_and_accounts(Spec::new_validator_safe_contract, None);
		let vc = Arc::new(ValidatorSafeContract::new("0000000000000000000000000000000000000005".parse::<Address>().unwrap()));
		vc.register_client(Arc::downgrade(&client) as _);
		let last_hash = client.best_block_header().hash();
		assert!(vc.contains(&last_hash, &"7d577a597b2742b498cb5cf0c26cdcd726d39e6e".parse::<Address>().unwrap()));
		assert!(vc.contains(&last_hash, &"82a978b3f5962a5b0957d9ee9eef472ee55b42f1".parse::<Address>().unwrap()));
	}

	#[test]
	fn knows_validators() {
		let tap = Arc::new(AccountProvider::transient_provider());
		let s0: Secret = keccak("1").into();
		let v0 = tap.insert_account(s0.clone(), "").unwrap();
		let v1 = tap.insert_account(keccak("0").into(), "").unwrap();
		let chain_id = Spec::new_validator_safe_contract().chain_id();
		let client = generate_dummy_client_with_spec_and_accounts(Spec::new_validator_safe_contract, Some(tap));
		client.engine().register_client(Arc::downgrade(&client) as _);
		let validator_contract = "0000000000000000000000000000000000000005".parse::<Address>().unwrap();

		client.miner().set_engine_signer(v1, "".into()).unwrap();
		// Remove "1" validator.
		let tx = Transaction {
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 500_000.into(),
			action: Action::Call(validator_contract),
			value: 0.into(),
			data: "bfc708a000000000000000000000000082a978b3f5962a5b0957d9ee9eef472ee55b42f1".from_hex().unwrap(),
		}.sign(&s0, Some(chain_id));
		client.miner().import_own_transaction(client.as_ref(), tx.into()).unwrap();
		::client::EngineClient::update_sealing(&*client);
		assert_eq!(client.chain_info().best_block_number, 1);
		// Add "1" validator back in.
		let tx = Transaction {
			nonce: 1.into(),
			gas_price: 0.into(),
			gas: 500_000.into(),
			action: Action::Call(validator_contract),
			value: 0.into(),
			data: "4d238c8e00000000000000000000000082a978b3f5962a5b0957d9ee9eef472ee55b42f1".from_hex().unwrap(),
		}.sign(&s0, Some(chain_id));
		client.miner().import_own_transaction(client.as_ref(), tx.into()).unwrap();
		::client::EngineClient::update_sealing(&*client);
		// The transaction is not yet included so still unable to seal.
		assert_eq!(client.chain_info().best_block_number, 1);

		// Switch to the validator that is still there.
		client.miner().set_engine_signer(v0, "".into()).unwrap();
		::client::EngineClient::update_sealing(&*client);
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
		}.sign(&s0, Some(chain_id));
		client.miner().import_own_transaction(client.as_ref(), tx.into()).unwrap();
		::client::EngineClient::update_sealing(&*client);
		// Able to seal again.
		assert_eq!(client.chain_info().best_block_number, 3);

		// Check syncing.
		let sync_client = generate_dummy_client_with_spec_and_data(Spec::new_validator_safe_contract, 0, 0, &[]);
		sync_client.engine().register_client(Arc::downgrade(&sync_client) as _);
		for i in 1..4 {
			sync_client.import_block(client.block(BlockId::Number(i)).unwrap().into_inner()).unwrap();
		}
		sync_client.flush_queue();
		assert_eq!(sync_client.chain_info().best_block_number, 3);
	}

	#[test]
	fn detects_bloom() {
		use header::Header;
		use engines::EpochChange;
		use machine::AuxiliaryRequest;
		use log_entry::LogEntry;

		let client = generate_dummy_client_with_spec_and_accounts(Spec::new_validator_safe_contract, None);
		let engine = client.engine().clone();
		let validator_contract = "0000000000000000000000000000000000000005".parse::<Address>().unwrap();

		let last_hash = client.best_block_header().hash();
		let mut new_header = Header::default();
		new_header.set_parent_hash(last_hash);
		new_header.set_number(1); // so the validator set looks for a log.

		// first, try without the parent hash.
		let mut event = LogEntry {
			address: validator_contract,
			topics: vec![*EVENT_NAME_HASH],
			data: Vec::new(),
		};

		new_header.set_log_bloom(event.bloom());
		match engine.signals_epoch_end(&new_header, Default::default()) {
			EpochChange::No => {},
			_ => panic!("Expected bloom to be unrecognized."),
		};

		// with the last hash, it should need the receipts.
		event.topics.push(last_hash);
		new_header.set_log_bloom(event.bloom());

		match engine.signals_epoch_end(&new_header, Default::default()) {
			EpochChange::Unsure(AuxiliaryRequest::Receipts) => {},
			_ => panic!("Expected bloom to be recognized."),
		};
	}

	#[test]
	fn initial_contract_is_signal() {
		use header::Header;
		use engines::{EpochChange, Proof};

		let client = generate_dummy_client_with_spec_and_accounts(Spec::new_validator_safe_contract, None);
		let engine = client.engine().clone();

		let mut new_header = Header::default();
		new_header.set_number(0); // so the validator set doesn't look for a log

		match engine.signals_epoch_end(&new_header, Default::default()) {
			EpochChange::Yes(Proof::WithState(_)) => {},
			_ => panic!("Expected state to be required to prove initial signal"),
		};
	}
}
