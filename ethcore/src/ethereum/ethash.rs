// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use ethash::{quick_get_difficulty, slow_get_seedhash, EthashManager, H256 as EH256};
use common::*;
use block::*;
use spec::CommonParams;
use engines::Engine;
use evm::Schedule;
use ethjson;
use rlp::{self, UntrustedRlp, View};

/// Ethash params.
#[derive(Debug, PartialEq)]
pub struct EthashParams {
	/// Gas limit divisor.
	pub gas_limit_bound_divisor: U256,
	/// Minimum difficulty.
	pub minimum_difficulty: U256,
	/// Difficulty bound divisor.
	pub difficulty_bound_divisor: U256,
	/// Difficulty increment divisor.
	pub difficulty_increment_divisor: u64,
	/// Block duration.
	pub duration_limit: u64,
	/// Block reward.
	pub block_reward: U256,
	/// Namereg contract address.
	pub registrar: Address,
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
	/// Bad gas transition block number.
	pub eip150_transition: u64,
}

impl From<ethjson::spec::EthashParams> for EthashParams {
	fn from(p: ethjson::spec::EthashParams) -> Self {
		EthashParams {
			gas_limit_bound_divisor: p.gas_limit_bound_divisor.into(),
			minimum_difficulty: p.minimum_difficulty.into(),
			difficulty_bound_divisor: p.difficulty_bound_divisor.into(),
			difficulty_increment_divisor: p.difficulty_increment_divisor.map_or(10, Into::into),
			duration_limit: p.duration_limit.into(),
			block_reward: p.block_reward.into(),
			registrar: p.registrar.map_or_else(Address::new, Into::into),
			homestead_transition: p.homestead_transition.map_or(0, Into::into),
			dao_hardfork_transition: p.dao_hardfork_transition.map_or(0x7fffffffffffffff, Into::into),
			dao_hardfork_beneficiary: p.dao_hardfork_beneficiary.map_or_else(Address::new, Into::into),
			dao_hardfork_accounts: p.dao_hardfork_accounts.unwrap_or_else(Vec::new).into_iter().map(Into::into).collect(),
			difficulty_hardfork_transition: p.difficulty_hardfork_transition.map_or(0x7fffffffffffffff, Into::into),
			difficulty_hardfork_bound_divisor: p.difficulty_hardfork_bound_divisor.map_or(p.difficulty_bound_divisor.into(), Into::into),
			bomb_defuse_transition: p.bomb_defuse_transition.map_or(0x7fffffffffffffff, Into::into),
			eip150_transition: p.eip150_transition.map_or(0, Into::into),
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
}

impl Ethash {
	/// Create a new instance of Ethash engine
	pub fn new(params: CommonParams, ethash_params: EthashParams, builtins: BTreeMap<Address, Builtin>) -> Self {
		Ethash {
			params: params,
			ethash_params: ethash_params,
			builtins: builtins,
			pow: EthashManager::new(),
		}
	}
}

impl Engine for Ethash {
	fn name(&self) -> &str { "Ethash" }
	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }
	// Two fields - mix
	fn seal_fields(&self) -> usize { 2 }

	fn params(&self) -> &CommonParams { &self.params }
	fn additional_params(&self) -> HashMap<String, String> { hash_map!["registrar".to_owned() => self.ethash_params.registrar.hex()] }

	fn builtins(&self) -> &BTreeMap<Address, Builtin> {
		&self.builtins
	}

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, header: &Header) -> HashMap<String, String> {
		hash_map!["nonce".to_owned() => format!("0x{}", header.nonce().hex()), "mixHash".to_owned() => format!("0x{}", header.mix_hash().hex())]
	}

	fn schedule(&self, env_info: &EnvInfo) -> Schedule {
		trace!(target: "client", "Creating schedule. fCML={}, bGCML={}", self.ethash_params.homestead_transition, self.ethash_params.eip150_transition);

		if env_info.number < self.ethash_params.homestead_transition {
			Schedule::new_frontier()
		} else if env_info.number < self.ethash_params.eip150_transition {
			Schedule::new_homestead()
		} else {
			Schedule::new_homestead_gas_fix()
		}
	}

	fn populate_from_parent(&self, header: &mut Header, parent: &Header, gas_floor_target: U256, gas_ceil_target: U256) {
		let difficulty = self.calculate_difficulty(header, parent);
		let gas_limit = {
			let gas_limit = parent.gas_limit().clone();
			let bound_divisor = self.ethash_params.gas_limit_bound_divisor;
			if gas_limit < gas_floor_target {
				min(gas_floor_target, gas_limit + gas_limit / bound_divisor - 1.into())
			} else if gas_limit > gas_ceil_target {
				max(gas_ceil_target, gas_limit - gas_limit / bound_divisor + 1.into())
			} else {
				min(gas_ceil_target,
					max(gas_floor_target,
						gas_limit - gas_limit / bound_divisor + 1.into() +
							(header.gas_used().clone() * 6.into() / 5.into()) / bound_divisor))
			}
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

	fn on_new_block(&self, block: &mut ExecutedBlock) {
		if block.fields().header.number() == self.ethash_params.dao_hardfork_transition {
			// TODO: enable trigger function maybe?
//			if block.fields().header.gas_limit() <= 4_000_000.into() {
				let mut state = block.fields_mut().state;
				for child in &self.ethash_params.dao_hardfork_accounts {
					let b = state.balance(child);
					state.transfer_balance(child, &self.ethash_params.dao_hardfork_beneficiary, &b);
				}
//			}
		}
	}

	/// Apply the block reward on finalisation of the block.
	/// This assumes that all uncles are valid uncles (i.e. of at least one generation before the current).
	fn on_close_block(&self, block: &mut ExecutedBlock) {
		let reward = self.ethash_params.block_reward;
		let fields = block.fields_mut();

		// Bestow block reward
		fields.state.add_balance(fields.header.author(), &(reward + reward / U256::from(32) * U256::from(fields.uncles.len())));

		// Bestow uncle rewards
		let current_number = fields.header.number();
		for u in fields.uncles.iter() {
			fields.state.add_balance(u.author(), &(reward * U256::from(8 + u.number() - current_number) / U256::from(8)));
		}

		// Commit state so that we can actually figure out the state root.
		if let Err(e) = fields.state.commit() {
			warn!("Encountered error on state commit: {}", e);		
		}		
	}

	fn verify_block_basic(&self, header: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		// check the seal fields.
		if header.seal().len() != self.seal_fields() {
			return Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: header.seal().len() }
			)));
		}
		try!(UntrustedRlp::new(&header.seal()[0]).as_val::<H256>());
		try!(UntrustedRlp::new(&header.seal()[1]).as_val::<H64>());

		// TODO: consider removing these lines.
		let min_difficulty = self.ethash_params.minimum_difficulty;
		if header.difficulty() < &min_difficulty {
			return Err(From::from(BlockError::DifficultyOutOfBounds(OutOfBounds { min: Some(min_difficulty), max: None, found: header.difficulty().clone() })))
		}

		let difficulty = Ethash::boundary_to_difficulty(&Ethash::from_ethash(quick_get_difficulty(
			&Ethash::to_ethash(header.bare_hash()),
			header.nonce().low_u64(),
			&Ethash::to_ethash(header.mix_hash())
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

	fn verify_block_unordered(&self, header: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		if header.seal().len() != self.seal_fields() {
			return Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: header.seal().len() }
			)));
		}
		let result = self.pow.compute_light(header.number() as u64, &Ethash::to_ethash(header.bare_hash()), header.nonce().low_u64());
		let mix = Ethash::from_ethash(result.mix_hash);
		let difficulty = Ethash::boundary_to_difficulty(&Ethash::from_ethash(result.value));
		trace!(target: "miner", "num: {}, seed: {}, h: {}, non: {}, mix: {}, res: {}" , header.number() as u64, Ethash::from_ethash(slow_get_seedhash(header.number() as u64)), header.bare_hash(), header.nonce().low_u64(), Ethash::from_ethash(result.mix_hash), Ethash::from_ethash(result.value));
		if mix != header.mix_hash() {
			return Err(From::from(BlockError::MismatchedH256SealElement(Mismatch { expected: mix, found: header.mix_hash() })));
		}
		if &difficulty < header.difficulty() {
			return Err(From::from(BlockError::InvalidProofOfWork(OutOfBounds { min: Some(header.difficulty().clone()), max: None, found: difficulty })));
		}
		Ok(())
	}

	fn verify_block_family(&self, header: &Header, parent: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		// we should not calculate difficulty for genesis blocks
		if header.number() == 0 {
			return Err(From::from(BlockError::RidiculousNumber(OutOfBounds { min: Some(1), max: None, found: header.number() })));
		}

		// Check difficulty is correct given the two timestamps.
		let expected_difficulty = self.calculate_difficulty(header, parent);
		if header.difficulty() != &expected_difficulty {
			return Err(From::from(BlockError::InvalidDifficulty(Mismatch { expected: expected_difficulty, found: header.difficulty().clone() })))
		}
		let gas_limit_divisor = self.ethash_params.gas_limit_bound_divisor;
		let min_gas = parent.gas_limit().clone() - parent.gas_limit().clone() / gas_limit_divisor;
		let max_gas = parent.gas_limit().clone() + parent.gas_limit().clone() / gas_limit_divisor;
		if header.gas_limit() <= &min_gas || header.gas_limit() >= &max_gas {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas), max: Some(max_gas), found: header.gas_limit().clone() })));
		}
		Ok(())
	}

	fn verify_transaction_basic(&self, t: &SignedTransaction, header: &Header) -> result::Result<(), Error> {
		if header.number() >= self.ethash_params.homestead_transition {
			try!(t.check_low_s());
		}
		Ok(())
	}

	fn verify_transaction(&self, t: &SignedTransaction, _header: &Header) -> Result<(), Error> {
		t.sender().map(|_|()) // Perform EC recovery and cache sender
	}
}

#[cfg_attr(feature="dev", allow(wrong_self_convention))] // to_ethash should take self
impl Ethash {
	fn calculate_difficulty(&self, header: &Header, parent: &Header) -> U256 {
		const EXP_DIFF_PERIOD: u64 = 100000;
		if header.number() == 0 {
			panic!("Can't calculate genesis block difficulty");
		}

		let min_difficulty = self.ethash_params.minimum_difficulty;
		let difficulty_hardfork = header.number() >= self.ethash_params.difficulty_hardfork_transition;
		let difficulty_bound_divisor = match difficulty_hardfork {
			true => self.ethash_params.difficulty_hardfork_bound_divisor,
			false => self.ethash_params.difficulty_bound_divisor,
		};
		let duration_limit = self.ethash_params.duration_limit;
		let frontier_limit = self.ethash_params.homestead_transition;

		let mut target = if header.number() < frontier_limit {
			if header.timestamp() >= parent.timestamp() + duration_limit {
				parent.difficulty().clone() - (parent.difficulty().clone() / difficulty_bound_divisor)
			} else {
				parent.difficulty().clone() + (parent.difficulty().clone() / difficulty_bound_divisor)
			}
		}
		else {
			trace!(target: "ethash", "Calculating difficulty parent.difficulty={}, header.timestamp={}, parent.timestamp={}", parent.difficulty(), header.timestamp(), parent.timestamp());
			//block_diff = parent_diff + parent_diff // 2048 * max(1 - (block_timestamp - parent_timestamp) // 10, -99)
			let diff_inc = (header.timestamp() - parent.timestamp()) / self.ethash_params.difficulty_increment_divisor;
			if diff_inc <= 1 {
				parent.difficulty().clone() + parent.difficulty().clone() / From::from(difficulty_bound_divisor) * From::from(1 - diff_inc)
			} else {
				parent.difficulty().clone() - parent.difficulty().clone() / From::from(difficulty_bound_divisor) * From::from(min(diff_inc - 1, 99))
			}
		};
		target = max(min_difficulty, target);
		if header.number() < self.ethash_params.bomb_defuse_transition {
			let period = ((parent.number() + 1) / EXP_DIFF_PERIOD) as usize;
			if period > 1 {
				target = max(min_difficulty, target + (U256::from(1) << (period - 2)));
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

	fn to_ethash(hash: H256) -> EH256 {
		unsafe { mem::transmute(hash) }
	}

	fn from_ethash(hash: EH256) -> H256 {
		unsafe { mem::transmute(hash) }
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
		self.set_seal(vec![rlp::encode(mix_hash).to_vec(), rlp::encode(nonce).to_vec()]);
	}
}

#[cfg(test)]
mod tests {
	use common::*;
	use block::*;
	use tests::helpers::*;
	use super::super::new_morden;
	use super::Ethash;
	use rlp;

	#[test]
	fn on_close_block() {
		let spec = new_morden();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let mut db_result = get_temp_state_db();
		let mut db = db_result.take();
		spec.ensure_db_good(&mut db).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let b = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes, Address::zero(), (3141562.into(), 31415620.into()), vec![]).unwrap();
		let b = b.close();
		assert_eq!(b.state().balance(&Address::zero()), U256::from_str("4563918244f40000").unwrap());
	}

	#[test]
	fn on_close_block_with_uncle() {
		let spec = new_morden();
		let engine = &*spec.engine;
		let genesis_header = spec.genesis_header();
		let mut db_result = get_temp_state_db();
		let mut db = db_result.take();
		spec.ensure_db_good(&mut db).unwrap();
		let last_hashes = Arc::new(vec![genesis_header.hash()]);
		let mut b = OpenBlock::new(engine, Default::default(), false, db, &genesis_header, last_hashes, Address::zero(), (3141562.into(), 31415620.into()), vec![]).unwrap();
		let mut uncle = Header::new();
		let uncle_author: Address = "ef2d6d194084c2de36e0dabfce45d046b37d1106".into();
		uncle.set_author(uncle_author);
		b.push_uncle(uncle).unwrap();

		let b = b.close();
		assert_eq!(b.state().balance(&Address::zero()), "478eae0e571ba000".into());
		assert_eq!(b.state().balance(&uncle_author), "3cb71f51fc558000".into());
	}

	#[test]
	fn has_valid_metadata() {
		let engine = new_morden().engine;
		assert!(!engine.name().is_empty());
		assert!(engine.version().major >= 1);
	}

	#[test]
	fn can_return_schedule() {
		let engine = new_morden().engine;
		let schedule = engine.schedule(&EnvInfo {
			number: 10000000,
			author: 0.into(),
			timestamp: 0,
			difficulty: 0.into(),
			last_hashes: Arc::new(vec![]),
			gas_used: 0.into(),
			gas_limit: 0.into(),
		});

		assert!(schedule.stack_limit > 0);

		let schedule = engine.schedule(&EnvInfo {
			number: 100,
			author: 0.into(),
			timestamp: 0,
			difficulty: 0.into(),
			last_hashes: Arc::new(vec![]),
			gas_used: 0.into(),
			gas_limit: 0.into(),
		});

		assert!(!schedule.have_delegate_call);
	}

	#[test]
	fn can_do_seal_verification_fail() {
		let engine = new_morden().engine;
		//let engine = Ethash::new_test(new_morden());
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
		let engine = new_morden().engine;
		let mut header: Header = Header::default();
		header.set_seal(vec![rlp::encode(&H256::zero()).to_vec(), rlp::encode(&H64::zero()).to_vec()]);

		let verify_result = engine.verify_block_basic(&header, None);

		match verify_result {
			Err(Error::Block(BlockError::DifficultyOutOfBounds(_))) => {},
			Err(_) => { panic!("should be block difficulty error (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn can_do_proof_of_work_verification_fail() {
		let engine = new_morden().engine;
		let mut header: Header = Header::default();
		header.set_seal(vec![rlp::encode(&H256::zero()).to_vec(), rlp::encode(&H64::zero()).to_vec()]);
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
		let engine = new_morden().engine;
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
		let engine = new_morden().engine;
		let mut header: Header = Header::default();
		header.set_seal(vec![rlp::encode(&H256::zero()).to_vec(), rlp::encode(&H64::zero()).to_vec()]);
		let verify_result = engine.verify_block_unordered(&header, None);

		match verify_result {
			Err(Error::Block(BlockError::MismatchedH256SealElement(_))) => {},
			Err(_) => { panic!("should be invalid 256-bit seal fail (got {:?})", verify_result); },
			_ => { panic!("Should be error, got Ok"); },
		}
	}

	#[test]
	fn can_do_proof_of_work_unordered_verification_fail() {
		let engine = new_morden().engine;
		let mut header: Header = Header::default();
		header.set_seal(vec![rlp::encode(&H256::from("b251bd2e0283d0658f2cadfdc8ca619b5de94eca5742725e2e757dd13ed7503d")).to_vec(), rlp::encode(&H64::zero()).to_vec()]);
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
		let engine = new_morden().engine;
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
		let engine = new_morden().engine;
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
		let engine = new_morden().engine;
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

	// TODO: difficulty test
}
