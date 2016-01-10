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
	fn version(&self) -> SemanticVersion { SemanticVersion::new(1, 0, 0) }
	fn spec(&self) -> &Spec { &self.spec }
	fn evm_schedule(&self, _env_info: &EnvInfo) -> EvmSchedule { EvmSchedule::new_frontier() }

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
	assert_eq!(b.state().balance(&Address::zero()), U256::from_str("4563918244f40000").unwrap());
}