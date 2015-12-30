//! Evm factory.

use evm::Evm;

/// Vm factory. Creates appropriate Evm.
/// TODO: SmartVm
pub struct VmFactory;

impl VmFactory {
	/// Returns jit vm
	#[cfg(feature = "jit")]
	pub fn create() -> Box<Evm> {
		Box::new(super::jit::JitEvm)
	}

	/// Returns native rust evm
	#[cfg(not(feature = "jit"))]
	pub fn create() -> Box<Evm> {
		unimplemented!();
	}
}

#[test]
fn test_create_vm() {
	let _vm = VmFactory::create();
}
