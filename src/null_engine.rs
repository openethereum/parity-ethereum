use engine::Engine;
use spec::Spec;
use evm::Schedule;
use evm::Factory;
use env_info::EnvInfo;

/// An engine which does not provide any consensus mechanism.
pub struct NullEngine {
	spec: Spec,
	factory: Factory
}

impl NullEngine {
	pub fn new_boxed(spec: Spec) -> Box<Engine> {
		Box::new(NullEngine{
			spec: spec,
			// TODO [todr] should this return any specific factory?
			factory: Factory::default()
		})
	}
}

impl Engine for NullEngine {
	fn vm_factory(&self) -> &Factory {
		&self.factory
	}
	fn name(&self) -> &str { "NullEngine" }
	fn spec(&self) -> &Spec { &self.spec }
	fn schedule(&self, _env_info: &EnvInfo) -> Schedule { Schedule::new_frontier() }
}
