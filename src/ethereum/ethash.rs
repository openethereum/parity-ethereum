use common::*;
use block::*;
use spec::*;
use engine::*;
use evm::Schedule;

/// Engine using Ethash proof-of-work consensus algorithm, suitable for Ethereum
/// mainnet chains in the Olympic, Frontier and Homestead eras.
pub struct Ethash {
	spec: Spec,
}

impl Ethash {
	pub fn new_boxed(spec: Spec) -> Box<Engine> {
		Box::new(Ethash{spec: spec})
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
	fn schedule(&self, _env_info: &EnvInfo) -> Schedule { Schedule::new_frontier() }

	/// Apply the block reward on finalisation of the block.
	/// This assumes that all uncles are valid uncles (i.e. of at least one generation before the current).
	fn on_close_block(&self, block: &mut Block) {
		let reward = self.spec().engine_params.get("blockReward").map(|a| decode(&a)).unwrap_or(U256::from(0u64));
		let fields = block.fields();

		// Bestow block reward
		fields.state.add_balance(&fields.header.author, &(reward + reward / U256::from(32) * U256::from(fields.uncles.len())));

		// Bestow uncle rewards
		let current_number = fields.header.number();
		for u in fields.uncles.iter() {
			fields.state.add_balance(u.author(), &(reward * U256::from((8 + u.number() - current_number) / 8)));
		}
	}


	fn verify_block_basic(&self, header: &Header,  _block: Option<&[u8]>) -> result::Result<(), Error> {
		let min_difficulty = decode(self.spec().engine_params.get("minimumDifficulty").unwrap());
		if header.difficulty < min_difficulty {
			return Err(From::from(BlockError::InvalidDifficulty(Mismatch { expected: min_difficulty, found: header.difficulty })))
		}
		// TODO: Verify seal (quick)
		Ok(())
	}

	fn verify_block_unordered(&self, _header: &Header, _block: Option<&[u8]>) -> result::Result<(), Error> {
		// TODO: Verify seal (full)
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

	fn verify_transaction(&self, _t: &Transaction, _header: &Header) -> result::Result<(), Error> { Ok(()) }
}

impl Ethash {
	fn calculate_difficuty(&self, header: &Header, parent: &Header) -> U256 {
		const EXP_DIFF_PERIOD: u64 = 100000;
		if header.number == 0 {
			panic!("Can't calculate genesis block difficulty");
		}

		let min_difficulty = decode(self.spec().engine_params.get("minimumDifficulty").unwrap());
		let difficulty_bound_divisor = decode(self.spec().engine_params.get("difficultyBoundDivisor").unwrap());
		let duration_limit: u64 = decode(self.spec().engine_params.get("durationLimit").unwrap());
		let frontier_limit = decode(self.spec().engine_params.get("frontierCompatibilityModeLimit").unwrap());
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

// TODO: difficulty test
