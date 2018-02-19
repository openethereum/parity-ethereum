use futures::Future;
use ethabi::{Address, Bytes};

use_contract!(registry, "Registry", "res/registrar.json");

pub struct RegistryClient {
	registrar: registry::Registry,
}

impl RegistryClient {
	/// ContractClient constructor
	pub fn new() -> Self {
		Self { registrar: registry::Registry::default() }
	}

	/// Get address (wrapper on top of registry::functions::GetAddress)
	pub fn get_address(&self) -> registry::functions::GetAddress {
		self.registrar.functions().get_address()
	}
}

/// RAW Contract interface.
/// Should execute transaction using current blockchain state.
pub trait ContractClient: Send + Sync {
	/// Get registrar address
	fn registrar(&self) -> Result<Address, String>;
	/// Call Contract
	fn call(&self, address: Address, data: Bytes) -> Box<Future<Item = Bytes, Error = String> + Send>;
}

