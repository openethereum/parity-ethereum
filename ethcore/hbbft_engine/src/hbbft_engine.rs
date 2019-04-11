use std::sync::{Arc, Weak};

use parking_lot::RwLock;

use ethcore::block::ExecutedBlock;
use ethcore::client::EngineClient;
use ethcore::engines::{total_difficulty_fork_choice, Engine, EthEngine, ForkChoice, Seal};
use ethcore::error::Error;
use ethcore::machine::EthereumMachine;
use types::header::{ExtendedHeader, Header};
use types::transaction::SignedTransaction;

pub struct HoneyBadgerBFT {
	client: Arc<RwLock<Option<Weak<EngineClient>>>>,
	machine: EthereumMachine,
	transactions_trigger: usize,
}

impl HoneyBadgerBFT {
	pub fn new(
		_params: &serde_json::Value,
		machine: EthereumMachine,
	) -> Result<Arc<EthEngine>, Box<Error>> {
		let engine = Arc::new(HoneyBadgerBFT {
			client: Arc::new(RwLock::new(None)),
			machine: machine,
			// TODO: configure through spec params
			transactions_trigger: 1,
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

	fn seals_internally(&self) -> Option<bool> {
		Some(true)
	}

	fn on_transactions_imported(&self) {
		if let Some(ref weak) = *self.client.read() {
			if let Some(c) = weak.upgrade() {
				if c.queued_transactions().len() >= self.transactions_trigger {
					c.update_sealing();
				}
			}
		}
	}

	fn on_prepare_block(&self, _block: &ExecutedBlock) -> Result<Vec<SignedTransaction>, Error> {
		// TODO: inject random number transactions
		Ok(Vec::new())
	}

	fn generate_seal(&self, _block: &ExecutedBlock, _parent: &Header) -> Seal {
		// For refactoring/debugging of block creation we seal instantly.
		Seal::Regular(Vec::new())
	}
}
