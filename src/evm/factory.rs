//! Evm factory.

use evm::Evm;

pub enum VMType {
	Jit,
	Interpreter
}

/// Evm factory. Creates appropriate Evm.
pub struct Factory {
	evm : VMType
}

impl Factory {

	pub fn create(&self) -> Box<Evm> {
		match self.evm {
			VMType::Jit => {
				Factory::jit()
			},
			VMType::Interpreter => {
				Box::new(super::interpreter::Interpreter)
			}
		}	
	}

	pub fn new(evm: VMType) -> Factory {
		Factory {
			evm: evm
		}
	}

	#[cfg(feature = "jit")]
	fn jit() -> Box<Evm> {
		Box::new(super::jit::JitEvm)
	}

	#[cfg(not(feature = "jit"))]
	fn jit() -> Box<Evm> {
		unimplemented!()
	}

	/// Returns jitvm factory
	#[cfg(feature = "jit")]
	pub fn default() -> Factory {
		Factory {
			evm: VMType::Jit
		}
	}

	/// Returns native rust evm factory
	#[cfg(not(feature = "jit"))]
	pub fn default() -> Factory {
		Factory {
			evm: VMType::Interpreter
		}
	}
}

#[test]
fn test_create_vm() {
	let _vm = Factory::default().create();
}
