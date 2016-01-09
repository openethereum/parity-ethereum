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
		block.state_mut().add_balance(&a, &decode(&self.spec().engine_params.get("block_reward").unwrap()));
	}
}

// TODO: test for on_close_block.
#[test]
fn playpen() {
	use super::*;
	use state::*;
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
