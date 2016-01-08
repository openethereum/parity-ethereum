//use util::error::*;
use util::rlp::decode;
use engine::Engine;
use spec::Spec;
use block::*;
use evm_schedule::EvmSchedule;
use env_info::EnvInfo;

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
