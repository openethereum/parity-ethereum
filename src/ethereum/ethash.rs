use common::*;
use block::*;
use spec::*;
use engine::*;
use verification::*;

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

	fn verify_block(&self, mode: VerificationMode, header: &Header, parent: Option<&Header>, block: Option<&[u8]>) -> Result<(), VerificationError> { 
		if mode == VerificationMode::Quick {
			let min_difficulty = decode(self.spec().engine_params.get("minimumDifficulty").unwrap());
			if header.difficulty < min_difficulty {
				return Err(VerificationError::block(
					BlockVerificationError::InvalidDifficulty { required: min_difficulty, got: header.difficulty }, 
					block.map(|b| b.to_vec())));
			}
			let min_gas_limit = decode(self.spec().engine_params.get("minGasLimit").unwrap());
			if header.gas_limit < min_gas_limit {
				return Err(VerificationError::block(
					BlockVerificationError::InvalidGasLimit { min: min_gas_limit, max: From::from(0), got: header.gas_limit }, 
					block.map(|b| b.to_vec())));
			}
			let len: U256 = From::from(header.extra_data.len());
			let maximum_extra_data_size: U256 = From::from(self.maximum_extra_data_size());
			if header.number != From::from(0) && len > maximum_extra_data_size {
				return Err(VerificationError::block(
					BlockVerificationError::ExtraDataTooBig { required: maximum_extra_data_size, got: len }, 
					block.map(|b| b.to_vec())));
			}
			match parent {
				Some(p) => {
					// Check difficulty is correct given the two timestamps.
					let expected_difficulty = self.calculate_difficuty(header, p);
					if header.difficulty != expected_difficulty {
						return Err(VerificationError::block(
							BlockVerificationError::InvalidDifficulty { required: expected_difficulty, got: header.difficulty }, 
							block.map(|b| b.to_vec())));
					}
					let gas_limit_divisor = decode(self.spec().engine_params.get("gasLimitBoundDivisor").unwrap());
					let min_gas = p.gas_limit - p.gas_limit / gas_limit_divisor;
					let max_gas = p.gas_limit + p.gas_limit / gas_limit_divisor;
					if header.gas_limit <= min_gas || header.gas_limit >= max_gas {
						return Err(VerificationError::block(
							BlockVerificationError::InvalidGasLimit { min: min_gas_limit, max: max_gas, got: header.gas_limit }, 
							block.map(|b| b.to_vec())));
					}
				},
				None => ()
			}
			// TODO: Verify seal
		}
		Ok(())
	}

	fn verify_transaction(&self, _t: &Transaction, _header: &Header) -> Result<(), VerificationError> { Ok(()) }
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
	assert!(SecTrieDB::new(&db, &genesis_header.state_root).contains(&address_from_hex("102e61f5d8f9bc71d0ad4a084df4e65e05ce0e1c")));
	{
		let s = State::from_existing(db.clone(), genesis_header.state_root.clone(), engine.account_start_nonce());
		assert_eq!(s.balance(&address_from_hex("0000000000000000000000000000000000000001")), U256::from(1u64));
	}
	let b = OpenBlock::new(engine.deref(), db, &genesis_header, vec![genesis_header.hash()]);
//	let c = b.close();
}

