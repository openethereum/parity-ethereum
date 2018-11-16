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

//! The `Hbbft` consensus engine.

#![allow(unused_imports)]

use std::sync::{Arc, Weak};

use ethereum_types::{H256, H520, Address, U128, U256};
use unexpected::Mismatch;

use account_provider::AccountProvider;
use block::{ExecutedBlock, IsBlock};
use client::{EngineClient, BlockInfo};
use engines::{default_system_or_code_call, Engine, EngineError, EpochChange, ForkChoice, Seal};
use engines::block_reward::{apply_block_rewards, BlockRewardContract, RewardKind};
use engines::signer::EngineSigner;
use engines::validator_set::{ValidatorSet, SimpleList, new_validator_set};
use error::{BlockError, Error, ErrorKind};
use ethjson;
use ethkey::Password;
use header::{Header, ExtendedHeader};
use machine::{AuxiliaryData, Call, EthereumMachine};
use parking_lot::RwLock;
use receipt::Receipt;
use rlp::{self, Decodable, Encodable, RlpStream, Rlp};
use types::BlockNumber;

// TODO: Use a threshold signature of the block.
/// A temporary fixed seal code. The seal has only a single field, containing this string.
const SEAL: &str = "Honey Badger isn't afraid of seals!";

/// The block number for the first block in the chain. This value is used to create the genesis
/// block's validator-set proof.
const GENESIS_BLOCK_NUMBER: BlockNumber = 0;

/// The proof of block finality for the first block in the chain. This value is used to create the
/// genesis block's validator-set proof. This is an empty bytes slice because the genesis block
/// does not require finality.
const GENESIS_FINALITY_PROOF: &[u8] = &[];

type StepNumber = u64;

fn get_step_number_from_header(header: &Header) -> StepNumber {
	header.number() - 1
}

/// Validator-set epoch transitions are stored in the blockchain database, this structure
/// represents the structure of the RLP encoded bytes that are stored in the database. This
/// structure is used to decode the RLP encoded transition-proof bytes that are returned from
/// `Client::epoch_transition_for()`.
struct TransitionProof {
	block_number: BlockNumber,
	validator_set_proof: Vec<u8>,
	finality_proof: Vec<u8>,
}

impl TransitionProof {
	/// Decodes RLP bytes into a `TransitionProof`.
	fn from_rlp_bytes(rlp_bytes: &[u8]) -> Result<Self, rlp::DecoderError> {
		let rlp = Rlp::new(rlp_bytes);
		let block_number = rlp.val_at(0)?;
		let validator_set_proof = rlp.at(1)?.data()?.to_vec();
		let finality_proof = rlp.at(2)?.data()?.to_vec();
		Ok(TransitionProof { block_number, validator_set_proof, finality_proof })
	}

	/// RLP encodes a `TransitionProof`.
	fn into_rlp_bytes(&self) -> Vec<u8> {
		let mut rlp = RlpStream::new_list(3);
		rlp.append(&self.block_number);
		rlp.append(&self.validator_set_proof);
		rlp.append(&self.finality_proof);
		rlp.out()
	}

	/// Creates a proof for the genesis block's validator-set containing: the block number,
	/// validator-set proof, and the genesis block's finality proof.
	fn new_genesis_proof(validator_set_proof: Vec<u8>) -> Self {
		TransitionProof {
			block_number: GENESIS_BLOCK_NUMBER,
			validator_set_proof: validator_set_proof,
			finality_proof: GENESIS_FINALITY_PROOF.to_vec(),
		}
	}
}

/// Represents an RLP decoded "set-proof". Set-proofs are used to store the receipts for a block
/// for which a validator-set transition occurred. The `receipts` in this structure contains an
/// `initiateChange` event log.
struct ValidatorSetProof {
	header: Header,
	receipts: Vec<Receipt>,
}

impl ValidatorSetProof {
	/// This method is the same as `fn decode_proof()` in
	/// `ethcore/src/engines/validator_set/safe_contract.rs`.
	fn from_rlp_bytes(rlp_bytes: &[u8]) -> Result<Self, rlp::DecoderError> {
		let rlp = Rlp::new(rlp_bytes);
		let header = rlp.val_at(0)?;
		let receipts = rlp.list_at(1)?;
		Ok(ValidatorSetProof { header, receipts })
	}

	/// This method is the same as `fn encode_proof()` in
	/// `ethcore/src/engines/validator_set/safe_contract.rs`.
	fn into_rlp_bytes(&self) -> Vec<u8> {
		let mut rlp = RlpStream::new_list(2);
		rlp.append(&self.header);
		rlp.append_list(&self.receipts);
		rlp.out()
	}
}

pub struct HbbftParams {
	/// Specifies whether or not we are using timestamps in units of milliseconds within the our
	/// block headers.
	pub millisecond_timestamp: bool,

	/// We can query this trait object for information concerning the validator-set.
	pub validators: Box<ValidatorSet>,

	/// If we are using a smart contract to calculate and distribute block rewards, we store the
	/// block reward contract address here. `BlockRewardContract` has a `reward()` method that
	/// handles the logic for serializing the input, deserializing the output, and calling the
	/// block reward contract's `reward()` function.
	pub block_reward_contract: Option<BlockRewardContract>,

	/// If we are not using a block reward smart contract (i.e. `self.block_reward_contract` is
	/// `None`), the amount specified by `block_reward` is added to the address in each newly
	/// sealed block's `author` block header field. We default this value to `U256::zero()` if
	/// the user did not provide a `block_reward` parameter in their Hbbft engine JSON spec.
	pub block_reward: U256,
}

impl From<ethjson::spec::HbbftParams> for HbbftParams {
	fn from(p: ethjson::spec::HbbftParams) -> Self {
		let block_reward_contract = p.block_reward_contract_address
			.map(|json_addr| {
				let addr: Address = json_addr.into();
				BlockRewardContract::new_from_address(addr)
			});

		let block_reward: U256 = match p.block_reward {
			Some(json_uint) => json_uint.into(),
			None => U256::zero(),
		};

		HbbftParams {
			millisecond_timestamp: p.millisecond_timestamp,
			validators: new_validator_set(p.validators),
			block_reward_contract,
			block_reward,
		}
	}
}

/// An engine which does not provide any consensus mechanism, just seals blocks internally.
/// Only seals blocks which have transactions.
pub struct Hbbft {
	machine: EthereumMachine,
	client: RwLock<Option<Weak<EngineClient>>>,
	signer: RwLock<EngineSigner>,
	validators: Box<ValidatorSet>,
	millisecond_timestamp: bool,
	block_reward_contract: Option<BlockRewardContract>,
	block_reward: U256,
}

impl Hbbft {
	/// Returns new instance of Hbbft over the given state machine.
	pub fn new(params: HbbftParams, machine: EthereumMachine) -> Self {
		Hbbft {
			machine,
			client: RwLock::new(None),
			signer: Default::default(),
			validators: params.validators,
			millisecond_timestamp: params.millisecond_timestamp,
			block_reward_contract: params.block_reward_contract,
			block_reward: params.block_reward,
		}
	}
}

impl Engine<EthereumMachine> for Hbbft {
	fn name(&self) -> &str {
		"Hbbft"
	}

	fn machine(&self) -> &EthereumMachine { &self.machine }

	fn seals_internally(&self) -> Option<bool> { Some(true) }

	fn seal_fields(&self, _header: &Header) -> usize { 1 }

	fn should_miner_prepare_blocks(&self) -> bool { false }

	fn generate_seal(&self, block: &ExecutedBlock, _parent: &Header) -> Seal {
		debug!(target: "engine", "####### Hbbft::generate_seal: Called for block: {:?}.", block);
		// match self.client.read().as_ref().and_then(|weak| weak.upgrade()) {
		//	Some(client) => {
		//		let best_block_header_num = (*client).as_full_client().unwrap().best_block_header().number();

		//		debug!(target: "engine", "###### block.header.number(): {}, best_block_header_num: {}",
		//			block.header.number(), best_block_header_num);

		//		if block.header.number() > best_block_header_num {
		//			Seal::Regular(vec![
		//				rlp::encode(&SEAL),
		//				// rlp::encode(&(&H520::from(&b"Another Field"[..]) as &[u8])),
		//			])
		//		} else {
		//			debug!(target: "engine", "Hbbft::generate_seal: Returning `Seal::None`.");
		//			Seal::None
		//		}
		//	},
		//	None => {
		//		debug!(target: "engine", "No client ref available.");
		//		Seal::None
		//	},
		// }

		Seal::Regular(vec![
			rlp::encode(&SEAL),
		])
	}

	fn verify_local_seal(&self, header: &Header) -> Result<(), Error> {
		if header.seal() == &[rlp::encode(&SEAL)] {
			Ok(())
		} else {
			Err(BlockError::InvalidSeal.into())
		}
	}

	fn open_block_header_timestamp(&self, parent_timestamp: u64) -> u64 {
		use std::{time, cmp};

		let dur = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap_or_default();
		let mut now = dur.as_secs();
		if self.millisecond_timestamp {
			now = now * 1000 + dur.subsec_millis() as u64;
		}
		cmp::max(now, parent_timestamp)
	}

	fn is_timestamp_valid(&self, header_timestamp: u64, parent_timestamp: u64) -> bool {
		header_timestamp >= parent_timestamp
	}

	fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> ForkChoice {
		// debug!("######## ENGINE-HBBFT::FORK_CHOICE: \n	 NEW: {:?}, \n	  OLD: {:?}", new, current);
		use ::parity_machine::TotalScoredHeader;
		if new.header.number() > current.header.number() {
			debug_assert!(new.total_score() > current.total_score());
			ForkChoice::New
		} else if new.header.number() < current.header.number() {
			debug_assert!(new.total_score() < current.total_score());
			ForkChoice::Old
		} else {
			// The entire header won't always be identical but the score should be:
			debug_assert_eq!(new.total_score(), current.total_score());
			ForkChoice::Old
		}
	}

	fn register_client(&self, client: Weak<EngineClient>) {
		*self.client.write() = Some(client.clone());
	}

	fn set_signer(&self, ap: Arc<AccountProvider>, address: Address, password: Password) {
		self.signer.write().set(ap, address, password);
	}

	/// Called by `OpenBlock::new()`. This method is responsible for running any validator-set
	/// related logic that should be run when a new block is opened. This method is called prior to
	/// any transactions being added to `block`. This method only runs if a newly opened block's
	/// block number is the first for a validator-set source (i.e. a hardcoded list of validator
	/// addresses or a smart contract that contains the validator addresses).
	///
	/// If the validator-set source corresponding to `block`'s block number uses a smart contract
	/// to acquire the list of validator addresses, this method will call the smart contract's
	/// `finalizeChange` function. The `finalizeChange` contract function will finalize any pending
	/// validator-set changes currently known to the Safe Contract.
	///
	/// # Arguments
	///
	/// * `block` - an `OpenBlock`'s internal block info (i.e. an `ExecutedBlock`).
	/// * `epoch_begin` - tells us whether or not `block` is the first block for a validator-set
	/// source (i.e. for a hardcoded list of validator addresses or for a safe contract from which
	/// we query the list of validator addresses).
	/// * `ancestry` - an iterator over all finalized block headers starting from the first block
	/// for the validator-set source up to and including `block`'s parent. We ignore this argument
	/// (it appears that every consensus engine ignores the `ancestry` argument, who knows why
	/// it's there).
	fn on_new_block(
		&self,
		block: &mut ExecutedBlock,
		epoch_begin: bool,
		_ancestry: &mut Iterator<Item=ExtendedHeader>,
	) -> Result<(), Error> {
		if !epoch_begin {
			return Ok(());
		}
		let header = block.header().clone();
		let mut call = |to, data| {
			let gas = U256::max_value();
			self.machine
				.execute_as_system(block, to, gas, Some(data))
				.map_err(|e| format!("{}", e))
		};
		self.validators.on_epoch_begin(epoch_begin, &header, &mut call)
	}

	/// Called by `OpenBlock::close_and_lock()`. This method is responsible for running any
	/// validator-set related logic that should be run after all transactions have been added to a
	/// block (i.e. when an `OpenBlock` is ready to be closed and locked).
	///
	/// If the `Hbbft` engine is configured to use a smart contract to distribute block rewards via
	/// a `reward()` function, this method will call the `reward()` contract function.
	///
	/// # Arguments
	///
	/// * `block` - an `OpenBlock`'s internal block info (i.e. an `ExecutedBlock`).
	fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
		// TODO: Indica nodes are temporarily using a constant address as the block author (set
		// using Parity's `--engine-signer` CLI option in the `indica-node-signer` startup script).
		// We must determine how exactly we will set the block author going forward when using the
		// `Hbbft` consensus engine; currently the POA network uses Aura's round robin algorithm
		// for selecting each block proposer/author from the set of validators.
		let author = *block.header().author();
		let rewards: Vec<(Address, RewardKind, U256)> = match self.block_reward_contract {
			Some(ref contract) => {
				let beneficiaries = vec![(author, RewardKind::Author)];
				let mut call = default_system_or_code_call(&self.machine, block);
				contract.reward(&beneficiaries, &mut call)?
					.into_iter()
					.map(|(author, amount)| (author, RewardKind::External, amount))
					.collect()
			},
			None => vec![(author, RewardKind::Author, self.block_reward)],
		};
		apply_block_rewards(&rewards, block, &self.machine)
	}

	/// Verifies that the number of seal fields in `header` is consistent with the expected number
	/// of seal fields for the `Hbbft` engine.
	///
	/// # Arguments
	///
	/// * `header` - the block header for which we want to check the number of seal fields.
	fn verify_block_basic(&self, header: &Header) -> Result<(), Error> {
		let n_seal_fields_found = header.seal().len();
		let n_seal_fields_expected = self.seal_fields(header);

		if n_seal_fields_found == n_seal_fields_expected {
			return Ok(());
		}

		trace!(
			target: "engine",
			"verify_block_basic: invalid number of seal fields, block number: {}, found: {}, expected: {}",
			header.number(),
			n_seal_fields_found,
			n_seal_fields_expected,
		);
		let invalid_number_of_seal_fields = Mismatch {
			expected: n_seal_fields_expected,
			found: n_seal_fields_found,
		};
		Err(BlockError::InvalidSealArity(invalid_number_of_seal_fields).into())
	}

	/// Returns an RLP encoded proof of the set validator addresses at the genesis block. The
	/// returned proof is an RLP encoded list containing three elements: the genesis block's block
	/// number, the proof of the validator-set at the genesis block, and the genesis block's
	/// "finality proof".
	///
	/// The genesis block number is always set to `0`. The genesis block's finality proof is always
	/// set to an empty bytes array because the genesis block does not require finality.
	///
	/// # Arguments
	///
	/// * `header` - the genesis block's header.
	/// * `call` - a function that executes a synchronous contract call within the EVM (EVM is an
	/// instance of `EthereumMachine`).
	fn genesis_epoch_data(&self, header: &Header, call: &Call) -> Result<Vec<u8>, String> {
		let validator_set_proof = self.validators.genesis_epoch_data(header, call)?;
		let genesis_transition_proof = TransitionProof::new_genesis_proof(validator_set_proof);
		let rlp_bytes = genesis_transition_proof.into_rlp_bytes();
		Ok(rlp_bytes)
	}

	/// Checks whether or not the block corresponding to `header` is the last block for a
	/// validator-set epoch. If `EpochChange::Yes` is returned from this method, the changes to the
	/// validator-set will not take effect until they have received finality.
	///
	/// This method is used by `Importer::check_epoch_end_signal()`. If `EpochChange::Yes` is
	/// returned, `Importer::check_epoch_end_signal()` will insert a pending validator-set
	/// transition into the blockchain database at the block corresponding to `header`.
	///
	/// # Arguments
	///
	/// * `header` - the header for the block for which we ask the question: "is this the last
	/// block for a validator-set epoch?".
	/// * `aux` - holds any block data and block transaction receipts from the block corresponding
	/// to `header`.
	fn signals_epoch_end(&self, header: &Header, aux: AuxiliaryData) -> EpochChange<EthereumMachine> {
		let is_genesis_block = header.number() == 0;
		self.validators.signals_epoch_end(is_genesis_block, header, aux)
	}
}

impl Hbbft {
	/// Returns the set of Validator addresses at the block corresponding to `header`.
	///
	/// # Arguments
	///
	/// * `header` - the block header at which we want to get the set of validator addresses.
	fn get_validator_set_at_block(&self, header: &Header) -> Result<Vec<Address>, Error> {
		let client_opt = self.client
			.read()
			.as_ref()
			.and_then(|weak| weak.upgrade());
		let client = match client_opt {
			Some(client) => client,
			None => {
				debug!(target: "engine", "get_validator_set_at_block: missing client.");
				return Err(EngineError::RequiresClient.into())
			},
		};

		// Get the validator-set transition for the block corresponding to `header`.
		let parent_hash = *header.parent_hash();
		let transition = match client.epoch_transition_for(parent_hash) {
			Some(transition) => transition,
			None => {
				let block_number = header.number();
				debug!(
					target: "engine",
					"get_validator_set_at_block: no epoch transition exists for block: {:?}",
					block_number
				);
				return Err(EngineError::NoTransitionFoundForBlock(block_number).into());
			},
		};

		// Decode the stored transition proof and extract the validator-set from the event logs.
		let transition_proof = TransitionProof::from_rlp_bytes(&transition.proof).unwrap();
		let is_first_validator_set = transition_proof.block_number == 0;
		let validator_addrs = self.validators
			.epoch_set(
				is_first_validator_set,
				&self.machine,
				transition_proof.block_number,
				&transition_proof.validator_set_proof,
			).map(|(simple_list, _block_hash)| simple_list.into_inner())
			.unwrap();
		Ok(validator_addrs)
	}
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use ethereum_types::{H520, Address};

	use block::*;
	use engines::Seal;
	use factory::Factories;
	use header::Header;
	use spec::Spec;
	use test_helpers::get_temp_state_db;

	#[test]
	fn hbbft_can_seal() {
		let spec = Spec::new_hbbft();
		let engine = &*spec.engine;
		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let genesis_header = spec.genesis_header();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = {
			let factories = Factories::default();
			let tracing_enabled = false;
			let author = Address::default();
			let gas_range = (3141562.into(), 31415620.into());
			let extra_data = vec![];
			let is_first_block_for_validator_set_source = false;
			let ancestry = &mut Vec::new().into_iter();
			OpenBlock::new(
				engine,
				factories,
				tracing_enabled,
				db,
				&genesis_header,
				last_hashes,
				author,
				gas_range,
				extra_data,
				is_first_block_for_validator_set_source,
				ancestry,
			).unwrap()
		};
		let b = b.close_and_lock().unwrap();
		if let Seal::Regular(seal) = engine.generate_seal(b.block(), &genesis_header) {
			assert!(b.try_seal(engine, seal).is_ok());
		} else {
			panic!("Failed to seal block.");
		}
	}

	#[test]
	fn hbbft_cant_verify() {
		let engine = Spec::new_hbbft().engine;
		let mut header: Header = Header::default();
		assert!(engine.verify_block_basic(&header).is_ok());
		header.set_seal(vec![::rlp::encode(&H520::default())]);
		assert!(engine.verify_block_unordered(&header).is_ok());
	}
}
