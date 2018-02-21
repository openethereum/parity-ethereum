use futures::{Future, future};
use ethabi::{Address, Bytes};
use std::sync::Arc;
use keccak_hash::keccak;

use_contract!(registry, "Registry", "res/registrar.json");

// This need to be documented why it is concatenated with the hash
const MAGIC_CONST: &'static str = "A";

/// Registrar client is a dedicated interface to access the registrar contract
/// which in turn generates an address when a service requests one
pub struct RegistrarClient {
	registrar: registry::Registry,
	contract_client: Arc<AsyncContractClient>,
}

impl RegistrarClient {
	/// Registrar client constructor
	pub fn new(client: Arc<AsyncContractClient>) -> Self {
		Self {
			registrar: registry::Registry::default(),
			contract_client: client,
		}
	}

	/// Generate an address for the given key
	pub fn get_address<'a>(&self, key: &'a str) -> Box<Future<Item = Address, Error = String> + Send> {
		// Address of the registrar itself
		let registrar_address = match self.contract_client.registrar_address() {
			Ok(a) => a,
			Err(e) => return Box::new(future::err(e)),
		};

		let address_fetcher = self.registrar.functions().get_address();
		let id = address_fetcher.input(keccak(key), MAGIC_CONST);

		let future = self.contract_client.call_contract(registrar_address, id).and_then(move |address| {
			address_fetcher.output(&address)
		}
		.map_err(|e| e.to_string()));

		Box::new(future)
	}
}

/// Syncronous contract interface.
/// Should execute transaction using current blockchain state.
pub trait ContractClient: Send + Sync {
	/// Get registrar address
	fn registrar_address(&self) -> Result<Address, String>;
	/// Call Contract
	fn call_contract(&self, address: Address, data: Bytes) -> Result<Bytes, String>;
}

/// Asyncronous contract interface.
/// Should execute transaction using current blockchain state.
pub trait AsyncContractClient: Send + Sync {
	/// Get registrar address
	fn registrar_address(&self) -> Result<Address, String>;
	/// Call Contract
	fn call_contract(&self, address: Address, data: Bytes) -> Box<Future<Item = Bytes, Error = String> + Send>;
}

