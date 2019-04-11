use std::sync::{Arc, Weak};

use parking_lot::RwLock;

use ethcore::client::EngineClient;
use ethcore::engines::{total_difficulty_fork_choice, Engine, EthEngine, ForkChoice};
use ethcore::error::Error;
use types::header::{ExtendedHeader, Header};
use ethcore::machine::EthereumMachine;

pub struct HoneyBadgerBFT {
	client: Arc<RwLock<Option<Weak<EngineClient>>>>,
	machine: EthereumMachine,
}

impl HoneyBadgerBFT {
	pub fn new(
		_params: &serde_json::Value,
		machine: EthereumMachine,
	) -> Result<Arc<EthEngine>, Box<Error>> {
		let engine = Arc::new(HoneyBadgerBFT {
			client: Arc::new(RwLock::new(None)),
			machine: machine,
		});
		Ok(engine)
	}
}

impl Engine<EthereumMachine> for HoneyBadgerBFT {
	fn name(&self) -> &str {
		"HoneyBadgerBFT"
	}

	fn machine(&self) -> &EthereumMachine {
		&self.machine
	}

	fn verify_local_seal(&self, _header: &Header) -> Result<(), Error> {
		Ok(())
	}

	fn fork_choice(&self, new: &ExtendedHeader, current: &ExtendedHeader) -> ForkChoice {
		total_difficulty_fork_choice(new, current)
	}

	fn register_client(&self, client: Weak<EngineClient>) {
		*self.client.write() = Some(client.clone());
	}
}
