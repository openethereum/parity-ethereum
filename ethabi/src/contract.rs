use spec::Interface;
use function::Function;
use constructor::Constructor;

/// API building calls to contracts ABI.
pub struct Contract {
	interface: Interface,
}

impl Contract {
	/// Initializes contract with ABI specification.
	pub fn new(interface: Interface) -> Self {
		Contract {
			interface: interface
		}
	}

	/// Creates constructor call builder.
	pub fn constructor(&self) -> Option<Constructor> {
		self.interface.constructor().map(Constructor::new)
	}

	/// Creates function call builder.
	pub fn function(&self, name: String) -> Option<Function> {
		self.interface.function(name).map(Function::new)
	}
}
