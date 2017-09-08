// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Evm factory.
//!
use std::sync::Arc;
use vm::Vm;
use bigint::prelude::U256;
use super::interpreter::SharedCache;
use super::vmtype::VMType;

/// Evm factory. Creates appropriate Evm.
#[derive(Clone)]
pub struct Factory {
	evm: VMType,
	evm_cache: Arc<SharedCache>,
}

impl Factory {
	/// Create fresh instance of VM
	/// Might choose implementation depending on supplied gas.
	#[cfg(feature = "jit")]
	pub fn create(&self, gas: U256) -> Box<Vm> {
		match self.evm {
			VMType::Jit => {
				Box::new(super::jit::JitEvm::default())
			},
			VMType::Interpreter => if Self::can_fit_in_usize(gas) {
				Box::new(super::interpreter::Interpreter::<usize>::new(self.evm_cache.clone()))
			} else {
				Box::new(super::interpreter::Interpreter::<U256>::new(self.evm_cache.clone()))
			}
		}
	}

	/// Create fresh instance of VM
	/// Might choose implementation depending on supplied gas.
	#[cfg(not(feature = "jit"))]
	pub fn create(&self, gas: U256) -> Box<Vm> {
		match self.evm {
			VMType::Interpreter => if Self::can_fit_in_usize(gas) {
				Box::new(super::interpreter::Interpreter::<usize>::new(self.evm_cache.clone()))
			} else {
				Box::new(super::interpreter::Interpreter::<U256>::new(self.evm_cache.clone()))
			}
		}
	}

	/// Create new instance of specific `VMType` factory, with a size in bytes
	/// for caching jump destinations.
	pub fn new(evm: VMType, cache_size: usize) -> Self {
		Factory {
			evm: evm,
			evm_cache: Arc::new(SharedCache::new(cache_size)),
		}
	}

	fn can_fit_in_usize(gas: U256) -> bool {
		gas == U256::from(gas.low_u64() as usize)
	}
}

impl Default for Factory {
	/// Returns jitvm factory
	#[cfg(all(feature = "jit", not(test)))]
	fn default() -> Factory {
		Factory {
			evm: VMType::Jit,
			evm_cache: Arc::new(SharedCache::default()),
		}
	}

	/// Returns native rust evm factory
	#[cfg(any(not(feature = "jit"), test))]
	fn default() -> Factory {
		Factory {
			evm: VMType::Interpreter,
			evm_cache: Arc::new(SharedCache::default()),
		}
	}
}

#[test]
fn test_create_vm() {
	let _vm = Factory::default().create(U256::zero());
}

/// Create tests by injecting different VM factories
#[macro_export]
macro_rules! evm_test(
	(ignorejit => $name_test: ident: $name_jit: ident, $name_int: ident) => {
		#[test]
		#[ignore]
		#[cfg(feature = "jit")]
		fn $name_jit() {
			$name_test(Factory::new(VMType::Jit, 1024 * 32));
		}
		#[test]
		fn $name_int() {
			$name_test(Factory::new(VMType::Interpreter, 1024 * 32));
		}
	};
	($name_test: ident: $name_jit: ident, $name_int: ident) => {
		#[test]
		#[cfg(feature = "jit")]
		fn $name_jit() {
			$name_test(Factory::new(VMType::Jit, 1024 * 32));
		}
		#[test]
		fn $name_int() {
			$name_test(Factory::new(VMType::Interpreter, 1024 * 32));
		}
	}
);

/// Create ignored tests by injecting different VM factories
#[macro_export]
macro_rules! evm_test_ignore(
	($name_test: ident: $name_jit: ident, $name_int: ident) => {
		#[test]
		#[ignore]
		#[cfg(feature = "jit")]
		#[cfg(feature = "ignored-tests")]
		fn $name_jit() {
			$name_test(Factory::new(VMType::Jit, 1024 * 32));
		}
		#[test]
		#[ignore]
		#[cfg(feature = "ignored-tests")]
		fn $name_int() {
			$name_test(Factory::new(VMType::Interpreter, 1024 * 32));
		}
	}
);
