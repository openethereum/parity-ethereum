use std::sync::Arc;

use ethcore::engines::{total_difficulty_fork_choice, Engine, ForkChoice};
use ethcore::error::Error;
use types::header::{ExtendedHeader, Header};
use ethcore::machine::EthereumMachine;

pub struct HoneyBadgerBFT {
	machine: EthereumMachine,
}

impl HoneyBadgerBFT {
	pub fn new(machine: EthereumMachine) -> Result<Arc<Self>, Error> {
		let engine = Arc::new(HoneyBadgerBFT { machine: machine });
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
}
