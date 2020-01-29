// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

/// Validator set maintained in a contract, updated using `getValidators` method.

use std::collections::VecDeque;
use std::sync::{Arc, Weak};

use client_traits::{BlockChainClient, EngineClient, TransactionRequest};
use common_types::{
	BlockNumber,
	header::Header,
	errors::{EngineError, EthcoreError, BlockError},
	ids::BlockId,
	log_entry::LogEntry,
	engines::machine::{Call, AuxiliaryData, AuxiliaryRequest},
	receipt::Receipt,
	transaction::{self, Action, Transaction},
};
use ethabi::FunctionOutputDecoder;
use ethabi_contract::use_contract;
use ethereum_types::{H256, U256, Address, Bloom};
use keccak_hash::keccak;
use kvdb::DBValue;
use lazy_static::lazy_static;
use log::{debug, info, trace, warn};
use machine::Machine;
use memory_cache::MemoryLruCache;
use parity_bytes::Bytes;
use parking_lot::{Mutex, RwLock};
use rlp::{Rlp, RlpStream};
use unexpected::Mismatch;

use super::{SystemCall, ValidatorSet};
use super::simple_list::SimpleList;

use_contract!(validator_set, "res/validator_set.json");

/// The maximum number of reports to keep queued.
const MAX_QUEUED_REPORTS: usize = 10;
/// The maximum number of malice reports to include when creating a new block.
const MAX_REPORTS_PER_BLOCK: usize = 10;
/// Don't re-send malice reports every block. Skip this many before retrying.
const REPORTS_SKIP_BLOCKS: u64 = 1;

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
}

impl engine::StateDependentProof for StateProof {
	fn generate_proof(&self, caller: &Call) -> Result<Vec<u8>, String> {
		prove_initial(self.contract_address, &self.header, caller)
	}

	fn check_proof(&self, machine: &Machine, proof: &[u8]) -> Result<(), String> {
		let (header, state_items) = decode_first_proof(&Rlp::new(proof))
			.map_err(|e| format!("proof incorrectly encoded: {}", e))?;
		if &header != &self.header {
			return Err("wrong header in proof".into());
		}

		check_first_proof(machine, self.contract_address, header, &state_items).map(|_| ())
	}
}

/// The validator contract should have the following interface:
pub struct ValidatorSafeContract {
	contract_address: Address,
	/// The LRU cache is indexed by the parent_hash, so given a hash, the value
	/// is the validator set valid for the blocks following that hash.
	validators: RwLock<MemoryLruCache<H256, SimpleList>>,
	client: RwLock<Option<Weak<dyn EngineClient>>>, // TODO [keorn]: remove
	report_queue: Mutex<ReportQueue>,
	/// The block number where we resent the queued reports last time.
	resent_reports_in_block: Mutex<BlockNumber>,
	/// If set, this is the block number at which the consensus engine switches from AuRa to AuRa
	/// with POSDAO modifications.
	posdao_transition: Option<BlockNumber>,
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
fn check_first_proof(machine: &Machine, contract_address: Address, old_header: Header, state_items: &[DBValue])
	-> Result<Vec<Address>, String>
{
	// TODO: match client contract_call_tx more cleanly without duplication.
	const PROVIDED_GAS: u64 = 50_000_000;

	let env_info = vm::EnvInfo {
		number: old_header.number(),
		author: *old_header.author(),
		difficulty: *old_header.difficulty(),
		gas_limit: PROVIDED_GAS.into(),
		timestamp: old_header.timestamp(),
		last_hashes: {
			// this will break if we don't include all 256 last hashes.
			let mut last_hashes: Vec<_> = (0..256).map(|_| H256::zero()).collect();
			last_hashes[255] = *old_header.parent_hash();
			Arc::new(last_hashes)
		},
		gas_used: 0.into(),
	};

	// check state proof using given machine.
	let number = old_header.number();
	let (data, decoder) = validator_set::functions::get_validators::call();

	let from = Address::zero();
	let tx = Transaction {
		nonce: machine.account_start_nonce(number),
		action: Action::Call(contract_address),
		gas: PROVIDED_GAS.into(),
		gas_price: U256::default(),
		value: U256::default(),
		data,
	}.fake_sign(from);

	let res = executive_state::check_proof(
		state_items,
		*old_header.state_root(),
		&tx,
		machine,
		&env_info,
	);

	match res {
		executive_state::ProvedExecution::BadProof => Err("Bad proof".into()),
		executive_state::ProvedExecution::Failed(e) => Err(format!("Failed call: {}", e)),
		executive_state::ProvedExecution::Complete(e) => decoder.decode(&e.output).map_err(|e| e.to_string()),
	}
}

fn decode_first_proof(rlp: &Rlp) -> Result<(Header, Vec<DBValue>), EthcoreError> {
	let header = rlp.val_at(0)?;
	let state_items = rlp.at(1)?
		.iter()
		.map(|x| Ok(x.data()?.to_vec()) )
		.collect::<Result<_, EthcoreError>>()?;

	Ok((header, state_items))
}

// inter-contract proofs are a header and receipts.
// checking will involve ensuring that the receipts match the header and
// extracting the validator set from the receipts.
fn encode_proof(header: &Header, receipts: &[Receipt]) -> Bytes {
	let mut stream = RlpStream::new_list(2);
	stream.append(header).append_list(receipts);
	stream.drain()
}

fn decode_proof(rlp: &Rlp) -> Result<(Header, Vec<Receipt>), EthcoreError> {
	Ok((rlp.val_at(0)?, rlp.list_at(1)?))
}

// given a provider and caller, generate proof. this will just be a state proof
// of `getValidators`.
fn prove_initial(contract_address: Address, header: &Header, caller: &Call) -> Result<Vec<u8>, String> {
	use std::cell::RefCell;

	let epoch_proof = RefCell::new(None);
	let validators = {
		let (data, decoder) = validator_set::functions::get_validators::call();
		let (value, proof) = caller(contract_address, data)?;
		*epoch_proof.borrow_mut() = Some(encode_first_proof(header, &proof));
		decoder.decode(&value).map_err(|e| e.to_string())?
	};

	let proof = epoch_proof.into_inner().expect("epoch_proof always set after call; qed");

	trace!(target: "engine", "obtained proof for initial set: {} validators, {} bytes",
		validators.len(), proof.len());

	info!(target: "engine", "Signal for switch to contract-based validator set.");
	info!(target: "engine", "Initial contract validators: {:?}", validators);

	Ok(proof)
}

impl ValidatorSafeContract {
	pub fn new(contract_address: Address, posdao_transition: Option<BlockNumber>) -> Self {
		ValidatorSafeContract {
			contract_address,
			validators: RwLock::new(MemoryLruCache::new(MEMOIZE_CAPACITY)),
			client: RwLock::new(None),
			report_queue: Mutex::new(ReportQueue::default()),
			resent_reports_in_block: Mutex::new(0),
			posdao_transition,
		}
	}

	fn transact(&self, data: Bytes, nonce: U256) -> Result<(), EthcoreError> {
		let client = self.client.read().as_ref().and_then(Weak::upgrade).ok_or("No client!")?;
		let full_client = client.as_full_client().ok_or("No full client!")?;

		let tx_request = TransactionRequest::call(self.contract_address, data).gas_price(U256::zero()).nonce(nonce);
		match full_client.transact(tx_request) {
			Ok(()) | Err(transaction::Error::AlreadyImported) => Ok(()),
			Err(e) => Err(e)?,
		}
	}

	/// Puts a malice report into the queue for later resending.
	///
	/// # Arguments
	///
	/// * `addr` - The address of the misbehaving validator.
	/// * `block` - The block number at which the misbehavior occurred.
	/// * `data` - The call data for the `reportMalicious` contract call.
	pub(crate) fn enqueue_report(&self, addr: Address, block: BlockNumber, data: Vec<u8>) {
		// Skip the rest of the function unless there has been a transition to POSDAO AuRa.
		if self.posdao_transition.map_or(true, |block_num| block < block_num) {
			trace!(target: "engine", "Skipping queueing a malicious behavior report");
			return;
		}
		self.report_queue.lock().push(addr, block, data)
	}

	/// Queries the state and gets the set of validators.
	fn get_list(&self, caller: &Call) -> Option<SimpleList> {
		let contract_address = self.contract_address;

		let (data, decoder) = validator_set::functions::get_validators::call();
		let value = caller(contract_address, data).and_then(|x| decoder.decode(&x.0).map_err(|e| e.to_string()));

		match value {
			Ok(new) => {
				debug!(target: "engine", "Got validator set from contract: {:?}", new);
				Some(SimpleList::new(new))
			},
			Err(s) => {
				debug!(target: "engine", "Could not get validator set from contract: {}", s);
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
			topics,
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

		//// iterate in reverse because only the _last_ change in a given
		//// block actually has any effect.
		//// the contract should only increment the nonce once.
		let mut decoded_events = receipts.iter()
			.rev()
			.filter(|r| r.log_bloom.contains_bloom(&bloom))
			.flat_map(|r| r.logs.iter())
			.filter(move |l| check_log(l))
			.filter_map(|log| {
				validator_set::events::initiate_change::parse_log((log.topics.clone(), log.data.clone()).into()).ok()
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

	fn generate_engine_transactions(&self, _first: bool, header: &Header, caller: &mut SystemCall)
		-> Result<Vec<(Address, Bytes)>, EthcoreError>
	{
		// Skip the rest of the function unless there has been a transition to POSDAO AuRa.
		if self.posdao_transition.map_or(true, |block_num| header.number() < block_num) {
			trace!(target: "engine", "Skipping a call to emitInitiateChange");
			return Ok(Vec::new());
		}
		let mut transactions = Vec::new();

		// Create the `InitiateChange` event if necessary.
		let (data, decoder) = validator_set::functions::emit_initiate_change_callable::call();
		let emit_initiate_change_callable = caller(self.contract_address, data)
			.and_then(|x| decoder.decode(&x)
			.map_err(|x| format!("chain spec bug: could not decode: {:?}", x)))
			.map_err(EngineError::FailedSystemCall)?;
		if !emit_initiate_change_callable {
			trace!(target: "engine", "New block #{} issued ― no need to call emitInitiateChange()", header.number());
		} else {
			trace!(target: "engine", "New block issued #{} ― calling emitInitiateChange()", header.number());
			let (data, _decoder) = validator_set::functions::emit_initiate_change::call();
			transactions.push((self.contract_address, data));
		}

		let client = self.client.read().as_ref().and_then(Weak::upgrade).ok_or("No client!")?;
		let client = client.as_full_client().ok_or("No full client!")?;

		// Retry all pending reports.
		let mut report_queue = self.report_queue.lock();
		report_queue.filter(client, header.author(), self.contract_address);
		for (_address, _block, data) in report_queue.iter().take(MAX_REPORTS_PER_BLOCK) {
			transactions.push((self.contract_address, data.clone()))
		}

		Ok(transactions)
	}

	fn on_close_block(&self, header: &Header, our_address: &Address) -> Result<(), EthcoreError> {
		// Skip the rest of the function unless there has been a transition to POSDAO AuRa.
		if self.posdao_transition.map_or(true, |block_num| header.number() < block_num) {
			trace!(target: "engine", "Skipping resending of queued malicious behavior reports");
			return Ok(());
		}

		let client = self.client.read().as_ref().and_then(Weak::upgrade).ok_or("No client!")?;
		let client = client.as_full_client().ok_or("No full client!")?;

		let mut report_queue = self.report_queue.lock();
		report_queue.filter(client, our_address, self.contract_address);
		report_queue.truncate();

		let mut resent_reports_in_block = self.resent_reports_in_block.lock();

		// Skip at least one block after sending malicious reports last time.
		if header.number() > *resent_reports_in_block + REPORTS_SKIP_BLOCKS {
			*resent_reports_in_block = header.number();
			let mut nonce = client.latest_nonce(our_address);
			for (address, block, data) in report_queue.iter() {
				debug!(target: "engine", "Retrying to report validator {} for misbehavior on block {} with nonce {}.",
					   address, block, nonce);
				while match self.transact(data.clone(), nonce) {
					Ok(()) => false,
					Err(EthcoreError::Transaction(transaction::Error::Old)) => true,
					Err(err) => {
						warn!(target: "engine", "Cannot report validator {} for misbehavior on block {}: {}",
							  address, block, err);
						false
					}
				} {
					warn!(target: "engine", "Nonce {} already used. Incrementing.", nonce);
					nonce += U256::from(1);
				}
				nonce += U256::from(1);
			}
		}

		Ok(())
	}

	fn on_epoch_begin(&self, _first: bool, _header: &Header, caller: &mut SystemCall) -> Result<(), EthcoreError> {
		let data = validator_set::functions::finalize_change::encode_input();
		caller(self.contract_address, data)
			.map(|_| ())
			.map_err(EngineError::FailedSystemCall)
			.map_err(Into::into)
	}

	fn genesis_epoch_data(&self, header: &Header, call: &Call) -> Result<Vec<u8>, String> {
		prove_initial(self.contract_address, header, call)
	}

	fn is_epoch_end(&self, _first: bool, _chain_head: &Header) -> Option<Vec<u8>> {
		None // no immediate transitions to contract.
	}

	fn signals_epoch_end(&self, first: bool, header: &Header, aux: AuxiliaryData)
		-> engine::EpochChange
	{
		let receipts = aux.receipts;

		// transition to the first block of a contract requires finality but has no log event.
		if first {
			debug!(target: "engine", "signalling transition to fresh contract.");
			let state_proof = Arc::new(StateProof {
				contract_address: self.contract_address,
				header: header.clone(),
			});
			return engine::EpochChange::Yes(engine::Proof::WithState(state_proof as Arc<_>));
		}

		// otherwise, we're checking for logs.
		let bloom = self.expected_bloom(header);
		let header_bloom = header.log_bloom();

		if &bloom & header_bloom != bloom { return engine::EpochChange::No }

		trace!(target: "engine", "detected epoch change event bloom");

		match receipts {
			None => engine::EpochChange::Unsure(AuxiliaryRequest::Receipts),
			Some(receipts) => match self.extract_from_event(bloom, header, receipts) {
				None => engine::EpochChange::No,
				Some(list) => {
					info!(target: "engine", "Signal for transition within contract. New validator list: {:?}",
						&*list);

					let proof = encode_proof(&header, receipts);
					engine::EpochChange::Yes(engine::Proof::Known(proof))
				}
			},
		}
	}

	fn epoch_set(&self, first: bool, machine: &Machine, _number: BlockNumber, proof: &[u8])
		-> Result<(SimpleList, Option<H256>), EthcoreError>
	{
		let rlp = Rlp::new(proof);

		if first {
			trace!(target: "engine", "Recovering initial epoch set");

			let (old_header, state_items) = decode_first_proof(&rlp)?;
			let number = old_header.number();
			let old_hash = old_header.hash();
			let addresses = check_first_proof(machine, self.contract_address, old_header, &state_items)
				.map_err(EngineError::InsufficientProof)?;

			trace!(target: "engine", "Extracted epoch validator set at block #{}: {} addresses",
				number, addresses.len());

			Ok((SimpleList::new(addresses), Some(old_hash)))
		} else {
			let (old_header, receipts) = decode_proof(&rlp)?;

			// ensure receipts match header.
			// TODO: optimize? these were just decoded.
			let found_root = triehash::ordered_trie_root(
				receipts.iter().map(::rlp::encode)
			);
			if found_root != *old_header.receipts_root() {
				return Err(BlockError::InvalidReceiptsRoot(
					Mismatch { expected: *old_header.receipts_root(), found: found_root }
				).into());
			}

			let bloom = self.expected_bloom(&old_header);

			match self.extract_from_event(bloom, &old_header, &receipts) {
				Some(list) => {
					trace!(target: "engine", "Extracted epoch validator set at block #{}: {} addresses", old_header.number(), list.len());

					Ok((list, Some(old_header.hash())))
				},
				None => Err(EngineError::InsufficientProof("No log event in proof.".into()).into()),
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

	fn register_client(&self, client: Weak<dyn EngineClient>) {
		trace!(target: "engine", "Setting up contract caller.");
		*self.client.write() = Some(client);
	}
}

/// A queue containing pending reports of malicious validators.
#[derive(Debug, Default)]
struct ReportQueue(VecDeque<(Address, BlockNumber, Vec<u8>)>);

impl ReportQueue {
	/// Pushes a report to the end of the queue.
	fn push(&mut self, addr: Address, block: BlockNumber, data: Vec<u8>) {
		self.0.push_back((addr, block, data));
	}

	/// Filters reports of validators that have already been reported or are banned.
	fn filter(
		&mut self,
		client: &dyn BlockChainClient,
		our_address: &Address,
		contract_address: Address,
	) {
		self.0.retain(|&(malicious_validator_address, block, ref _data)| {
			trace!(
				target: "engine",
				"Checking if report of malicious validator {} at block {} should be removed from cache",
				malicious_validator_address,
				block
			);
			// Check if the validator should be reported.
			let (data, decoder) = validator_set::functions::should_validator_report::call(
				*our_address, malicious_validator_address, block
			);
			match client.call_contract(BlockId::Latest, contract_address, data)
				.and_then(|result| decoder.decode(&result[..]).map_err(|e| e.to_string()))
			{
				Ok(false) => {
					trace!(target: "engine", "Successfully removed report from report cache");
					false
				}
				Ok(true) => true,
				Err(err) => {
					warn!(target: "engine", "Failed to query report status {:?}, dropping pending report.", err);
					false
				}
			}
		});
	}

	/// Returns an iterator over all transactions in the queue.
	fn iter(&self) -> impl Iterator<Item = &(Address, BlockNumber, Vec<u8>)> {
		self.0.iter()
	}

	/// Removes reports from the queue if it contains more than `MAX_QUEUED_REPORTS` entries.
	fn truncate(&mut self) {
		if self.0.len() > MAX_QUEUED_REPORTS {
			warn!(
				target: "engine",
				"Removing {} reports from report cache, even though it has not been finalized",
				self.0.len() - MAX_QUEUED_REPORTS
			);
			self.0.truncate(MAX_QUEUED_REPORTS);
		}
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use accounts::AccountProvider;
	use common_types::{
		ids::BlockId,
		engines::machine::AuxiliaryRequest,
		header::Header,
		log_entry::LogEntry,
		transaction::{Transaction, Action},
		verification::Unverified,
	};
	use client_traits::{BlockInfo, ChainInfo, ImportBlock, EngineClient, ForceUpdateSealing};
	use engine::{EpochChange, Proof};
	use ethcore::{
		miner::{self, MinerService},
		test_helpers::generate_dummy_client_with_spec
	};
	use parity_crypto::publickey::Secret;
	use ethereum_types::Address;
	use keccak_hash::keccak;
	use rustc_hex::FromHex;
	use spec;

	use super::super::ValidatorSet;
	use super::{ValidatorSafeContract, EVENT_NAME_HASH};

	#[test]
	fn fetches_validators() {
		let client = generate_dummy_client_with_spec(spec::new_validator_safe_contract);
		let addr: Address = "0000000000000000000000000000000000000005".parse().unwrap();
		let vc = Arc::new(ValidatorSafeContract::new(addr, None));
		vc.register_client(Arc::downgrade(&client) as _);
		let last_hash = client.best_block_header().hash();
		assert!(vc.contains(&last_hash, &"7d577a597b2742b498cb5cf0c26cdcd726d39e6e".parse::<Address>().unwrap()));
		assert!(vc.contains(&last_hash, &"82a978b3f5962a5b0957d9ee9eef472ee55b42f1".parse::<Address>().unwrap()));
	}

	#[test]
	fn knows_validators() {
		let _ = env_logger::try_init();
		let tap = Arc::new(AccountProvider::transient_provider());
		let s0: Secret = keccak("1").into();
		let v0 = tap.insert_account(s0.clone(), &"".into()).unwrap();
		let v1 = tap.insert_account(keccak("0").into(), &"".into()).unwrap();
		let chain_id = spec::new_validator_safe_contract().chain_id();
		let client = generate_dummy_client_with_spec(spec::new_validator_safe_contract);
		client.engine().register_client(Arc::downgrade(&client) as _);
		let validator_contract = "0000000000000000000000000000000000000005".parse::<Address>().unwrap();
		let signer = Box::new((tap.clone(), v1, "".into()));

		client.miner().set_author(miner::Author::Sealer(signer));
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
		EngineClient::update_sealing(&*client, ForceUpdateSealing::No);
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
		EngineClient::update_sealing(&*client, ForceUpdateSealing::No);
		// The transaction is not yet included so still unable to seal.
		assert_eq!(client.chain_info().best_block_number, 1);

		// Switch to the validator that is still there.
		let signer = Box::new((tap.clone(), v0, "".into()));
		client.miner().set_author(miner::Author::Sealer(signer));
		EngineClient::update_sealing(&*client, ForceUpdateSealing::No);
		assert_eq!(client.chain_info().best_block_number, 2);
		// Switch back to the added validator, since the state is updated.
		let signer = Box::new((tap.clone(), v1, "".into()));
		client.miner().set_author(miner::Author::Sealer(signer));
		let tx = Transaction {
			nonce: 2.into(),
			gas_price: 0.into(),
			gas: 21000.into(),
			action: Action::Call(Address::zero()),
			value: 0.into(),
			data: Vec::new(),
		}.sign(&s0, Some(chain_id));
		client.miner().import_own_transaction(client.as_ref(), tx.into()).unwrap();
		EngineClient::update_sealing(&*client, ForceUpdateSealing::No);
		// Able to seal again.
		assert_eq!(client.chain_info().best_block_number, 3);

		// Check syncing.
		let sync_client = generate_dummy_client_with_spec(spec::new_validator_safe_contract);
		sync_client.engine().register_client(Arc::downgrade(&sync_client) as _);
		for i in 1..4 {
			sync_client.import_block(Unverified::from_rlp(client.block(BlockId::Number(i)).unwrap().into_inner()).unwrap()).unwrap();
		}
		sync_client.flush_queue();
		assert_eq!(sync_client.chain_info().best_block_number, 3);
	}

	#[test]
	fn detects_bloom() {
		let client = generate_dummy_client_with_spec(spec::new_validator_safe_contract);
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
		let client = generate_dummy_client_with_spec(spec::new_validator_safe_contract);
		let engine = client.engine().clone();

		let mut new_header = Header::default();
		new_header.set_number(0); // so the validator set doesn't look for a log

		match engine.signals_epoch_end(&new_header, Default::default()) {
			EpochChange::Yes(Proof::WithState(_)) => {},
			_ => panic!("Expected state to be required to prove initial signal"),
		};
	}
}
