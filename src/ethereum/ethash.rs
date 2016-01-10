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
		let reward = self.spec().engine_params.get("blockReward").map(|a| decode(&a)).unwrap_or(U256::from(0u64));
		let fields = block.fields();
		fields.state.add_balance(&fields.header.author, &reward);
/*
		let uncle_authors = block.uncles.iter().map(|u| u.author().clone()).collect();
		for a in uncle_authors {
			block.state_mut().addBalance(a, _blockReward * (8 + i.number() - m_currentBlock.number()) / 8);
			r += _blockReward / 32;
		}*/
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