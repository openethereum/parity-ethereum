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
use std::collections::BTreeMap;
use std::sync::Arc;
use hash::{KECCAK_EMPTY_LIST_RLP};
use ethash::{quick_get_difficulty, slow_hash_block_number, EthashManager, OptimizeFor};
use bigint::prelude::U256;
use bigint::hash::{H256, H64};
use util::Address;
use unexpected::{OutOfBounds, Mismatch};
use block::*;
use error::{BlockError, Error};
use header::{Header, BlockNumber};
use engines::{self, Engine};
use ethjson;
use rlp::{self, UntrustedRlp};
use machine::EthereumMachine;
use semantic_version::SemanticVersion;

/// Number of blocks in an ethash snapshot.
// make dependent on difficulty incrment divisor?
const SNAPSHOT_BLOCKS: u64 = 5000;
/// Maximum number of blocks allowed in an ethash snapshot.
const MAX_SNAPSHOT_BLOCKS: u64 = 30000;

const DEFAULT_EIP649_DELAY: u64 = 3_000_000;

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
	/// Transition block for a change of difficulty params (currently just bound_divisor).
	pub difficulty_hardfork_transition: u64,
	/// Difficulty param after the difficulty transition.
	pub difficulty_hardfork_bound_divisor: U256,
	/// Block on which there is no additional difficulty from the exponential bomb.
	pub bomb_defuse_transition: u64,
	/// Number of first block where EIP-100 rules begin.
	pub eip100b_transition: u64,
	/// Number of first block where ECIP-1010 begins.
	pub ecip1010_pause_transition: u64,
	/// Number of first block where ECIP-1010 ends.
	pub ecip1010_continue_transition: u64,
	/// Total block number for one ECIP-1017 era.
	pub ecip1017_era_rounds: u64,
	/// Number of first block where MCIP-3 begins.
	pub mcip3_transition: u64,
	/// MCIP-3 Block reward coin-base for miners.
	pub mcip3_miner_reward: U256,
	/// MCIP-3 Block reward ubi-base for basic income.
	pub mcip3_ubi_reward: U256,
	/// MCIP-3 contract address for universal basic income.
	pub mcip3_ubi_contract: Address,
	/// MCIP-3 Block reward dev-base for dev funds.
	pub mcip3_dev_reward: U256,
	/// MCIP-3 contract address for the developer funds.
	pub mcip3_dev_contract: Address,
	/// Block reward in base units.
	pub block_reward: U256,
	/// EIP-649 transition block.
	pub eip649_transition: u64,
	/// EIP-649 bomb delay.
	pub eip649_delay: u64,
	/// EIP-649 base reward.
	pub eip649_reward: Option<U256>,
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
			difficulty_hardfork_transition: p.difficulty_hardfork_transition.map_or(u64::max_value(), Into::into),
			difficulty_hardfork_bound_divisor: p.difficulty_hardfork_bound_divisor.map_or(p.difficulty_bound_divisor.into(), Into::into),
			bomb_defuse_transition: p.bomb_defuse_transition.map_or(u64::max_value(), Into::into),
			eip100b_transition: p.eip100b_transition.map_or(u64::max_value(), Into::into),
			ecip1010_pause_transition: p.ecip1010_pause_transition.map_or(u64::max_value(), Into::into),
			ecip1010_continue_transition: p.ecip1010_continue_transition.map_or(u64::max_value(), Into::into),
			ecip1017_era_rounds: p.ecip1017_era_rounds.map_or(u64::max_value(), Into::into),
			mcip3_transition: p.mcip3_transition.map_or(u64::max_value(), Into::into),
			mcip3_miner_reward: p.mcip3_miner_reward.map_or_else(Default::default, Into::into),
			mcip3_ubi_reward: p.mcip3_ubi_reward.map_or(U256::from(0), Into::into),
			mcip3_ubi_contract: p.mcip3_ubi_contract.map_or_else(Address::new, Into::into),
			mcip3_dev_reward: p.mcip3_dev_reward.map_or(U256::from(0), Into::into),
			mcip3_dev_contract: p.mcip3_dev_contract.map_or_else(Address::new, Into::into),
			block_reward: p.block_reward.map_or_else(Default::default, Into::into),
			eip649_transition: p.eip649_transition.map_or(u64::max_value(), Into::into),
			eip649_delay: p.eip649_delay.map_or(DEFAULT_EIP649_DELAY, Into::into),
			eip649_reward: p.eip649_reward.map(Into::into),
		}
	}
}

/// Engine using Ethash proof-of-work consensus algorithm, suitable for Ethereum
/// mainnet chains in the Olympic, Frontier and Homestead eras.
pub struct Ethash {
	ethash_params: EthashParams,
	pow: EthashManager,
	machine: EthereumMachine,
}

impl Ethash {
	/// Create a new instance of Ethash engine
	pub fn new<T: Into<Option<OptimizeFor>>>(
		cache_dir: &Path,
		ethash_params: EthashParams,
		machine: EthereumMachine,
		optimize_for: T,
	) -> Arc<Self> {
		Arc::new(Ethash {
			ethash_params,
			machine,
			pow: EthashManager::new(cache_dir.as_ref(), optimize_for.into()),
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
impl engines::EpochVerifier<EthereumMachine> for Arc<Ethash> {
	fn verify_light(&self, _header: &Header) -> Result<(), Error> { Ok(()) }
	fn verify_heavy(&self, header: &Header) -> Result<(), Error> {
		self.verify_block_unordered(header)
	}
}

impl Engine<EthereumMachine> for Arc<Ethash> {
	fn name(&self) -> &str { "Ethash" }
	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }
	fn machine(&self) -> &EthereumMachine { &self.machine }

	// Two fields - nonce and mix.
	fn seal_fields(&self) -> usize { 2 }

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

	fn maximum_uncle_count(&self, _block: BlockNumber) -> usize { 2 }

	fn populate_from_parent(&self, header: &mut Header, parent: &Header) {
		let difficulty = self.calculate_difficulty(header, parent);
		header.set_difficulty(difficulty);
	}

	fn on_new_block(
		&self,
		_block: &mut ExecutedBlock,
		_begins_epoch: bool,
	) -> Result<(), Error> {
		Ok(())
	}

	/// Apply the block reward on finalisation of the block.
	/// This assumes that all uncles are valid uncles (i.e. of at least one generation before the current).
	fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
		use std::ops::Shr;
		use parity_machine::{LiveBlock, WithBalances};

		let author = *LiveBlock::header(&*block).author();
		let number = LiveBlock::header(&*block).number();

		// Applies EIP-649 reward.
		let reward = if number >= self.ethash_params.eip649_transition {
			self.ethash_params.eip649_reward.unwrap_or(self.ethash_params.block_reward)
		} else {
			self.ethash_params.block_reward
		};

		// Applies ECIP-1017 eras.
		let eras_rounds = self.ethash_params.ecip1017_era_rounds;
		let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, reward, number);

		let n_uncles = LiveBlock::uncles(&*block).len();

		// Bestow block rewards.
		let mut result_block_reward = reward + reward.shr(5) * U256::from(n_uncles);
		let mut uncle_rewards = Vec::with_capacity(n_uncles);

		if number >= self.ethash_params.mcip3_transition {
			result_block_reward = self.ethash_params.mcip3_miner_reward;
			let ubi_contract = self.ethash_params.mcip3_ubi_contract;
			let ubi_reward = self.ethash_params.mcip3_ubi_reward;
			let dev_contract = self.ethash_params.mcip3_dev_contract;
			let dev_reward = self.ethash_params.mcip3_dev_reward;

			self.machine.add_balance(block, &author, &result_block_reward)?;
			self.machine.add_balance(block, &ubi_contract, &ubi_reward)?;
			self.machine.add_balance(block, &dev_contract, &dev_reward)?;
		} else {
			self.machine.add_balance(block, &author, &result_block_reward)?;
		}

		// Bestow uncle rewards.
		for u in LiveBlock::uncles(&*block) {
			let uncle_author = u.author();
			let result_uncle_reward = if eras == 0 {
				(reward * U256::from(8 + u.number() - number)).shr(3)
			} else {
				reward.shr(5)
			};

			uncle_rewards.push((*uncle_author, result_uncle_reward));
		}

		for &(ref a, ref reward) in &uncle_rewards {
			self.machine.add_balance(block, a, reward)?;
		}

		// Note and trace.
		self.machine.note_rewards(block, &[(author, result_block_reward)], &uncle_rewards)
	}

	fn verify_local_seal(&self, header: &Header) -> Result<(), Error> {
		self.verify_block_basic(header)
			.and_then(|_| self.verify_block_unordered(header))
	}

	fn verify_block_basic(&self, header: &Header) -> Result<(), Error> {
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

		if header.gas_limit() > &0x7fffffffffffffffu64.into() {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: None, max: Some(0x7fffffffffffffffu64.into()), found: header.gas_limit().clone() })));
		}

		Ok(())
	}

	fn verify_block_unordered(&self, header: &Header) -> Result<(), Error> {
		if header.seal().len() != self.seal_fields() {
			return Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: header.seal().len() }
			)));
		}
		let result = self.pow.compute_light(header.number() as u64, &header.bare_hash().0, header.nonce().low_u64());
		let mix = H256(result.mix_hash);
		let difficulty = Ethash::boundary_to_difficulty(&H256(result.value));
		trace!(target: "miner", "num: {}, seed: {}, h: {}, non: {}, mix: {}, res: {}" , header.number() as u64, H256(slow_hash_block_number(header.number() as u64)), header.bare_hash(), header.nonce().low_u64(), H256(result.mix_hash), H256(result.value));
		if mix != header.mix_hash() {
			return Err(From::from(BlockError::MismatchedH256SealElement(Mismatch { expected: mix, found: header.mix_hash() })));
		}
		if &difficulty < header.difficulty() {
			return Err(From::from(BlockError::InvalidProofOfWork(OutOfBounds { min: Some(header.difficulty().clone()), max: None, found: difficulty })));
		}
		Ok(())
	}

	fn verify_block_family(&self, header: &Header, parent: &Header) -> Result<(), Error> {
		// we should not calculate difficulty for genesis blocks
		if header.number() == 0 {
			return Err(From::from(BlockError::RidiculousNumber(OutOfBounds { min: Some(1), max: None, found: header.number() })));
		}

		// Check difficulty is correct given the two timestamps.
		let expected_difficulty = self.calculate_difficulty(header, parent);
		if header.difficulty() != &expected_difficulty {
			return Err(From::from(BlockError::InvalidDifficulty(Mismatch { expected: expected_difficulty, found: header.difficulty().clone() })))
		}

		Ok(())
	}

	fn epoch_verifier<'a>(&self, _header: &Header, _proof: &'a [u8]) -> engines::ConstructedVerifier<'a, EthereumMachine> {
		engines::ConstructedVerifier::Trusted(Box::new(self.clone()))
	}

	fn snapshot_components(&self) -> Option<Box<::snapshot::SnapshotComponents>> {
		Some(Box::new(::snapshot::PowSnapshot::new(SNAPSHOT_BLOCKS, MAX_SNAPSHOT_BLOCKS)))
	}
}

impl Ethash {
	fn calculate_difficulty(&self, header: &Header, parent: &Header) -> U256 {
		const EXP_DIFF_PERIOD: u64 = 100_000;
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
				let mut number = header.number();
				if number >= self.ethash_params.eip649_transition {
					number = number.saturating_sub(self.ethash_params.eip649_delay);
				}
				let period = (number / EXP_DIFF_PERIOD) as usize;
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
	/// Get the nonce field of the header.
	pub fn nonce(&self) -> H64 {
		rlp::decode(&self.seal()[1])
	}

	/// Get the mix hash field of the header.
	pub fn mix_hash(&self) -> H256 {
		rlp::decode(&self.seal()[0])
	}
}

fn ecip1017_eras_block_reward(era_rounds: u64, mut reward: U256, block_number:u64) -> (u64, U256) {
	let eras = if block_number != 0 && block_number % era_rounds == 0 {
		block_number / era_rounds - 1
	} else {
		block_number / era_rounds
	};
	let mut divi = U256::from(1);
	for _ in 0..eras {
		reward = reward * U256::from(4);
		divi = divi * U256::from(5);
	}
	reward = reward / divi;
	(eras, reward)
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use std::sync::Arc;
	use bigint::prelude::U256;
	use bigint::hash::{H64, H256};
	use util::*;
	use block::*;
	use tests::helpers::*;
	use error::{BlockError, Error};
	use header::Header;
	use spec::Spec;
	use super::super::{new_morden, new_mcip3_test, new_homestead_test_machine};
	use super::{Ethash, EthashParams, ecip1017_eras_block_reward};
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

		let block_number = 250000000;
		let (eras, reward) = ecip1017_eras_block_reward(eras_rounds, start_reward, block_number);
		assert_eq!(49, eras);
		assert_eq!(U256::from_str("51212FFBAF0A").unwrap(), reward);
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
	fn has_valid_mcip3_era_block_rewards() {
		let spec = new_mcip3_test();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let db = spec.ensure_db_good(get_temp_state_db(), &Default::default()).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes, Address::zero(), (3141562.into(), 31415620.into()), vec![], false).unwrap();
		let b = b.close();

		let ubi_contract: Address = "00efdd5883ec628983e9063c7d969fe268bbf310".into();
		let dev_contract: Address = "00756cf8159095948496617f5fb17ed95059f536".into();
		assert_eq!(b.state().balance(&Address::zero()).unwrap(), U256::from_str("d8d726b7177a80000").unwrap());
		assert_eq!(b.state().balance(&ubi_contract).unwrap(), U256::from_str("2b5e3af16b1880000").unwrap());
		assert_eq!(b.state().balance(&dev_contract).unwrap(), U256::from_str("c249fdd327780000").unwrap());
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
		let header: Header = Header::default();

		let verify_result = engine.verify_block_basic(&header);

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

		let verify_result = engine.verify_block_basic(&header);

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

		let verify_result = engine.verify_block_basic(&header);

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

		let verify_result = engine.verify_block_unordered(&header);

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
		let verify_result = engine.verify_block_unordered(&header);

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

		let verify_result = engine.verify_block_unordered(&header);

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

		let verify_result = engine.verify_block_family(&header, &parent_header);

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

		let verify_result = engine.verify_block_family(&header, &parent_header);

		match verify_result {
			Err(Error::Block(BlockError::InvalidDifficulty(_))) => {},
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
		let machine = new_homestead_test_machine();
		let ethparams = get_default_ethash_params();
		let ethash = Ethash::new(&::std::env::temp_dir(), ethparams, machine, None);

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
		let machine = new_homestead_test_machine();
		let ethparams = get_default_ethash_params();
		let ethash = Ethash::new(&::std::env::temp_dir(), ethparams, machine, None);

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
	fn difficulty_classic_bomb_delay() {
		let machine = new_homestead_test_machine();
		let ethparams = EthashParams {
			ecip1010_pause_transition: 3000000,
			..get_default_ethash_params()
		};
		let ethash = Ethash::new(&::std::env::temp_dir(), ethparams, machine, None);

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
		let machine = new_homestead_test_machine();
		let ethparams = EthashParams {
			ecip1010_pause_transition: 3000000,
			ecip1010_continue_transition: 5000000,
			..get_default_ethash_params()
		};
		let ethash = Ethash::new(&::std::env::temp_dir(), ethparams, machine, None);

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
	fn difficulty_max_timestamp() {
		let machine = new_homestead_test_machine();
		let ethparams = get_default_ethash_params();
		let ethash = Ethash::new(&::std::env::temp_dir(), ethparams, machine, None);

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
}
