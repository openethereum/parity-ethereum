use common::*;
use block::*;
use spec::*;
use engine::*;

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
	fn spec(&self) -> &Spec { &self.spec }
	fn evm_schedule(&self, _env_info: &EnvInfo) -> EvmSchedule { EvmSchedule::new_frontier() }

	/// Apply the block reward on finalisation of the block.
	fn on_close_block(&self, block: &mut Block) {
		let a = block.header().author.clone();
		block.state_mut().add_balance(&a, &decode(&self.spec().engine_params.get("blockReward").unwrap()));
	}


	fn verify_block_basic(&self, header: &Header,  _block: Option<&[u8]>) -> Result<(), Error> {
		let min_difficulty = decode(self.spec().engine_params.get("minimumDifficulty").unwrap());
		if header.difficulty < min_difficulty {
			return Err(From::from(BlockError::InvalidDifficulty(Mismatch { expected: min_difficulty, found: header.difficulty })))
		}
		let min_gas_limit = decode(self.spec().engine_params.get("minGasLimit").unwrap());
		if header.gas_limit < min_gas_limit {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: min_gas_limit, max: From::from(0), found: header.gas_limit }))); 
		}
		let maximum_extra_data_size = self.maximum_extra_data_size();
		if header.number != From::from(0) && header.extra_data.len() > maximum_extra_data_size {
			return Err(From::from(BlockError::ExtraDataOutOfBounds(OutOfBounds { min: 0, max: maximum_extra_data_size, found: header.extra_data.len() }))); 
		}
		// TODO: Verify seal (quick)
		Ok(())
	}

	fn verify_block_unordered(&self, _header: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		// TODO: Verify seal (full)
		Ok(())
	}

	fn verify_block_final(&self, header: &Header, parent: &Header, _block: Option<&[u8]>) -> Result<(), Error> {
		// Check difficulty is correct given the two timestamps.
		let expected_difficulty = self.calculate_difficuty(header, parent);
		if header.difficulty != expected_difficulty {
			return Err(From::from(BlockError::InvalidDifficulty(Mismatch { expected: expected_difficulty, found: header.difficulty })))
		}
		let gas_limit_divisor = decode(self.spec().engine_params.get("gasLimitBoundDivisor").unwrap());
		let min_gas = parent.gas_limit - parent.gas_limit / gas_limit_divisor;
		let max_gas = parent.gas_limit + parent.gas_limit / gas_limit_divisor;
		if header.gas_limit <= min_gas || header.gas_limit >= max_gas {
			return Err(From::from(BlockError::InvalidGasLimit(OutOfBounds { min: min_gas, max: max_gas, found: header.gas_limit }))); 
		}
		Ok(())
	}

	fn verify_transaction(&self, _t: &Transaction, _header: &Header) -> Result<(), Error> { Ok(()) }
}

impl Ethash {
	fn calculate_difficuty(&self, header: &Header, parent: &Header) -> U256 {
		const EXP_DIFF_PERIOD: u64 = 100000;
		if header.number == From::from(0) {
			panic!("Can't calculate genesis block difficulty");
		}

		let min_difficulty = decode(self.spec().engine_params.get("minimumDifficulty").unwrap());
		let difficulty_bound_divisor = decode(self.spec().engine_params.get("difficultyBoundDivisor").unwrap());
		let duration_limit = decode(self.spec().engine_params.get("durationLimit").unwrap());
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
			let diff_inc = (header.timestamp - parent.timestamp) / From::from(10);
			if diff_inc <= From::from(1) {
				parent.difficulty + parent.difficulty / From::from(2048) * (U256::from(1) - diff_inc)
			}
			else {
				parent.difficulty - parent.difficulty / From::from(2048) * max(diff_inc - From::from(1), From::from(99))
			}
		};
		target = max(min_difficulty, target);
		let period = ((parent.number + From::from(1)).as_u64() / EXP_DIFF_PERIOD) as usize;
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
	let b = OpenBlock::new(engine.deref(), db, &genesis_header, vec![genesis_header.hash()], Address::zero(), vec![]);
	let b = b.close();
	assert_eq!(b.state().balance(&Address::zero()), U256::from_str("4563918244F40000").unwrap());
}

// TODO: difficulty test
