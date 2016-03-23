// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

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
	/// Returns new instance of NullEngine with default VM Factory
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

	fn schedule(&self, env_info: &EnvInfo) -> Schedule {
		if env_info.number < self.u64_param("frontierCompatibilityModeLimit") {
			Schedule::new_frontier()
		} else {
			Schedule::new_homestead()
		}
	}
}
