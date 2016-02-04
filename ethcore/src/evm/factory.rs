//! Evm factory.
//!
//! TODO: consider spliting it into two separate files.
#[cfg(test)]
use std::fmt;
use evm::Evm;

#[derive(Clone)]
/// Type of EVM to use.
pub enum VMType {
	/// JIT EVM
	#[cfg(feature="jit")]
	Jit,
	/// RUST EVM
	Interpreter
}

#[cfg(test)]
impl fmt::Display for VMType {
	#[cfg(feature="jit")]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", match *self {
			VMType::Jit => "JIT",
			VMType::Interpreter => "INT"
		})
	}
	#[cfg(not(feature="jit"))]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", match *self {
			VMType::Interpreter => "INT"
		})
	}
}

#[cfg(test)]
#[cfg(feature = "json-tests")]
impl VMType {
	/// Return all possible VMs (JIT, Interpreter)
	#[cfg(feature="jit")]
	pub fn all() -> Vec<VMType> {
		vec![VMType::Jit, VMType::Interpreter]
	}

	/// Return all possible VMs (Interpreter)
	#[cfg(not(feature="jit"))]
	pub fn all() -> Vec<VMType> {
		vec![VMType::Interpreter]
	}
}

/// Evm factory. Creates appropriate Evm.
pub struct Factory {
	evm : VMType
}

impl Factory {
	/// Create fresh instance of VM
	#[cfg(feature="jit")]
	pub fn create(&self) -> Box<Evm> {
		match self.evm {
			VMType::Jit => {
				Box::new(super::jit::JitEvm)
			},
			VMType::Interpreter => {
				Box::new(super::interpreter::Interpreter)
			}
		}	
	}

	/// Create fresh instance of VM
	#[cfg(not(feature="jit"))]
	pub fn create(&self) -> Box<Evm> {
		match self.evm {
			VMType::Interpreter => {
				Box::new(super::interpreter::Interpreter)
			}
		}	
	}

	/// Create new instance of specific `VMType` factory
	#[cfg(test)]
	pub fn new(evm: VMType) -> Factory {
		Factory {
			evm: evm
		}
	}
}
impl Default for Factory {
	/// Returns jitvm factory
	#[cfg(feature = "jit")]
	fn default() -> Factory {
		Factory {
			evm: VMType::Jit
		}
	}

	/// Returns native rust evm factory
	#[cfg(not(feature = "jit"))]
	fn default() -> Factory {
		Factory {
			evm: VMType::Interpreter
		}
	}
}

#[test]
fn test_create_vm() {
	let _vm = Factory::default().create();
}

/// Create tests by injecting different VM factories
#[macro_export]
macro_rules! evm_test(
	(ignorejit => $name_test: ident: $name_jit: ident, $name_int: ident) => {
		#[test]
		#[ignore]
		#[cfg(feature = "jit")]
		fn $name_jit() {
			$name_test(Factory::new(VMType::Jit));
		}
		#[test]
		fn $name_int() {
			$name_test(Factory::new(VMType::Interpreter));
		}
	};
	($name_test: ident: $name_jit: ident, $name_int: ident) => {
		#[test]
		#[cfg(feature = "jit")]
		fn $name_jit() {
			$name_test(Factory::new(VMType::Jit));
		}
		#[test]
		fn $name_int() {
			$name_test(Factory::new(VMType::Interpreter));
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
		fn $name_jit() {
			$name_test(Factory::new(VMType::Jit));
		}
		#[test]
		#[ignore]
		fn $name_int() {
			$name_test(Factory::new(VMType::Interpreter));
		}
	}
);
