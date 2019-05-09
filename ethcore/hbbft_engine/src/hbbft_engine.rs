use crate::contribution::Contribution;
use ethcore::block::ExecutedBlock;
use ethcore::client::EngineClient;
use ethcore::engines::signer::EngineSigner;
use ethcore::engines::{
	total_difficulty_fork_choice, Engine, EngineError, EthEngine, ForkChoice, Seal,
};
use ethcore::error::Error;
use ethcore::machine::EthereumMachine;
use hbbft::honey_badger::{HoneyBadgerBuilder, Step};
use hbbft::Target;
use itertools::Itertools;
use parking_lot::RwLock;
use rlp::{Decodable, Rlp};
use serde_json;
use std::sync::{Arc, Weak};
use types::header::{ExtendedHeader, Header};
use types::transaction::SignedTransaction;

type NodeId = usize;
type HoneyBadger = hbbft::honey_badger::HoneyBadger<Contribution, NodeId>;
type Message = hbbft::honey_badger::Message<NodeId>;
type Batch = hbbft::honey_badger::Batch<Contribution, NodeId>;
type TargetedMessage = hbbft::TargetedMessage<Message, NodeId>;

pub struct HoneyBadgerBFT {
	client: Arc<RwLock<Option<Weak<EngineClient>>>>,
	signer: RwLock<Option<Box<EngineSigner>>>,
	machine: EthereumMachine,
	transactions_trigger: usize,
	honey_badger: RwLock<Option<HoneyBadger>>,
}

impl HoneyBadgerBFT {
	pub fn new(
		_params: &serde_json::Value,
		machine: EthereumMachine,
	) -> Result<Arc<EthEngine>, Box<Error>> {
		let engine = Arc::new(HoneyBadgerBFT {
			client: Arc::new(RwLock::new(None)),
			signer: RwLock::new(None),
			machine: machine,
			// TODO: configure through spec params
			transactions_trigger: 1,
			honey_badger: RwLock::new(None),
		});
		Ok(engine)
	}

	fn new_honey_badger(&self) -> Option<HoneyBadger> {
		if let Some(ref weak) = *self.client.read() {
			if let Some(client) = weak.upgrade() {
				// TODO: Retrieve the information to build a node-specific NetworkInfo
				//       struct from the chain spec and from contracts.
				let net_info = client.net_info().unwrap();
				let mut builder: HoneyBadgerBuilder<Contribution, _> =
					HoneyBadger::builder(Arc::new(net_info.clone()));
				return Some(builder.build());
			}
		}
		None
	}

	fn process_output(&self, client: &Arc<EngineClient>, output: Vec<Batch>) {
		let batch = output.first().unwrap();
		// Decode and deduplicate transactions
		let batch_txns: Vec<_> = batch
			.contributions
			.iter()
			.flat_map(|(_, c)| &c.transactions)
			.filter_map(|ser_txn| {
				// TODO: Report proposers of malformed transactions.
				Decodable::decode(&Rlp::new(ser_txn)).ok()
			})
			.filter_map(|txn| {
				// TODO: Report proposers of invalidly signed transactions.
				SignedTransaction::new(txn).ok()
			})
			.collect();

		// We use the median of all contributions' timestamps
		let timestamps = batch
			.contributions
			.iter()
			.map(|(_, c)| c.timestamp)
			.sorted();
		let timestamp = timestamps[timestamps.len() / 2];

		client.create_pending_block(batch_txns, timestamp);
	}

	fn process_message(&self, sender_id: usize, message: Message) -> Result<(), EngineError> {
		if let Some(ref weak) = *self.client.read() {
			if let Some(client) = weak.upgrade() {
				if let Some(ref mut honey_badger) = *self.honey_badger.write() {
					match honey_badger.handle_message(&sender_id, message) {
						Ok(step) => {
							self.process_step(client, step);
						}
						_ => {
							// TODO: Report consensus step errors
							error!(target: "engine", "Error on HoneyBadger consensus step");
						}
					}
					Ok(())
				} else {
					Err(EngineError::UnexpectedMessage)
				}
			} else {
				Err(EngineError::RequiresClient)
			}
		} else {
			Err(EngineError::RequiresClient)
		}
	}

	fn dispatch_messages(&self, client: &Arc<EngineClient>, messages: Vec<TargetedMessage>) {
		for m in messages {
			if let Ok(ser) = serde_json::to_vec(&m.message) {
				match m.target {
					Target::Node(n) => {
						// for debugging
						println!("Sending targeted message: {:?}", m.message);
						client.send_consensus_message(ser, n);
					}
					Target::All => {
						// for debugging
						println!("Sending broadcast message: {:?}", m.message);
						let net_info = client.net_info().unwrap();
						for peer_id in net_info.all_ids().filter(|p| p != &net_info.our_id()) {
							client.send_consensus_message(ser.clone(), *peer_id);
						}
					}
				}
			} else {
				error!(target: "engine", "Serialization of consensus message failed!");
				panic!("Serialization of consensus message failed!");
			}
		}
	}

	fn process_step(&self, client: Arc<EngineClient>, step: Step<Contribution, usize>) {
		self.dispatch_messages(&client, step.messages);
		self.process_output(&client, step.output);
	}

	fn send_contribution(&self, client: Arc<EngineClient>) {
		if let Some(ref mut honey_badger) = *self.honey_badger.write() {
			// TODO: Select a random *subset* of transactions to propose
			let input_contribution = Contribution::new(
				&client
					.queued_transactions()
					.iter()
					.map(|txn| txn.signed().clone())
					.collect(),
			);
			let mut rng = rand::thread_rng();
			let step = honey_badger.propose(&input_contribution, &mut rng);

			match step {
				Ok(step) => {
					self.process_step(client, step);
				}
				_ => {
					// TODO: Report consensus step errors
					error!(target: "engine", "Error on HoneyBadger consensus step");
				}
			}
		}
	}

	fn start_hbbft_epoch(&self, client: Arc<EngineClient>) {
		// TODO: Check if an epoch is already started
		if let Some(honey_badger) = self.new_honey_badger() {
			*self.honey_badger.write() = Some(honey_badger);
			self.send_contribution(client);
		} else {
			error!(target: "engine", "HoneyBadger algorithm could not be created!");
			panic!("HoneyBadger algorithm could not be created!");
		}
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

	fn set_signer(&self, signer: Box<EngineSigner>) {
		*self.signer.write() = Some(signer);
	}

	fn clear_signer(&self) {
		*self.signer.write() = Default::default();
	}

	fn seals_internally(&self) -> Option<bool> {
		Some(true)
	}

	fn on_transactions_imported(&self) {
		if let Some(ref weak) = *self.client.read() {
			if let Some(client) = weak.upgrade() {
				if client.queued_transactions().len() >= self.transactions_trigger {
					self.start_hbbft_epoch(client);
				}
			}
		}
	}

	fn handle_message(&self, message: &[u8], peer_id: usize) -> Result<(), EngineError> {
		match serde_json::from_slice(message) {
			Ok(decoded_message) => self.process_message(peer_id, decoded_message),
			_ => Err(EngineError::UnexpectedMessage),
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

	fn should_miner_prepare_blocks(&self) -> bool {
		false
	}
}

#[cfg(test)]
mod tests {
	use crate::contribution::Contribution;
	use crate::test_helpers::create_transaction;
	use hbbft::honey_badger::{HoneyBadger, HoneyBadgerBuilder};
	use hbbft::NetworkInfo;
	use rand;
	use std::sync::Arc;
	use types::transaction::SignedTransaction;

	#[test]
	fn test_honey_badger_instantiation() {
		let mut rng = rand::thread_rng();
		let net_infos = NetworkInfo::generate_map(0..1usize, &mut rng)
			.expect("NetworkInfo generation is expected to always succeed");

		let net_info = net_infos
			.get(&0)
			.expect("A NetworkInfo must exist for node 0");

		let mut builder: HoneyBadgerBuilder<Contribution, _> =
			HoneyBadger::builder(Arc::new(net_info.clone()));

		let mut honey_badger = builder.build();

		let mut pending: Vec<SignedTransaction> = Vec::new();
		pending.push(create_transaction());
		let input_contribution = Contribution::new(&pending);

		let step = honey_badger
			.propose(&input_contribution, &mut rng)
			.expect("Since there is only one validator we expect an immediate result");

		// Assure the contribution returned by HoneyBadger matches the input
		assert_eq!(step.output.len(), 1);
		let out = step.output.first().unwrap();
		assert_eq!(out.epoch, 0);
		assert_eq!(out.contributions.len(), 1);
		assert_eq!(out.contributions.get(&0).unwrap(), &input_contribution);
	}
}
