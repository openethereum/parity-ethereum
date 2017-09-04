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

use std::path::Path;
use std::cmp;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Weak};
use hash::{KECCAK_EMPTY_LIST_RLP};
use ethash::{quick_get_difficulty, slow_get_seedhash, EthashManager};
use util::*;
use block::*;
use builtin::Builtin;
use vm::EnvInfo;
use error::{BlockError, Error, TransactionError};
use trace::{Tracer, ExecutiveTracer, RewardType};
use header::{Header, BlockNumber};
use state::CleanupMode;
use spec::CommonParams;
use transaction::{UnverifiedTransaction, SignedTransaction};
use engines::{self, Engine};
use evm::Schedule;
use ethjson;
use rlp::{self, UntrustedRlp};
use vm::LastHashes;
use semantic_version::SemanticVersion;
use tx_filter::{TransactionFilter};
use client::{Client, BlockChainClient};

/// Parity tries to round block.gas_limit to multiple of this constant
pub const PARITY_GAS_LIMIT_DETERMINANT: U256 = U256([37, 0, 0, 0]);

/// Number of blocks in an ethash snapshot.
// make dependent on difficulty incrment divisor?
const SNAPSHOT_BLOCKS: u64 = 5000;
/// Maximum number of blocks allowed in an ethash snapshot.
const MAX_SNAPSHOT_BLOCKS: u64 = 30000;


/// Ethash params.
#[derive(Debug, PartialEq)]
pub struct EthashParams {
	/// Minimum difficulty.
	pub minimum_difficulty: U256,
	/// Difficulty bound divisor.
	pub difficulty_bound_divisor: U256,
	/// Difficulty increment divisor.
	pub difficulty_increment_divisor: u64,
	/// Metropolis difficulty increment divisor.
	pub metropolis_difficulty_increment_divisor: u64,
	/// Block duration.
	pub duration_limit: u64,
	/// Homestead transition block number.
	pub homestead_transition: u64,
	/// DAO hard-fork transition block (X).
	pub dao_hardfork_transition: u64,
	/// DAO hard-fork refund contract address (C).
	pub dao_hardfork_beneficiary: Address,
	/// DAO hard-fork DAO accounts list (L)
	pub dao_hardfork_accounts: Vec<Address>,
	/// Transition block for a change of difficulty params (currently just bound_divisor).
	pub difficulty_hardfork_transition: u64,
	/// Difficulty param after the difficulty transition.
	pub difficulty_hardfork_bound_divisor: U256,
	/// Block on which there is no additional difficulty from the exponential bomb.
	pub bomb_defuse_transition: u64,
	/// Number of first block where EIP-100 rules begin.
	pub eip100b_transition: u64,
	/// Number of first block where EIP-150 rules begin.
	pub eip150_transition: u64,
	/// Number of first block where EIP-160 rules begin.
	pub eip160_transition: u64,
	/// Number of first block where EIP-161.abc begin.
	pub eip161abc_transition: u64,
	/// Number of first block where EIP-161.d begins.
	pub eip161d_transition: u64,
	/// Number of first block where ECIP-1010 begins.
	pub ecip1010_pause_transition: u64,
	/// Number of first block where ECIP-1010 ends.
	pub ecip1010_continue_transition: u64,
	/// Total block number for one ECIP-1017 era.
	pub ecip1017_era_rounds: u64,
	/// Maximum amount of code that can be deploying into a contract.
	pub max_code_size: u64,
	/// Number of first block where the max gas limit becomes effective.
	pub max_gas_limit_transition: u64,
	/// Maximum valid block gas limit,
	pub max_gas_limit: U256,
	/// Number of first block where the minimum gas price becomes effective.
	pub min_gas_price_transition: u64,
	/// Do not alow transactions with lower gas price.
	pub min_gas_price: U256,
}

impl From<ethjson::spec::EthashParams> for EthashParams {
	fn from(p: ethjson::spec::EthashParams) -> Self {
		EthashParams {
			minimum_difficulty: p.minimum_difficulty.into(),
			difficulty_bound_divisor: p.difficulty_bound_divisor.into(),
			difficulty_increment_divisor: p.difficulty_increment_divisor.map_or(10, Into::into),
			metropolis_difficulty_increment_divisor: p.metropolis_difficulty_increment_divisor.map_or(9, Into::into),
			duration_limit: p.duration_limit.map_or(0, Into::into),
			homestead_transition: p.homestead_transition.map_or(0, Into::into),
			dao_hardfork_transition: p.dao_hardfork_transition.map_or(u64::max_value(), Into::into),
			dao_hardfork_beneficiary: p.dao_hardfork_beneficiary.map_or_else(Address::new, Into::into),
			dao_hardfork_accounts: p.dao_hardfork_accounts.unwrap_or_else(Vec::new).into_iter().map(Into::into).collect(),
			difficulty_hardfork_transition: p.difficulty_hardfork_transition.map_or(u64::max_value(), Into::into),
			difficulty_hardfork_bound_divisor: p.difficulty_hardfork_bound_divisor.map_or(p.difficulty_bound_divisor.into(), Into::into),
			bomb_defuse_transition: p.bomb_defuse_transition.map_or(u64::max_value(), Into::into),
			eip100b_transition: p.eip100b_transition.map_or(u64::max_value(), Into::into),
			eip150_transition: p.eip150_transition.map_or(0, Into::into),
			eip160_transition: p.eip160_transition.map_or(0, Into::into),
			eip161abc_transition: p.eip161abc_transition.map_or(0, Into::into),
			eip161d_transition: p.eip161d_transition.map_or(u64::max_value(), Into::into),
			ecip1010_pause_transition: p.ecip1010_pause_transition.map_or(u64::max_value(), Into::into),
			ecip1010_continue_transition: p.ecip1010_continue_transition.map_or(u64::max_value(), Into::into),
			ecip1017_era_rounds: p.ecip1017_era_rounds.map_or(u64::max_value(), Into::into),
			max_code_size: p.max_code_size.map_or(u64::max_value(), Into::into),
			max_gas_limit_transition: p.max_gas_limit_transition.map_or(u64::max_value(), Into::into),
			max_gas_limit: p.max_gas_limit.map_or(U256::max_value(), Into::into),
			min_gas_price_transition: p.min_gas_price_transition.map_or(u64::max_value(), Into::into),
			min_gas_price: p.min_gas_price.map_or(U256::zero(), Into::into),
		}
	}
}

/// Engine using Ethash proof-of-work consensus algorithm, suitable for Ethereum
/// mainnet chains in the Olympic, Frontier and Homestead eras.
pub struct Ethash {
	params: CommonParams,
	ethash_params: EthashParams,
	builtins: BTreeMap<Address, Builtin>,
	pow: EthashManager,
	tx_filter: Option<TransactionFilter>,
}

impl Ethash {
	/// Create a new instance of Ethash engine
	pub fn new<T: AsRef<Path>>(
		cache_dir: T,
		params: CommonParams,
		ethash_params: EthashParams,
		builtins: BTreeMap<Address, Builtin>,
	) -> Arc<Self> {
		Arc::new(Ethash {
			tx_filter: TransactionFilter::from_params(&params),
			params,
			ethash_params,
			builtins,
			pow: EthashManager::new(cache_dir),
		})
	}
}

// TODO [rphmeier]
//
// for now, this is different than Ethash's own epochs, and signal
// "consensus epochs".
// in this sense, `Ethash` is epochless: the same `EpochVerifier` can be used
// for any block in the chain.
// in the future, we might move the Ethash epoch
// caching onto this mechanism as well.
impl engines::EpochVerifier for Arc<Ethash> {
	fn verify_light(&self, _header: &Header) -> Result<(), Error> { Ok(()) }
	fn verify_heavy(&self, header: &Header) -> Result<(), Error> {
		self.verify_block_unordered(header, None)
	}
}

impl Engine for Arc<Ethash> {
	fn name(&self) -> &str { "Ethash" }
	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }
	// Two fields - mix
	fn seal_fields(&self) -> usize { 2 }

	fn params(&self) -> &CommonParams { &self.params }
	fn additional_params(&self) -> HashMap<String, String> { hash_map!["registrar".to_owned() => self.params().registrar.hex()] }

	fn builtins(&self) -> &BTreeMap<Address, Builtin> {
		&self.builtins
	}

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, header: &Header) -> BTreeMap<String, String> {
		if header.seal().len() == self.seal_fields() {
			map![
				"nonce".to_owned() => format!("0x{}", header.nonce().hex()),
				"mixHash".to_owned() => format!("0x{}", header.mix_hash().hex())
			]
		} else {
			BTreeMap::default()
		}
	}

	fn schedule(&self, block_number: BlockNumber) -> Schedule {
		trace!(target: "client", "Creating schedule. fCML={}, bGCML={}", self.ethash_params.homestead_transition, self.ethash_params.eip150_transition);

		if block_number < self.ethash_params.homestead_transition {
			Schedule::new_frontier()
		} else if block_number < self.ethash_params.eip150_transition {
			Schedule::new_homestead()
		} else {
			let mut schedule = Schedule::new_post_eip150(
				self.ethash_params.max_code_size as usize,
				block_number >= self.ethash_params.eip160_transition,
				block_number >= self.ethash_params.eip161abc_transition,
				block_number >= self.ethash_params.eip161d_transition);

			self.params().update_schedule(block_number, &mut schedule);
			schedule
		}
	}

	fn signing_chain_id(&self, env_info: &EnvInfo) -> Option<u64> {
		if env_info.number >= self.params().eip155_transition {
			Some(self.params().chain_id)
		} else {
			None
		}
	}

	fn populate_from_parent(&self, header: &mut Header, parent: &Header, gas_floor_target: U256, mut gas_ceil_target: U256) {
		let difficulty = self.calculate_difficulty(header, parent);
		if header.number() >= self.ethash_params.max_gas_limit_transition && gas_ceil_target > self.ethash_params.max_gas_limit {
			warn!("Gas limit target is limited to {}", self.ethash_params.max_gas_limit);
			gas_ceil_target = self.ethash_params.max_gas_limit;
		}
		let gas_limit = {
			let gas_limit = parent.gas_limit().clone();
			let bound_divisor = self.params().gas_limit_bound_divisor;
			let lower_limit = gas_limit - gas_limit / bound_divisor + 1.into();
			let upper_limit = gas_limit + gas_limit / bound_divisor - 1.into();
			let gas_limit = if gas_limit < gas_floor_target {
				let gas_limit = cmp::min(gas_floor_target, upper_limit);
				round_block_gas_limit(gas_limit, lower_limit, upper_limit)
			} else if gas_limit > gas_ceil_target {
				let gas_limit = cmp::max(gas_ceil_target, lower_limit);
				round_block_gas_limit(gas_limit, lower_limit, upper_limit)
			} else {
				let total_lower_limit = cmp::max(lower_limit, gas_floor_target);
				let total_upper_limit = cmp::min(upper_limit, gas_ceil_target);
				let gas_limit = cmp::max(gas_floor_target, cmp::min(total_upper_limit,
					lower_limit + (header.gas_used().clone() * 6.into() / 5.into()) / bound_divisor));
				round_block_gas_limit(gas_limit, total_lower_limit, total_upper_limit)
			};
			// ensure that we are not violating protocol limits
			debug_assert!(gas_limit >= lower_limit);
			debug_assert!(gas_limit <= upper_limit);
			gas_limit
		};
		header.set_difficulty(difficulty);
		header.set_gas_limit(gas_limit);
		if header.number() >= self.ethash_params.dao_hardfork_transition &&
			header.number() <= self.ethash_params.dao_hardfork_transition + 9 {
			header.set_extra_data(b"dao-hard-fork"[..].to_owned());
		}
		header.note_dirty();
//		info!("ethash: populate_from_parent #{}: difficulty={} and gas_limit={}", header.number(), header.difficulty(), header.gas_limit());
	}

	fn on_new_block(
		&self,
		block: &mut ExecutedBlock,
		last_hashes: Arc<LastHashes>,
		_begins_epoch: bool,
	) -> Result<(), Error> {
		let parent_hash = block.fields().header.parent_hash().clone();
		engines::common::push_last_hash(block, last_hashes, self, &parent_hash)?;
		if block.fields().header.number() == self.ethash_params.dao_hardfork_transition {
			let state = block.fields_mut().state;
			for child in &self.ethash_params.dao_hardfork_accounts {
				let beneficiary = &self.ethash_params.dao_hardfork_beneficiary;
				state.balance(child)
					.and_then(|b| state.transfer_balance(child, beneficiary, &b, CleanupMode::NoEmpty))?;
			}
		}
		Ok(())
	}

	/// Apply the block reward on finalisation of the block.
	/// This assumes that all uncles are valid uncles (i.e. of at least one generation before the current).
	fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
		use std::ops::Shr;
		let reward = self.params().block_reward;
		let tracing_enabled = block.tracing_enabled();
		let fields = block.fields_mut();
		let eras_rounds = self.ethash_params.ecip1017_era_rounds;
		let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, reward, fields.header.number());
		let mut tracer = ExecutiveTracer::default();

		// Bestow block reward
		let result_block_reward = reward + reward.shr(5) * U256::from(fields.uncles.len());
		fields.state.add_balance(
			fields.header.author(),
			&result_block_reward,
			CleanupMode::NoEmpty
		)?;

		if tracing_enabled {
			let block_author = fields.header.author().clone();
			tracer.trace_reward(block_author, result_block_reward, RewardType::Block);
		}

		// Bestow uncle rewards
		let current_number = fields.header.number();
		for u in fields.uncles.iter() {
			let uncle_author = u.author().clone();
			let result_uncle_reward: U256;

			if eras == 0 {
				result_uncle_reward = (reward * U256::from(8 + u.number() - current_number)).shr(3);
				fields.state.add_balance(
					u.author(),
					&result_uncle_reward,
					CleanupMode::NoEmpty
				)
			} else {
				result_uncle_reward = reward.shr(5);
				fields.state.add_balance(
					u.author(),
					&result_uncle_reward,
					CleanupMode::NoEmpty
				)
			}?;

			// Trace uncle rewards
			if tracing_enabled {
				tracer.trace_reward(uncle_author, result_uncle_reward, RewardType::Uncle);
			}
		}

		// Commit state so that we can actually figure out the state root.
		fields.state.commit()?;
		if tracing_enabled {
			fields.traces.as_mut().map(|mut traces| traces.push(tracer.drain()));
		}
		Ok(())
	}

	fn verify_block_basic(&self, header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		// check the seal fields.
		if header.seal().len() != self.seal_fields() {
			return Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: header.seal().len() }
			)));
		}
		UntrustedRlp::new(&header.seal()[0]).as_val::<H256>()?;
		UntrustedRlp::new(&header.seal()[1]).as_val::<H64>()?;

		// TODO: consider removing these lines.
		let min_difficulty = self.ethash_params.minimum_difficulty;
		if header.difficulty() < &min_difficulty {
			return Err(From::from(BlockError::DifficultyOutOfBounds(OutOfBounds { min: Some(min_difficulty), max: None, found: header.difficulty().clone() })))
		}

		let difficulty = Ethash::boundary_to_difficulty(&H256(quick_get_difficulty(
			&header.bare_hash().0,
			header.nonce().low_u64(),
			&header.mix_hash().0
		)));
		if &difficulty < header.difficulty() {
			return Err(From::from(BlockError::InvalidProofOfWork(OutOfBounds { min: Some(header.difficulty().clone()), max: None, found: difficulty })));
		}

		if header.number() >= self.ethash_params.dao_hardfork_transition &&
			header.number() <= self.ethash_params.dao_hardfork_transition + 9 &&
			header.extra_data()[..] != b"dao-hard-fork"[..] {
			return Err(From::from(BlockError::ExtraDataOutOfBounds(OutOfBounds { min: None, max: None, found: 0 })));
		}

		if header.gas_limit() > &0x7fffffffffffffffu64.into() {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: None, max: Some(0x7fffffffffffffffu64.into()), found: header.gas_limit().clone() })));
		}

		Ok(())
	}

	fn verify_block_unordered(&self, header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		if header.seal().len() != self.seal_fields() {
			return Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: header.seal().len() }
			)));
		}
		let result = self.pow.compute_light(header.number() as u64, &header.bare_hash().0, header.nonce().low_u64());
		let mix = H256(result.mix_hash);
		let difficulty = Ethash::boundary_to_difficulty(&H256(result.value));
		trace!(target: "miner", "num: {}, seed: {}, h: {}, non: {}, mix: {}, res: {}" , header.number() as u64, H256(slow_get_seedhash(header.number() as u64)), header.bare_hash(), header.nonce().low_u64(), H256(result.mix_hash), H256(result.value));
		if mix != header.mix_hash() {
			return Err(From::from(BlockError::MismatchedH256SealElement(Mismatch { expected: mix, found: header.mix_hash() })));
		}
		if &difficulty < header.difficulty() {
			return Err(From::from(BlockError::InvalidProofOfWork(OutOfBounds { min: Some(header.difficulty().clone()), max: None, found: difficulty })));
		}
		Ok(())
	}

	fn verify_block_family(&self, header: &Header, parent: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		// we should not calculate difficulty for genesis blocks
		if header.number() == 0 {
			return Err(From::from(BlockError::RidiculousNumber(OutOfBounds { min: Some(1), max: None, found: header.number() })));
		}

		// Check difficulty is correct given the two timestamps.
		let expected_difficulty = self.calculate_difficulty(header, parent);
		if header.difficulty() != &expected_difficulty {
			return Err(From::from(BlockError::InvalidDifficulty(Mismatch { expected: expected_difficulty, found: header.difficulty().clone() })))
		}
		let gas_limit_divisor = self.params().gas_limit_bound_divisor;
		let parent_gas_limit = *parent.gas_limit();
		let min_gas = parent_gas_limit - parent_gas_limit / gas_limit_divisor;
		let max_gas = parent_gas_limit + parent_gas_limit / gas_limit_divisor;
		if header.gas_limit() <= &min_gas || header.gas_limit() >= &max_gas {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas), max: Some(max_gas), found: header.gas_limit().clone() })));
		}
		if header.number() >= self.ethash_params.max_gas_limit_transition && header.gas_limit() > &self.ethash_params.max_gas_limit && header.gas_limit() > &parent_gas_limit {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas), max: Some(self.ethash_params.max_gas_limit), found: header.gas_limit().clone() })));
		}
		Ok(())
	}

	fn verify_transaction_basic(&self, t: &UnverifiedTransaction, header: &Header) -> Result<(), Error> {
		if header.number() >= self.ethash_params.min_gas_price_transition && t.gas_price < self.ethash_params.min_gas_price {
			return Err(TransactionError::InsufficientGasPrice { minimal: self.ethash_params.min_gas_price, got: t.gas_price }.into());
		}

		let check_low_s = header.number() >= self.ethash_params.homestead_transition;
		let chain_id = if header.number() >= self.params().eip155_transition { Some(self.params().chain_id) } else { None };
		t.verify_basic(check_low_s, chain_id, false)?;
		Ok(())
	}

	fn verify_transaction(&self, t: UnverifiedTransaction, header: &Header) -> Result<SignedTransaction, Error> {
		let signed = SignedTransaction::new(t)?;
		if !self.tx_filter.as_ref().map_or(true, |filter| filter.transaction_allowed(header.parent_hash(), &signed)) {
			return Err(From::from(TransactionError::NotAllowed));
		}
		Ok(signed)
	}

	fn epoch_verifier<'a>(&self, _header: &Header, _proof: &'a [u8]) -> engines::ConstructedVerifier<'a> {
		engines::ConstructedVerifier::Trusted(Box::new(self.clone()))
	}

	fn snapshot_components(&self) -> Option<Box<::snapshot::SnapshotComponents>> {
		Some(Box::new(::snapshot::PowSnapshot::new(SNAPSHOT_BLOCKS, MAX_SNAPSHOT_BLOCKS)))
	}

	fn register_client(&self, client: Weak<Client>) {
		if let Some(ref filter) = self.tx_filter {
			filter.register_client(client as Weak<BlockChainClient>);
		}
	}

}

// Try to round gas_limit a bit so that:
// 1) it will still be in desired range
// 2) it will be a nearest (with tendency to increase) multiple of PARITY_GAS_LIMIT_DETERMINANT
fn round_block_gas_limit(gas_limit: U256, lower_limit: U256, upper_limit: U256) -> U256 {
	let increased_gas_limit = gas_limit + (PARITY_GAS_LIMIT_DETERMINANT - gas_limit % PARITY_GAS_LIMIT_DETERMINANT);
	if increased_gas_limit > upper_limit {
		let decreased_gas_limit = increased_gas_limit - PARITY_GAS_LIMIT_DETERMINANT;
		if decreased_gas_limit < lower_limit {
			gas_limit
		} else {
			decreased_gas_limit
		}
	} else {
		increased_gas_limit
	}
}

fn ecip1017_eras_block_reward(era_rounds: u64, mut reward: U256, block_number:u64) -> (u64, U256){
	let eras = if block_number != 0 && block_number % era_rounds == 0 {
		block_number / era_rounds - 1
	} else {
		block_number / era_rounds
	};
	for _ in 0..eras {
		reward = reward / U256::from(5) * U256::from(4);
	}
	(eras, reward)
}

#[cfg_attr(feature="dev", allow(wrong_self_convention))]
impl Ethash {
	fn calculate_difficulty(&self, header: &Header, parent: &Header) -> U256 {
		const EXP_DIFF_PERIOD: u64 = 100000;
		if header.number() == 0 {
			panic!("Can't calculate genesis block difficulty");
		}

		let parent_has_uncles = parent.uncles_hash() != &KECCAK_EMPTY_LIST_RLP;

		let min_difficulty = self.ethash_params.minimum_difficulty;

		let difficulty_hardfork = header.number() >= self.ethash_params.difficulty_hardfork_transition;
		let difficulty_bound_divisor = if difficulty_hardfork {
			self.ethash_params.difficulty_hardfork_bound_divisor
		} else {
			self.ethash_params.difficulty_bound_divisor
		};

		let duration_limit = self.ethash_params.duration_limit;
		let frontier_limit = self.ethash_params.homestead_transition;

		let mut target = if header.number() < frontier_limit {
			if header.timestamp() >= parent.timestamp() + duration_limit {
				*parent.difficulty() - (*parent.difficulty() / difficulty_bound_divisor)
			} else {
				*parent.difficulty() + (*parent.difficulty() / difficulty_bound_divisor)
			}
		}
		else {
			trace!(target: "ethash", "Calculating difficulty parent.difficulty={}, header.timestamp={}, parent.timestamp={}", parent.difficulty(), header.timestamp(), parent.timestamp());
			//block_diff = parent_diff + parent_diff // 2048 * max(1 - (block_timestamp - parent_timestamp) // 10, -99)
			let (increment_divisor, threshold) = if header.number() < self.ethash_params.eip100b_transition {
				(self.ethash_params.difficulty_increment_divisor, 1)
			} else if parent_has_uncles {
				(self.ethash_params.metropolis_difficulty_increment_divisor, 2)
			} else {
				(self.ethash_params.metropolis_difficulty_increment_divisor, 1)
			};

			let diff_inc = (header.timestamp() - parent.timestamp()) / increment_divisor;
			if diff_inc <= threshold {
				*parent.difficulty() + *parent.difficulty() / difficulty_bound_divisor * (threshold - diff_inc).into()
			} else {
				let multiplier = cmp::min(diff_inc - threshold, 99).into();
				parent.difficulty().saturating_sub(
					*parent.difficulty() / difficulty_bound_divisor * multiplier
				)
			}
		};
		target = cmp::max(min_difficulty, target);
		if header.number() < self.ethash_params.bomb_defuse_transition {
			if header.number() < self.ethash_params.ecip1010_pause_transition {
				let period = ((parent.number() + 1) / EXP_DIFF_PERIOD) as usize;
				if period > 1 {
					target = cmp::max(min_difficulty, target + (U256::from(1) << (period - 2)));
				}
			}
			else if header.number() < self.ethash_params.ecip1010_continue_transition {
				let fixed_difficulty = ((self.ethash_params.ecip1010_pause_transition / EXP_DIFF_PERIOD) - 2) as usize;
				target = cmp::max(min_difficulty, target + (U256::from(1) << fixed_difficulty));
			}
			else {
				let period = ((parent.number() + 1) / EXP_DIFF_PERIOD) as usize;
				let delay = ((self.ethash_params.ecip1010_continue_transition - self.ethash_params.ecip1010_pause_transition) / EXP_DIFF_PERIOD) as usize;
				target = cmp::max(min_difficulty, target + (U256::from(1) << (period - delay - 2)));
			}
		}
		target
	}

	/// Convert an Ethash boundary to its original difficulty. Basically just `f(x) = 2^256 / x`.
	pub fn boundary_to_difficulty(boundary: &H256) -> U256 {
		let d = U256::from(*boundary);
		if d <= U256::one() {
			U256::max_value()
		} else {
			((U256::one() << 255) / d) << 1
		}
	}

	/// Convert an Ethash difficulty to the target boundary. Basically just `f(x) = 2^256 / x`.
	pub fn difficulty_to_boundary(difficulty: &U256) -> H256 {
		if *difficulty <= U256::one() {
			U256::max_value().into()
		} else {
			(((U256::one() << 255) / *difficulty) << 1).into()
		}
	}
}

impl Header {
	/// Get the none field of the header.
	pub fn nonce(&self) -> H64 {
		rlp::decode(&self.seal()[1])
	}

	/// Get the mix hash field of the header.
	pub fn mix_hash(&self) -> H256 {
		rlp::decode(&self.seal()[0])
	}

	/// Set the nonce and mix hash fields of the header.
	pub fn set_nonce_and_mix_hash(&mut self, nonce: &H64, mix_hash: &H256) {
		self.set_seal(vec![rlp::encode(mix_hash).into_vec(), rlp::encode(nonce).into_vec()]);
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use std::collections::BTreeMap;
	use std::sync::Arc;
	use util::*;
	use block::*;
	use tests::helpers::*;
	use engines::Engine;
	use error::{BlockError, Error};
	use header::Header;
	use spec::Spec;
	use super::super::{new_morden, new_homestead_test};
	use super::{Ethash, EthashParams, PARITY_GAS_LIMIT_DETERMINANT, ecip1017_eras_block_reward};
	use rlp;

	fn test_spec() -> Spec {
		new_morden(&::std::env::temp_dir())
	}

	#[test]
	fn on_close_block() {
		let spec = test_spec();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes, Address::zero(), (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b = b.close();
		assert_eq!(b.state().balance(&Address::zero()).unwrap(), U256::from_str("4563918244f40000").unwrap());
	}

	#[test]
	fn on_close_block_with_uncle() {
		let spec = test_spec();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let mut b = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes, Address::zero(), (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let mut uncle = Header::new();
		let uncle_author: Address = "ef2d6d194084c2de36e0dabfce45d046b37d1106".into();
		uncle.set_author(uncle_author);
		b.push_uncle(uncle).unwrap();

		let b = b.close();
		assert_eq!(b.state().balance(&Address::zero()).unwrap(), "478eae0e571ba000".into());
		assert_eq!(b.state().balance(&uncle_author).unwrap(), "3cb71f51fc558000".into());
	}

	#[test]
	fn has_valid_metadata() {
		let engine = test_spec().engine;
		assert!(!engine.name().is_empty());
		assert!(engine.version().major >= 1);
	}

	#[test]
	fn can_return_schedule() {
		let engine = test_spec().engine;
		let schedule = engine.schedule(10000000);
		assert!(schedule.stack_limit > 0);

		let schedule = engine.schedule(100);
		assert!(!schedule.have_delegate_call);
	}

	#[test]
	fn can_do_seal_verification_fail() {
		let engine = test_spec().engine;
		//let engine = Ethash::new_test(test_spec());
		let header: Header = Header::default();

		let verify_result = engine.verify_block_basic(&header, None);

		match verify_result {
			Err(Error::Block(BlockError::InvalidSealArity(_))) => {},
			Err(_) => { panic!("should be block seal-arity mismatch error (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn can_do_difficulty_verification_fail() {
		let engine = test_spec().engine;
		let mut header: Header = Header::default();
		header.set_seal(vec![rlp::encode(&H256::zero()).into_vec(), rlp::encode(&H64::zero()).into_vec()]);

		let verify_result = engine.verify_block_basic(&header, None);

		match verify_result {
			Err(Error::Block(BlockError::DifficultyOutOfBounds(_))) => {},
			Err(_) => { panic!("should be block difficulty error (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn can_do_proof_of_work_verification_fail() {
		let engine = test_spec().engine;
		let mut header: Header = Header::default();
		header.set_seal(vec![rlp::encode(&H256::zero()).into_vec(), rlp::encode(&H64::zero()).into_vec()]);
		header.set_difficulty(U256::from_str("ffffffffffffffffffffffffffffffffffffffffffffaaaaaaaaaaaaaaaaaaaa").unwrap());

		let verify_result = engine.verify_block_basic(&header, None);

		match verify_result {
			Err(Error::Block(BlockError::InvalidProofOfWork(_))) => {},
			Err(_) => { panic!("should be invalid proof of work error (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn can_do_seal_unordered_verification_fail() {
		let engine = test_spec().engine;
		let header: Header = Header::default();

		let verify_result = engine.verify_block_unordered(&header, None);

		match verify_result {
			Err(Error::Block(BlockError::InvalidSealArity(_))) => {},
			Err(_) => { panic!("should be block seal-arity mismatch error (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn can_do_seal256_verification_fail() {
		let engine = test_spec().engine;
		let mut header: Header = Header::default();
		header.set_seal(vec![rlp::encode(&H256::zero()).into_vec(), rlp::encode(&H64::zero()).into_vec()]);
		let verify_result = engine.verify_block_unordered(&header, None);

		match verify_result {
			Err(Error::Block(BlockError::MismatchedH256SealElement(_))) => {},
			Err(_) => { panic!("should be invalid 256-bit seal fail (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn can_do_proof_of_work_unordered_verification_fail() {
		let engine = test_spec().engine;
		let mut header: Header = Header::default();
		header.set_seal(vec![rlp::encode(&H256::from("b251bd2e0283d0658f2cadfdc8ca619b5de94eca5742725e2e757dd13ed7503d")).into_vec(), rlp::encode(&H64::zero()).into_vec()]);
		header.set_difficulty(U256::from_str("ffffffffffffffffffffffffffffffffffffffffffffaaaaaaaaaaaaaaaaaaaa").unwrap());

		let verify_result = engine.verify_block_unordered(&header, None);

		match verify_result {
			Err(Error::Block(BlockError::InvalidProofOfWork(_))) => {},
			Err(_) => { panic!("should be invalid proof-of-work fail (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn can_verify_block_family_genesis_fail() {
		let engine = test_spec().engine;
		let header: Header = Header::default();
		let parent_header: Header = Header::default();

		let verify_result = engine.verify_block_family(&header, &parent_header, None);

		match verify_result {
			Err(Error::Block(BlockError::RidiculousNumber(_))) => {},
			Err(_) => { panic!("should be invalid block number fail (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn can_verify_block_family_difficulty_fail() {
		let engine = test_spec().engine;
		let mut header: Header = Header::default();
		header.set_number(2);
		let mut parent_header: Header = Header::default();
		parent_header.set_number(1);

		let verify_result = engine.verify_block_family(&header, &parent_header, None);

		match verify_result {
			Err(Error::Block(BlockError::InvalidDifficulty(_))) => {},
			Err(_) => { panic!("should be invalid difficulty fail (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn can_verify_block_family_gas_fail() {
		let engine = test_spec().engine;
		let mut header: Header = Header::default();
		header.set_number(2);
		header.set_difficulty(U256::from_str("0000000000000000000000000000000000000000000000000000000000020000").unwrap());
		let mut parent_header: Header = Header::default();
		parent_header.set_number(1);

		let verify_result = engine.verify_block_family(&header, &parent_header, None);

		match verify_result {
			Err(Error::Block(BlockError::InvalidGasLimit(_))) => {},
			Err(_) => { panic!("should be invalid difficulty fail (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn test_difficulty_to_boundary() {
		// result of f(0) is undefined, so do not assert the result
		let _ = Ethash::difficulty_to_boundary(&U256::from(0));
		assert_eq!(Ethash::difficulty_to_boundary(&U256::from(1)), H256::from(U256::max_value()));
		assert_eq!(Ethash::difficulty_to_boundary(&U256::from(2)), H256::from_str("8000000000000000000000000000000000000000000000000000000000000000").unwrap());
		assert_eq!(Ethash::difficulty_to_boundary(&U256::from(4)), H256::from_str("4000000000000000000000000000000000000000000000000000000000000000").unwrap());
		assert_eq!(Ethash::difficulty_to_boundary(&U256::from(32)), H256::from_str("0800000000000000000000000000000000000000000000000000000000000000").unwrap());
	}

	#[test]
	fn difficulty_frontier() {
		let spec = new_homestead_test();
		let ethparams = get_default_ethash_params();
		let ethash = Ethash::new(&::std::env::temp_dir(), spec.params().clone(), ethparams, BTreeMap::new());

		let mut parent_header = Header::default();
		parent_header.set_number(1000000);
		parent_header.set_difficulty(U256::from_str("b69de81a22b").unwrap());
		parent_header.set_timestamp(1455404053);
		let mut header = Header::default();
		header.set_number(parent_header.number() + 1);
		header.set_timestamp(1455404058);

		let difficulty = ethash.calculate_difficulty(&header, &parent_header);
		assert_eq!(U256::from_str("b6b4bbd735f").unwrap(), difficulty);
	}

	#[test]
	fn difficulty_homestead() {
		let spec = new_homestead_test();
		let ethparams = get_default_ethash_params();
		let ethash = Ethash::new(&::std::env::temp_dir(), spec.params().clone(), ethparams, BTreeMap::new());

		let mut parent_header = Header::default();
		parent_header.set_number(1500000);
		parent_header.set_difficulty(U256::from_str("1fd0fd70792b").unwrap());
		parent_header.set_timestamp(1463003133);
		let mut header = Header::default();
		header.set_number(parent_header.number() + 1);
		header.set_timestamp(1463003177);

		let difficulty = ethash.calculate_difficulty(&header, &parent_header);
		assert_eq!(U256::from_str("1fc50f118efe").unwrap(), difficulty);
	}

	#[test]
	fn has_valid_ecip1017_eras_block_reward() {
		let eras_rounds = 5000000;

		let start_reward: U256 = "4563918244F40000".parse().unwrap();

		let block_number = 0;
		let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, start_reward, block_number);
		assert_eq!(0, eras);
		assert_eq!(U256::from_str("4563918244F40000").unwrap(), reward);

		let block_number = 5000000;
		let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, start_reward, block_number);
		assert_eq!(0, eras);
		assert_eq!(U256::from_str("4563918244F40000").unwrap(), reward);

		let block_number = 10000000;
		let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, start_reward, block_number);
		assert_eq!(1, eras);
		assert_eq!(U256::from_str("3782DACE9D900000").unwrap(), reward);

		let block_number = 20000000;
		let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, start_reward, block_number);
		assert_eq!(3, eras);
		assert_eq!(U256::from_str("2386F26FC1000000").unwrap(), reward);

		let block_number = 80000000;
		let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, start_reward, block_number);
		assert_eq!(15, eras);
		assert_eq!(U256::from_str("271000000000000").unwrap(), reward);
	}

	#[test]
	fn difficulty_classic_bomb_delay() {
		let spec = new_homestead_test();
		let ethparams = EthashParams {
			ecip1010_pause_transition: 3000000,
			..get_default_ethash_params()
		};
		let ethash = Ethash::new(&::std::env::temp_dir(), spec.params().clone(), ethparams, BTreeMap::new());

		let mut parent_header = Header::default();
		parent_header.set_number(3500000);
		parent_header.set_difficulty(U256::from_str("6F62EAF8D3C").unwrap());
		parent_header.set_timestamp(1452838500);
		let mut header = Header::default();
		header.set_number(parent_header.number() + 1);

		header.set_timestamp(parent_header.timestamp() + 20);
		assert_eq!(
			U256::from_str("6F55FE9B74B").unwrap(),
			ethash.calculate_difficulty(&header, &parent_header)
		);
		header.set_timestamp(parent_header.timestamp() + 5);
		assert_eq!(
			U256::from_str("6F71D75632D").unwrap(),
			ethash.calculate_difficulty(&header, &parent_header)
		);
		header.set_timestamp(parent_header.timestamp() + 80);
		assert_eq!(
			U256::from_str("6F02746B3A5").unwrap(),
			ethash.calculate_difficulty(&header, &parent_header)
		);
	}

	#[test]
	fn test_difficulty_bomb_continue() {
		let spec = new_homestead_test();
		let ethparams = EthashParams {
			ecip1010_pause_transition: 3000000,
			ecip1010_continue_transition: 5000000,
			..get_default_ethash_params()
		};
		let ethash = Ethash::new(&::std::env::temp_dir(), spec.params().clone(), ethparams, BTreeMap::new());

		let mut parent_header = Header::default();
		parent_header.set_number(5000102);
		parent_header.set_difficulty(U256::from_str("14944397EE8B").unwrap());
		parent_header.set_timestamp(1513175023);
		let mut header = Header::default();
		header.set_number(parent_header.number() + 1);
		header.set_timestamp(parent_header.timestamp() + 6);
		assert_eq!(
			U256::from_str("1496E6206188").unwrap(),
			ethash.calculate_difficulty(&header, &parent_header)
		);
		parent_header.set_number(5100123);
		parent_header.set_difficulty(U256::from_str("14D24B39C7CF").unwrap());
		parent_header.set_timestamp(1514609324);
		header.set_number(parent_header.number() + 1);
		header.set_timestamp(parent_header.timestamp() + 41);
		assert_eq!(
			U256::from_str("14CA9C5D9227").unwrap(),
			ethash.calculate_difficulty(&header, &parent_header)
		);
		parent_header.set_number(6150001);
		parent_header.set_difficulty(U256::from_str("305367B57227").unwrap());
		parent_header.set_timestamp(1529664575);
		header.set_number(parent_header.number() + 1);
		header.set_timestamp(parent_header.timestamp() + 105);
		assert_eq!(
			U256::from_str("309D09E0C609").unwrap(),
			ethash.calculate_difficulty(&header, &parent_header)
		);
		parent_header.set_number(8000000);
		parent_header.set_difficulty(U256::from_str("1180B36D4CE5B6A").unwrap());
		parent_header.set_timestamp(1535431724);
		header.set_number(parent_header.number() + 1);
		header.set_timestamp(parent_header.timestamp() + 420);
		assert_eq!(
			U256::from_str("5126FFD5BCBB9E7").unwrap(),
			ethash.calculate_difficulty(&header, &parent_header)
		);
	}

	#[test]
	fn gas_limit_is_multiple_of_determinant() {
		let spec = new_homestead_test();
		let ethparams = get_default_ethash_params();
		let ethash = Ethash::new(&::std::env::temp_dir(), spec.params().clone(), ethparams, BTreeMap::new());
		let mut parent = Header::new();
		let mut header = Header::new();
		header.set_number(1);

		// this test will work for this constant only
		assert_eq!(PARITY_GAS_LIMIT_DETERMINANT, U256::from(37));

		// when parent.gas_limit < gas_floor_target:
		parent.set_gas_limit(U256::from(50_000));
		ethash.populate_from_parent(&mut header, &parent, U256::from(100_000), U256::from(200_000));
		assert_eq!(*header.gas_limit(), U256::from(50_024));

		// when parent.gas_limit > gas_ceil_target:
		parent.set_gas_limit(U256::from(250_000));
		ethash.populate_from_parent(&mut header, &parent, U256::from(100_000), U256::from(200_000));
		assert_eq!(*header.gas_limit(), U256::from(249_787));

		// when parent.gas_limit is in miner's range
		header.set_gas_used(U256::from(150_000));
		parent.set_gas_limit(U256::from(150_000));
		ethash.populate_from_parent(&mut header, &parent, U256::from(100_000), U256::from(200_000));
		assert_eq!(*header.gas_limit(), U256::from(150_035));

		// when parent.gas_limit is in miner's range
		// && we can NOT increase it to be multiple of constant
		header.set_gas_used(U256::from(150_000));
		parent.set_gas_limit(U256::from(150_000));
		ethash.populate_from_parent(&mut header, &parent, U256::from(100_000), U256::from(150_002));
		assert_eq!(*header.gas_limit(), U256::from(149_998));

		// when parent.gas_limit is in miner's range
		// && we can NOT increase it to be multiple of constant
		// && we can NOT decrease it to be multiple of constant
		header.set_gas_used(U256::from(150_000));
		parent.set_gas_limit(U256::from(150_000));
		ethash.populate_from_parent(&mut header, &parent, U256::from(150_000), U256::from(150_002));
		assert_eq!(*header.gas_limit(), U256::from(150_002));
	}

	#[test]
	fn difficulty_max_timestamp() {
		let spec = new_homestead_test();
		let ethparams = get_default_ethash_params();
		let ethash = Ethash::new(&::std::env::temp_dir(), spec.params().clone(), ethparams, BTreeMap::new());

		let mut parent_header = Header::default();
		parent_header.set_number(1000000);
		parent_header.set_difficulty(U256::from_str("b69de81a22b").unwrap());
		parent_header.set_timestamp(1455404053);
		let mut header = Header::default();
		header.set_number(parent_header.number() + 1);
		header.set_timestamp(u64::max_value());

		let difficulty = ethash.calculate_difficulty(&header, &parent_header);
		assert_eq!(U256::from(12543204905719u64), difficulty);
	}

	#[test]
	fn rejects_blocks_over_max_gas_limit() {
		let spec = new_homestead_test();
		let mut ethparams = get_default_ethash_params();
		ethparams.max_gas_limit_transition = 10;
		ethparams.max_gas_limit = 100_000.into();

		let mut parent_header = Header::default();
		parent_header.set_number(1);
		parent_header.set_gas_limit(100_000.into());
		let mut header = Header::default();
		header.set_number(parent_header.number() + 1);
		header.set_gas_limit(100_001.into());
		header.set_difficulty(ethparams.minimum_difficulty);
		let ethash = Ethash::new(&::std::env::temp_dir(), spec.params().clone(), ethparams, BTreeMap::new());
		assert!(ethash.verify_block_family(&header, &parent_header, None).is_ok());

		parent_header.set_number(9);
		header.set_number(parent_header.number() + 1);

		parent_header.set_gas_limit(99_999.into());
		header.set_gas_limit(100_000.into());
		assert!(ethash.verify_block_family(&header, &parent_header, None).is_ok());

		parent_header.set_gas_limit(200_000.into());
		header.set_gas_limit(200_000.into());
		assert!(ethash.verify_block_family(&header, &parent_header, None).is_ok());

		parent_header.set_gas_limit(100_000.into());
		header.set_gas_limit(100_001.into());
		assert!(ethash.verify_block_family(&header, &parent_header, None).is_err());

		parent_header.set_gas_limit(200_000.into());
		header.set_gas_limit(200_001.into());
		assert!(ethash.verify_block_family(&header, &parent_header, None).is_err());
	}

	#[test]
	fn rejects_transactions_below_min_gas_price() {
		use ethkey::{Generator, Random};
		use transaction::{Transaction, Action};

		let spec = new_homestead_test();
		let mut ethparams = get_default_ethash_params();
		ethparams.min_gas_price_transition = 10;
		ethparams.min_gas_price = 100000.into();

		let mut header = Header::default();
		header.set_number(1);

		let keypair = Random.generate().unwrap();
		let tx1 = Transaction {
			action: Action::Create,
			value: U256::zero(),
			data: Vec::new(),
			gas: 100_000.into(),
			gas_price: 100_000.into(),
			nonce: U256::zero(),
		}.sign(keypair.secret(), None).into();

		let tx2 = Transaction {
			action: Action::Create,
			value: U256::zero(),
			data: Vec::new(),
			gas: 100_000.into(),
			gas_price: 99_999.into(),
			nonce: U256::zero(),
		}.sign(keypair.secret(), None).into();

		let ethash = Ethash::new(&::std::env::temp_dir(), spec.params().clone(), ethparams, BTreeMap::new());
		assert!(ethash.verify_transaction_basic(&tx1, &header).is_ok());
		assert!(ethash.verify_transaction_basic(&tx2, &header).is_ok());

		header.set_number(10);
		assert!(ethash.verify_transaction_basic(&tx1, &header).is_ok());
		assert!(ethash.verify_transaction_basic(&tx2, &header).is_err());
	}
}
