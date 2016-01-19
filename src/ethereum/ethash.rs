extern crate ethash;

use self::ethash::{quick_get_difficulty, EthashManager, H256 as EH256};
use common::*;
use block::*;
use spec::*;
use engine::*;
use evm::Schedule;
use evm::Factory;

/// Engine using Ethash proof-of-work consensus algorithm, suitable for Ethereum
/// mainnet chains in the Olympic, Frontier and Homestead eras.
pub struct Ethash {
	spec: Spec,
	pow: EthashManager,
	factory: Factory,
	u64_params: RwLock<HashMap<String, u64>>,
	u256_params: RwLock<HashMap<String, U256>>,
}

impl Ethash {
	pub fn new_boxed(spec: Spec) -> Box<Engine> {
		Box::new(Ethash {
			spec: spec,
			pow: EthashManager::new(),
			// TODO [todr] should this return any specific factory?
			factory: Factory::default(),
			u64_params: RwLock::new(HashMap::new()),
			u256_params: RwLock::new(HashMap::new())
		})
	}

	fn u64_param(&self, name: &str) -> u64 {
		*self.u64_params.write().unwrap().entry(name.to_owned()).or_insert_with(||
			self.spec().engine_params.get(name).map_or(0u64, |a| decode(&a)))
	}

	fn u256_param(&self, name: &str) -> U256 {
		*self.u256_params.write().unwrap().entry(name.to_owned()).or_insert_with(||
			self.spec().engine_params.get(name).map_or(x!(0), |a| decode(&a)))
	}
}

impl Engine for Ethash {
	fn name(&self) -> &str { "Ethash" }
	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }
	// Two fields - mix
	fn seal_fields(&self) -> usize { 2 }
	// Two empty data items in RLP.
	fn seal_rlp(&self) -> Bytes { encode(&H64::new()) }

	/// Additional engine-specific information for the user/developer concerning `header`.
	fn extra_info(&self, _header: &Header) -> HashMap<String, String> { HashMap::new() }
	fn spec(&self) -> &Spec { &self.spec }

	fn vm_factory(&self) -> &Factory {
		&self.factory
	}

	fn schedule(&self, env_info: &EnvInfo) -> Schedule {
		match env_info.number < self.u64_param("frontierCompatibilityModeLimit") {
			true => Schedule::new_frontier(),
			_ => Schedule::new_homestead(),
		}
	}

	fn populate_from_parent(&self, header: &mut Header, parent: &Header) {
		header.difficulty = self.calculate_difficuty(header, parent);
		header.gas_limit = {
			let gas_floor_target: U256 = x!(3141562);
			let gas_limit = parent.gas_limit;
			let bound_divisor = self.u256_param("gasLimitBoundDivisor");
			if gas_limit < gas_floor_target {
				min(gas_floor_target, gas_limit + gas_limit / bound_divisor - x!(1))
			} else {
				max(gas_floor_target, gas_limit - gas_limit / bound_divisor + x!(1) + (header.gas_used * x!(6) / x!(5)) / bound_divisor)
			}
		};

//		info!("ethash: populate_from_parent #{}: difficulty={} and gas_limit={}", header.number, header.difficulty, header.gas_limit);
	}

	/// Apply the block reward on finalisation of the block.
	/// This assumes that all uncles are valid uncles (i.e. of at least one generation before the current).
	fn on_close_block(&self, block: &mut Block) {
		let reward = self.spec().engine_params.get("blockReward").map_or(U256::from(0u64), |a| decode(&a));
		let fields = block.fields();

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
		let min_difficulty = decode(self.spec().engine_params.get("minimumDifficulty").unwrap());
		if header.difficulty < min_difficulty {
			return Err(From::from(BlockError::InvalidDifficulty(Mismatch { expected: min_difficulty, found: header.difficulty })))
		}
		let difficulty = Ethash::boundary_to_difficulty(&Ethash::from_ethash(quick_get_difficulty(
				&Ethash::to_ethash(header.bare_hash()), 
				header.nonce(),
				&Ethash::to_ethash(header.mix_hash()))));
		if difficulty < header.difficulty {
			return Err(From::from(BlockError::InvalidEthashDifficulty(Mismatch { expected: header.difficulty, found: difficulty })));
		}
		Ok(())
	}

	fn verify_block_unordered(&self, header: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		let result = self.pow.compute_light(header.number as u64, &Ethash::to_ethash(header.bare_hash()), header.nonce());
		let mix = Ethash::from_ethash(result.mix_hash);
		let difficulty = Ethash::boundary_to_difficulty(&Ethash::from_ethash(result.value));
		if mix != header.mix_hash() {
			return Err(From::from(BlockError::InvalidBlockNonce(Mismatch { expected: mix, found: header.mix_hash() })));
		}
		if difficulty < header.difficulty {
			return Err(From::from(BlockError::InvalidEthashDifficulty(Mismatch { expected: header.difficulty, found: difficulty })));
		}
		Ok(())
	}

	fn verify_block_family(&self, header: &Header, parent: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		// Check difficulty is correct given the two timestamps.
		let expected_difficulty = self.calculate_difficuty(header, parent);
		if header.difficulty != expected_difficulty {
			return Err(From::from(BlockError::InvalidDifficulty(Mismatch { expected: expected_difficulty, found: header.difficulty })))
		}
		let gas_limit_divisor = decode(self.spec().engine_params.get("gasLimitBoundDivisor").unwrap());
		let min_gas = parent.gas_limit - parent.gas_limit / gas_limit_divisor;
		let max_gas = parent.gas_limit + parent.gas_limit / gas_limit_divisor;
		if header.gas_limit <= min_gas || header.gas_limit >= max_gas {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: Some(min_gas), max: Some(max_gas), found: header.gas_limit }))); 
		}
		Ok(())
	}

	fn verify_transaction_basic(&self, t: &Transaction, header: &Header) -> result::Result<(), Error> {
		if header.number() >= self.u64_param("frontierCompatibilityModeLimit") {
			try!(t.check_low_s());
		}
		Ok(())
	}

	fn verify_transaction(&self, t: &Transaction, _header: &Header) -> Result<(), Error> {
		t.sender().map(|_|()) // Perform EC recovery and cache sender
	}
}

impl Ethash {
	fn calculate_difficuty(&self, header: &Header, parent: &Header) -> U256 {
		const EXP_DIFF_PERIOD: u64 = 100000;
		if header.number == 0 {
			panic!("Can't calculate genesis block difficulty");
		}

		let min_difficulty = self.u256_param("minimumDifficulty");
		let difficulty_bound_divisor = self.u256_param("difficultyBoundDivisor");
		let duration_limit = self.u64_param("durationLimit");
		let frontier_limit = self.u64_param("frontierCompatibilityModeLimit");
		let mut target = if header.number < frontier_limit {
			if header.timestamp >= parent.timestamp + duration_limit {
				parent.difficulty - (parent.difficulty / difficulty_bound_divisor) 
			} 
			else {
				parent.difficulty + (parent.difficulty / difficulty_bound_divisor)
			}
		}
		else {
			let diff_inc = (header.timestamp - parent.timestamp) / 10;
			if diff_inc <= 1 {
				parent.difficulty + parent.difficulty / From::from(2048) * From::from(1 - diff_inc)
			}
			else {
				parent.difficulty - parent.difficulty / From::from(2048) * From::from(max(diff_inc - 1, 99))
			}
		};
		target = max(min_difficulty, target);
		let period = ((parent.number + 1) / EXP_DIFF_PERIOD) as usize;
		if period > 1 {
			target = max(min_difficulty, target + (U256::from(1) << (period - 2)));
		}
		target
	}
	
	fn boundary_to_difficulty(boundary: &H256) -> U256 {
		U256::from((U512::one() << 256) / x!(U256::from(boundary.as_slice())))
	}

	fn to_ethash(hash: H256) -> EH256 {
		unsafe { mem::transmute(hash) }
	}

	fn from_ethash(hash: EH256) -> H256 {
		unsafe { mem::transmute(hash) }
	}
}

impl Header {
	fn nonce(&self) -> u64 {
		decode(&self.seal()[1])
	}
	fn mix_hash(&self) -> H256 {
		decode(&self.seal()[0])
	}
}

#[test]
fn on_close_block() {
	use super::*;
	let engine = new_morden().to_engine().unwrap();
	let genesis_header = engine.spec().genesis_header();
	let mut db = OverlayDB::new_temp();
	engine.spec().ensure_db_good(&mut db);
	let last_hashes = vec![genesis_header.hash()];
	let b = OpenBlock::new(engine.deref(), db, &genesis_header, &last_hashes, Address::zero(), vec![]);
	let b = b.close();
	assert_eq!(b.state().balance(&Address::zero()), U256::from_str("4563918244f40000").unwrap());
}

#[test]
fn on_close_block_with_uncle() {
	use super::*;
	let engine = new_morden().to_engine().unwrap();
	let genesis_header = engine.spec().genesis_header();
	let mut db = OverlayDB::new_temp();
	engine.spec().ensure_db_good(&mut db);
	let last_hashes = vec![genesis_header.hash()];
	let mut b = OpenBlock::new(engine.deref(), db, &genesis_header, &last_hashes, Address::zero(), vec![]);
	let mut uncle = Header::new();
	let uncle_author = address_from_hex("ef2d6d194084c2de36e0dabfce45d046b37d1106");
	uncle.author = uncle_author.clone();
	b.push_uncle(uncle).unwrap();
	
	let b = b.close();
	assert_eq!(b.state().balance(&Address::zero()), U256::from_str("478eae0e571ba000").unwrap());
	assert_eq!(b.state().balance(&uncle_author), U256::from_str("3cb71f51fc558000").unwrap());
}

// TODO: difficulty test
