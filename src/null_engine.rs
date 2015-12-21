use engine::Engine;
use spec::Spec;
use evm_schedule::EvmSchedule;
use env_info::EnvInfo;

/// An engine which does not provide any consensus mechanism.
pub struct NullEngine {
	spec: Spec,
}

impl NullEngine {
	pub fn new_boxed(spec: Spec) -> Box<Engine> {
		Box::new(NullEngine{spec: spec})
	}
}

impl Engine for NullEngine {
	fn name(&self) -> &str { "NullEngine" }
	fn spec(&self) -> &Spec { &self.spec }
	fn evm_schedule(&self, _env_info: &EnvInfo) -> EvmSchedule { EvmSchedule::new_frontier() }
}
