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

extern crate ethash;

use self::ethash::{quick_get_difficulty, EthashManager, H256 as EH256};
use common::*;
use block::*;
use spec::CommonParams;
use engine::*;
use evm::{Schedule, Factory};
use ethjson;

/// Ethash params.
#[derive(Debug, PartialEq)]
pub struct EthashParams {
	/// Tie breaking gas.
	pub tie_breaking_gas: bool,
	/// Gas limit divisor.
	pub gas_limit_bound_divisor: U256,
	/// Minimum difficulty.
	pub minimum_difficulty: U256,
	/// Difficulty bound divisor.
	pub difficulty_bound_divisor: U256,
	/// Block duration.
	pub duration_limit: u64,
	/// Block reward.
	pub block_reward: U256,
	/// Namereg contract address.
	pub registrar: Address,
}

impl From<ethjson::spec::EthashParams> for EthashParams {
	fn from(p: ethjson::spec::EthashParams) -> Self {
		EthashParams {
			tie_breaking_gas: p.tie_breaking_gas,
			gas_limit_bound_divisor: p.gas_limit_bound_divisor.into(),
			minimum_difficulty: p.minimum_difficulty.into(),
			difficulty_bound_divisor: p.difficulty_bound_divisor.into(),
			duration_limit: p.duration_limit.into(),
			block_reward: p.block_reward.into(),
			registrar: p.registrar.into(),
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
	factory: Factory,
}

impl Ethash {
	/// Create a new instance of Ethash engine
	pub fn new(params: CommonParams, ethash_params: EthashParams, builtins: BTreeMap<Address, Builtin>) -> Self {
		Ethash {
			params: params,
			ethash_params: ethash_params,
			builtins: builtins,
			pow: EthashManager::new(),
			factory: Factory::default(),
		}
	}
}

impl Engine for Ethash {
	fn name(&self) -> &str { "Ethash" }
	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }
	// Two fields - mix
	fn seal_fields(&self) -> usize { 2 }

	fn params(&self) -> &CommonParams { &self.params }

	fn builtins(&self) -> &BTreeMap<Address, Builtin> {
		&self.builtins
	}

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, _header: &Header) -> HashMap<String, String> { HashMap::new() }

	fn vm_factory(&self) -> &Factory {
		&self.factory
	}

	fn schedule(&self, env_info: &EnvInfo) -> Schedule {
		trace!(target: "client", "Creating schedule. fCML={}", self.params.frontier_compatibility_mode_limit);

		if env_info.number < self.params.frontier_compatibility_mode_limit {
			Schedule::new_frontier()
		} else {
			Schedule::new_homestead()
		}
	}

	fn populate_from_parent(&self, header: &mut Header, parent: &Header, gas_floor_target: U256) {
		header.difficulty = self.calculate_difficuty(header, parent);
		header.gas_limit = {
			let gas_limit = parent.gas_limit;
			let bound_divisor = self.ethash_params.gas_limit_bound_divisor;
			if gas_limit < gas_floor_target {
				min(gas_floor_target, gas_limit + gas_limit / bound_divisor - x!(1))
			} else {
				max(gas_floor_target, gas_limit - gas_limit / bound_divisor + x!(1) + (header.gas_used * x!(6) / x!(5)) / bound_divisor)
			}
		};
		header.note_dirty();
//		info!("ethash: populate_from_parent #{}: difficulty={} and gas_limit={}", header.number, header.difficulty, header.gas_limit);
	}

	/// Apply the block reward on finalisation of the block.
	/// This assumes that all uncles are valid uncles (i.e. of at least one generation before the current).
	fn on_close_block(&self, block: &mut ExecutedBlock) {
		let reward = self.ethash_params.block_reward;
		let fields = block.fields_mut();

		// Bestow block reward
		fields.state.add_balance(&fields.header.author, &(reward + reward / U256::from(32) * U256::from(fields.uncles.len())));

		// Bestow uncle rewards
		let current_number = fields.header.number();
		for u in fields.uncles.iter() {
			fields.state.add_balance(u.author(), &(reward * U256::from(8 + u.number() - current_number) / U256::from(8)));
		}
		fields.state.commit();
	}

	fn verify_block_basic(&self, header: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		// check the seal fields.
		if header.seal.len() != self.seal_fields() {
			return Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: header.seal.len() }
			)));
		}
		try!(UntrustedRlp::new(&header.seal[0]).as_val::<H256>());
		try!(UntrustedRlp::new(&header.seal[1]).as_val::<H64>());

		// TODO: consider removing these lines.
		let min_difficulty = self.ethash_params.minimum_difficulty;
		if header.difficulty < min_difficulty {
			return Err(From::from(BlockError::DifficultyOutOfBounds(OutOfBounds { min: Some(min_difficulty), max: None, found: header.difficulty })))
		}

		let difficulty = Ethash::boundary_to_difficulty(&Ethash::from_ethash(quick_get_difficulty(
			&Ethash::to_ethash(header.bare_hash()),
			header.nonce().low_u64(),
			&Ethash::to_ethash(header.mix_hash())
		)));
		if difficulty < header.difficulty {
			return Err(From::from(BlockError::InvalidProofOfWork(OutOfBounds { min: Some(header.difficulty), max: None, found: difficulty })));
		}
		Ok(())
	}

	fn verify_block_unordered(&self, header: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		if header.seal.len() != self.seal_fields() {
			return Err(From::from(BlockError::InvalidSealArity(
				Mismatch { expected: self.seal_fields(), found: header.seal.len() }
			)));
		}
		let result = self.pow.compute_light(header.number as u64, &Ethash::to_ethash(header.bare_hash()), header.nonce().low_u64());
		let mix = Ethash::from_ethash(result.mix_hash);
		let difficulty = Ethash::boundary_to_difficulty(&Ethash::from_ethash(result.value));
		if mix != header.mix_hash() {
			return Err(From::from(BlockError::MismatchedH256SealElement(Mismatch { expected: mix, found: header.mix_hash() })));
		}
		if difficulty < header.difficulty {
			return Err(From::from(BlockError::InvalidProofOfWork(OutOfBounds { min: Some(header.difficulty), max: None, found: difficulty })));
		}
		Ok(())
	}

	fn verify_block_family(&self, header: &Header, parent: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		// we should not calculate difficulty for genesis blocks
		if header.number() == 0 {
			return Err(From::from(BlockError::RidiculousNumber(OutOfBounds { min: Some(1), max: None, found: header.number() })));
		}

		// Check difficulty is correct given the two timestamps.
		let expected_difficulty = self.calculate_difficuty(header, parent);
		if header.difficulty != expected_difficulty {
			return Err(From::from(BlockError::InvalidDifficulty(Mismatch { expected: expected_difficulty, found: header.difficulty })))
		}
		let gas_limit_divisor = self.ethash_params.gas_limit_bound_divisor;
		let min_gas = parent.gas_limit - parent.gas_limit / gas_limit_divisor;
		let max_gas = parent.gas_limit + parent.gas_limit / gas_limit_divisor;
		if header.gas_limit <= min_gas || header.gas_limit >= max_gas {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas), max: Some(max_gas), found: header.gas_limit })));
		}
		Ok(())
	}

	fn verify_transaction_basic(&self, t: &SignedTransaction, header: &Header) -> result::Result<(), Error> {
		if header.number() >= self.params.frontier_compatibility_mode_limit {
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
	fn calculate_difficuty(&self, header: &Header, parent: &Header) -> U256 {
		const EXP_DIFF_PERIOD: u64 = 100000;
		if header.number == 0 {
			panic!("Can't calculate genesis block difficulty");
		}

		let min_difficulty = self.ethash_params.minimum_difficulty;
		let difficulty_bound_divisor = self.ethash_params.difficulty_bound_divisor;
		let duration_limit = self.ethash_params.duration_limit;
		let frontier_limit = self.params.frontier_compatibility_mode_limit;

		let mut target = if header.number < frontier_limit {
			if header.timestamp >= parent.timestamp + duration_limit {
				parent.difficulty - (parent.difficulty / difficulty_bound_divisor)
			}
			else {
				parent.difficulty + (parent.difficulty / difficulty_bound_divisor)
			}
		}
		else {
			trace!(target: "ethash", "Calculating difficulty parent.difficulty={}, header.timestamp={}, parent.timestamp={}", parent.difficulty, header.timestamp, parent.timestamp);
			//block_diff = parent_diff + parent_diff // 2048 * max(1 - (block_timestamp - parent_timestamp) // 10, -99)
			let diff_inc = (header.timestamp - parent.timestamp) / 10;
			if diff_inc <= 1 {
				parent.difficulty + parent.difficulty / From::from(2048) * From::from(1 - diff_inc)
			} else {
				parent.difficulty - parent.difficulty / From::from(2048) * From::from(min(diff_inc - 1, 99))
			}
		};
		target = max(min_difficulty, target);
		let period = ((parent.number + 1) / EXP_DIFF_PERIOD) as usize;
		if period > 1 {
			target = max(min_difficulty, target + (U256::from(1) << (period - 2)));
		}
		target
	}

	/// Convert an Ethash boundary to its original difficulty. Basically just `f(x) = 2^256 / x`.
	pub fn boundary_to_difficulty(boundary: &H256) -> U256 {
		U256::from((U512::one() << 256) / x!(U256::from(boundary.as_slice())))
	}

	/// Convert an Ethash difficulty to the target boundary. Basically just `f(x) = 2^256 / x`.
	pub fn difficulty_to_boundary(difficulty: &U256) -> H256 {
		x!(U256::from((U512::one() << 256) / x!(difficulty)))
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
		decode(&self.seal()[1])
	}

	/// Get the mix hash field of the header.
	pub fn mix_hash(&self) -> H256 {
		decode(&self.seal()[0])
	}

	/// Set the nonce and mix hash fields of the header.
	pub fn set_nonce_and_mix_hash(&mut self, nonce: &H64, mix_hash: &H256) {
		self.seal = vec![encode(mix_hash).to_vec(), encode(nonce).to_vec()];
	}
}

#[cfg(test)]
mod tests {
	extern crate ethash;

	use common::*;
	use block::*;
	use engine::*;
	use tests::helpers::*;
	use super::super::new_morden;

	#[test]
	fn on_close_block() {
		let spec = new_morden();
		let engine = &spec.engine;
		let genesis_header = spec.genesis_header();
		let mut db_result = get_temp_journal_db();
		let mut db = db_result.take();
		spec.ensure_db_good(db.as_hashdb_mut());
		let last_hashes = vec![genesis_header.hash()];
		let b = OpenBlock::new(engine.deref(), false, db, &genesis_header, last_hashes, Address::zero(), x!(3141562), vec![]);
		let b = b.close();
		assert_eq!(b.state().balance(&Address::zero()), U256::from_str("4563918244f40000").unwrap());
	}

	#[test]
	fn on_close_block_with_uncle() {
		let spec = new_morden();
		let engine = &spec.engine;
		let genesis_header = spec.genesis_header();
		let mut db_result = get_temp_journal_db();
		let mut db = db_result.take();
		spec.ensure_db_good(db.as_hashdb_mut());
		let last_hashes = vec![genesis_header.hash()];
		let mut b = OpenBlock::new(engine.deref(), false, db, &genesis_header, last_hashes, Address::zero(), x!(3141562), vec![]);
		let mut uncle = Header::new();
		let uncle_author = address_from_hex("ef2d6d194084c2de36e0dabfce45d046b37d1106");
		uncle.author = uncle_author.clone();
		b.push_uncle(uncle).unwrap();

		let b = b.close();
		assert_eq!(b.state().balance(&Address::zero()), U256::from_str("478eae0e571ba000").unwrap());
		assert_eq!(b.state().balance(&uncle_author), U256::from_str("3cb71f51fc558000").unwrap());
	}

	#[test]
	fn has_valid_metadata() {
		let engine = new_morden().engine;
		assert!(!engine.name().is_empty());
		assert!(engine.version().major >= 1);
	}

	#[test]
	fn can_return_factory() {
		let engine = new_morden().engine;
		engine.vm_factory();
	}

	#[test]
	fn can_return_schedule() {
		let engine = new_morden().engine;
		let schedule = engine.schedule(&EnvInfo {
			number: 10000000,
			author: x!(0),
			timestamp: 0,
			difficulty: x!(0),
			last_hashes: vec![],
			gas_used: x!(0),
			gas_limit: x!(0)
		});

		assert!(schedule.stack_limit > 0);

		let schedule = engine.schedule(&EnvInfo {
			number: 100,
			author: x!(0),
			timestamp: 0,
			difficulty: x!(0),
			last_hashes: vec![],
			gas_used: x!(0),
			gas_limit: x!(0)
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

	// TODO: difficulty test
}
