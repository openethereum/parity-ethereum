//! Evm factory.
use std::fmt;
use evm::Evm;

#[derive(Clone)]
/// TODO [Tomusdrw] Please document me
pub enum VMType {
	/// TODO [Tomusdrw] Please document me
	Jit,
	/// TODO [Tomusdrw] Please document me
	Interpreter
}

impl fmt::Display for VMType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", match *self {
			VMType::Jit => "JIT",
			VMType::Interpreter => "INT"
		})
	}
}

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

	/// Create new instance of specific `VMType` factory
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

/// Create tests by injecting different VM factories
#[macro_export]
macro_rules! evm_test(
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
