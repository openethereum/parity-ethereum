//! An hbbft <-> Parity link which relays events and acts as an intermediary.

#![allow(dead_code, unused_imports, unused_variables, unused_mut, missing_docs)]

use std::collections::HashMap;
use std::sync::{Arc, Weak, atomic::{AtomicBool, AtomicIsize, Ordering}};
use std::thread;
use std::time::{Instant, Duration, UNIX_EPOCH};
use std::ops::Range;
// TODO (c0gent): Update rand crate wide.
use rand::{self, OsRng, Rng, distributions::{Sample, Range as RandRange}};
use futures::{
	task, Future, Poll, Stream, Async,
	future::{self, Loop},
	sync::mpsc::Receiver,
	sync::oneshot,
};
use parking_lot::Mutex;
use hydrabadger::{Hydrabadger, Error as HydrabadgerError, Batch, BatchRx, Uid, StateDsct};
use parity_runtime::Executor;
use tokio::{self, timer::Delay};
use hbbft::HbbftConfig;
use itertools::Itertools;
use rlp::{Decodable, Encodable, Rlp};
use ethstore;
use ethjson::misc::AccountMeta;
use ethkey::{Brain, Generator, Password, Random};
use ethereum_types::{U256, Address};
use header::Header;
use client::{BlockChainClient, Client, ClientConfig, BlockId, ChainInfo, BlockInfo, PrepareOpenBlock,
	ImportSealedBlock, ImportBlock};
use miner::{Miner, MinerService};
use verification::queue::kind::blocks::{Unverified};
use transaction::{Transaction, Action, SignedTransaction};
use block::{OpenBlock, ClosedBlock, IsBlock, LockedBlock, SealedBlock};
use state::{self, State, CleanupMode};
use account_provider::AccountProvider;
use super::laboratory::{Laboratory, Accounts};

const RICHIE_ACCT: &'static str = "0x002eb83d1d04ca12fe1956e67ccaa195848e437f";
const RICHIE_PWD: &'static str =  "richie";

const TXN_AMOUNT_MAX: usize = 1000;

// TODO: Move to config.
pub(super) const CONTRIBUTION_PUSH_DELAY_MS: u64 = 100;
const CONTRIBUTION_PUSH_ATTEMPTS_LIMIT: usize = 20;
const CONTRIBUTION_PUSH_BATCH_SIZE_MAX_LOG2: usize = 16;

type NodeId = Uid;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub(super) struct Contribution {
	transactions: Vec<Vec<u8>>,
	timestamp: u64,
}

// TODO (c0gent): Replace error_chain with failure.
error_chain! {
	types {
		Error, ErrorKind, ErrorResultExt, HbbftDaemonResult;
	}

	errors {
		#[doc = "A tokio runtime start error."]
		RuntimeStart(err: tokio::io::Error) {
			description("Tokio runtime failed to start")
			display("Tokio runtime failed to start: {:?}", err)
		}
		#[doc = "An unhandled hydrabadger error."]
		Hydrabadger(err: HydrabadgerError) {
			description("Unhandled hydrabadger error")
			display("Unhandled hydrabadger error: {:?}", err)
		}
		#[doc = "A hydrabadger batch receiver error."]
		HydrabadgerBatchRxPoll {
			description("Error polling hydrabadger internal receiver")
			display("Error polling hydrabadger internal receiver")
		}
		#[doc = "An ethstore account related error."]
		EthstoreAccountInitNode(err: ethstore::Error) {
			description("ethstore error (node)")
			display("ethstore error (node): {:?}", err)
		}
		#[doc = "An ethstore account related error."]
		EthstoreAccountInitRichie(err: ethstore::Error) {
			description("ethstore error (richie)")
			display("ethstore error (richie): {:?}", err)
		}
	}
}

/// Methods for use by hbbft.
//
// The purpose of this trait is to keep experimental methods separate and
// organized. TODO (c0gent): Consider this trait's future...
pub trait HbbftClientExt {
	fn a_specialized_method(&self);
	fn change_me_into_something_useful(&self);
	fn import_a_bad_block_and_panic(&self);

	fn set_hbbft_daemon(&self, hbbft_daemon: Arc<HbbftDaemon>);
}

/// Returns the current UNIX Epoch time, in seconds.
fn unix_now_secs() -> u64 {
	UNIX_EPOCH.elapsed().expect("Time not available").as_secs()
}

/// Handles submission of transactions into Hydrabadger.
struct ContributionPusher {
	client: Weak<Client>,
	hydrabadger: Hydrabadger<Contribution>,
	block_counter: Arc<AtomicIsize>,
	push_attempts: usize,
}

impl ContributionPusher {
	fn new(client: Weak<Client>, hydrabadger: Hydrabadger<Contribution>,
		block_counter: Arc<AtomicIsize>) -> ContributionPusher
	{
		ContributionPusher { client, hydrabadger, block_counter, push_attempts: 0 }
	}

	/// Returns the current number of transactions needed before a
	/// contribution is pushed.
	fn next_batch_threshold(&mut self) -> usize {
		let threshold = 1 << (CONTRIBUTION_PUSH_BATCH_SIZE_MAX_LOG2.saturating_sub(self.push_attempts));
		self.push_attempts += 1;
		threshold
	}

	/// Inputs pending transactions as this node's contribution for the next batch into Honey Badger.
	///
	/// Called every `CONTRIBUTION_PUSH_DELAY_MS`.
	fn push_contribution(&mut self) {
		let client = match self.client.upgrade() {
			Some(client) => client,
			None => return,
		};

		// Select new transactions and propose them for the next block.
		//
		// TODO (c0gent): Pull this from cfg or adjust dynamically.
		let batch_threshold = self.next_batch_threshold();

		let pending = client.miner().pending_transactions_from_queue(&*client, 1 << CONTRIBUTION_PUSH_BATCH_SIZE_MAX_LOG2);

		let validator_count = self.hydrabadger.peers().count_validators() + 1;

		if !self.hydrabadger.is_validator()
			|| validator_count < 2
			|| (pending.len() < batch_threshold
				&& !self.hydrabadger.state().dhb().map(|dhb| dhb.should_propose()).unwrap_or(false))
		{
			// Postpone the next epoch.
			return;
		}

		// Our contribution size.
		let contrib_size = batch_threshold / validator_count;

		let mut rng = rand::thread_rng();
		let txns = if pending.len() <= contrib_size {
			pending
		} else {
			rand::seq::sample_slice(&mut rng, &pending, contrib_size)
		};
		let ser_txns: Vec<_> = txns.into_iter().map(|txn| txn.signed().rlp_bytes()).collect();
		let contribution = Contribution {
			transactions: ser_txns,
			timestamp: unix_now_secs(),
		};
		info!("Proposing {} transactions (after {} attempts).", contribution.transactions.len(),
			self.push_attempts);

		self.hydrabadger.push_user_contribution(contribution)
			.expect("TODO: Add transactions back to miner txn queue");

		// Reset push attempts counter:
		self.push_attempts = 0;
	}

	/// Consumes this `ContributionPusher` and returns a `LoopFn` which calls
	/// `::push_contribution` every `CONTRIBUTION_PUSH_DELAY_MS`.
	fn into_loop(self) -> impl Future<Item = (), Error = ()> + Send {
		future::loop_fn(self, |mut cp| {
			cp.push_contribution();

			// This can be adjusted dynamically if needed:
			let loop_delay = CONTRIBUTION_PUSH_DELAY_MS;

			Delay::new(Instant::now() + Duration::from_millis(loop_delay))
				.map(|_| Loop::Continue(cp))
				.map_err(|err| panic!("{:?}", err))
		})
	}
}


/// Handles honey badger batch outputs.
//
// TODO: Create a transaction queue semaphore to allow/disallow transactions
// from being streamed into hydrabadger and manipulate its state from here.
struct BatchHandler {
	batch_rx: BatchRx<Contribution>,
	client: Weak<Client>,
	hydrabadger: Hydrabadger<Contribution>,
	block_counter: Arc<AtomicIsize>,
}

impl BatchHandler {
	fn new(batch_rx: BatchRx<Contribution>, client: Weak<Client>, hydrabadger: Hydrabadger<Contribution>,
		block_counter: Arc<AtomicIsize>) -> BatchHandler
	{
		BatchHandler { batch_rx, client, hydrabadger, block_counter }
	}

	/// Handles a batch of transactions output by the Honey Badger BFT
	/// algorithm.
	fn handle_batch(&mut self, batch: Batch<Contribution, NodeId>) {
		let epoch = batch.epoch();

		let client = match self.client.upgrade() {
			Some(client) => client,
			None => return,
		};

		let timestamps = batch.contributions().map(|(_, c)| c.timestamp).sorted();
		let batch_txns: Vec<_> = batch.contributions().flat_map(|(_, c)| &c.transactions).filter_map(|ser_txn| {
			// TODO: Report proposers of malformed transactions.
			Decodable::decode(&Rlp::new(ser_txn)).ok()
		}).filter_map(|txn| {
			// TODO: Report proposers of invalidly signed transactions.
			SignedTransaction::new(txn).ok()
		}).collect();

		let miner = client.miner();

		let mut open_block = miner.prepare_new_block(&*client).expect("TODO");

		// TODO: Sync block num with epoch upon startup.
		//
		if open_block.header().number() == epoch {
			// The block's timestamp is the median of the proposed timestamps. This guarantees that at least one correct
			// node's proposal was above it, and at least one was below it.
			let timestamp = open_block.header().timestamp().max(timestamps[timestamps.len() / 2]);
			open_block.set_timestamp(timestamp);
			let min_tx_gas = u64::max_value().into(); // TODO

			let txn_count = batch_txns.len();

			// Create a block from the agreed transactions. Seal it instantly and
			// import it.
			let block = miner.prepare_block_from(open_block, batch_txns, &*client, min_tx_gas).expect("TODO");

			info!("Importing block {} (#{}, epoch: {}, txns: {})",
				block.hash(), block.block().header.number(), epoch, txn_count);

			// TODO (afck/drpete): Replace instant sealing with a threshold signature.
			if !miner.seal_and_import_block_internally(&*client, block) {
				warn!("Failed to seal and import block.");
			}
		} else if open_block.header().number() < epoch {
			error!("Can't produce block: missing parent.");
		} else {
			error!("Block {} already imported.", epoch);
		}

		// Increment the counter used to sync the contribution pusher.
		self.block_counter.store(epoch as isize, Ordering::Release);
	}
}

impl Future for BatchHandler {
	type Item = ();
	type Error = Error;

	/// Polls the batch receiver until the hydrabadger handler batch
	/// transmitter (e.g. handler) is dropped.
	fn poll(&mut self) -> Poll<(), Error> {
		const BATCHES_PER_TICK: usize = 3;

		for i in 0..BATCHES_PER_TICK {
			match self.batch_rx.poll() {
				Ok(Async::Ready(Some(batch))) => {
					self.handle_batch(batch);

					// Exceeded max batches per tick, schedule notification:
					if i + 1 == BATCHES_PER_TICK {
						task::current().notify();
					}
				}
				Ok(Async::Ready(None)) => {
					// Batch handler has dropped.
					return Ok(Async::Ready(()));
				}
				Ok(Async::NotReady) => {}
				Err(()) => return Err(ErrorKind::HydrabadgerBatchRxPoll.into()),
			};
		}

		Ok(Async::NotReady)
	}
}

/// An hbbft <-> Parity link which relays events and acts as an intermediary.
pub struct HbbftDaemon {
	// Unused:
	client: Weak<Client>,
	hydrabadger: Hydrabadger<Contribution>,
}

impl HbbftDaemon {
	/// Returns a new `HbbftDaemon`.
	pub fn new(
		client: Arc<Client>,
		cfg: &HbbftConfig,
		account_provider: Arc<AccountProvider>,
		executor: &Executor,
	) -> Result<HbbftDaemon, Error> {
		let mut hdb_config = cfg.to_hydrabadger();

		// Set our starting epoch equal to the best block number in the chain:
		hdb_config.start_epoch =  client.chain_info().best_block_number;

		// Spawn Hydrabadger node:
		let hydrabadger = Hydrabadger::<Contribution>::new(cfg.bind_address, hdb_config);
		let hdb_peers = cfg.remote_addresses.clone();
		executor.spawn(hydrabadger.clone().node(Some(hdb_peers), None));

		// Used by laboratory:
		let block_counter = Arc::new(AtomicIsize::new(-1));

		let batch_handler = BatchHandler::new(
			hydrabadger.batch_rx()
				.expect("The Hydrabadger batch receiver can not be `None` immediately after creation; qed \
					These proofs are bullshit and prove nothing; qed"),
			Arc::downgrade(&client),
			hydrabadger.clone(),
			block_counter.clone(),
		);

		// Spawn batch handler:
		executor.spawn(batch_handler.map_err(|err| panic!("Unhandled batch handler error: {:?}", err)));

		info!("Hbbft batch handler has been started.");

		// Spawn contribution pusher:
		executor.spawn(ContributionPusher::new(
			Arc::downgrade(&client),
			hydrabadger.clone(),
			block_counter.clone(),
		).into_loop());

		info!("Hbbft contribution pusher has been started.");

		// Set up an account to use for txn gen:
		let accounts = Accounts::new(&*account_provider, &*client, &cfg.bind_address.to_string(),
			cfg.txn_gen_count, 5)?;

		// Spawn experimentation loop:
		executor.spawn(Laboratory {
			client: client.clone(),
			hydrabadger: hydrabadger.clone(),
			hdb_cfg: cfg.clone(),
			account_provider,
			accounts,
			block_counter,
			last_block: 0,
			gen_counter: 0,
		}.into_loop());

		Ok(HbbftDaemon {
			client: Arc::downgrade(&client),
			hydrabadger,
		})
	}
}


#[cfg(test)]
mod tests {
	use client::{TestBlockChainClient, EachBlockWith, BlockId, BlockChainClient,
		Nonce, Balance, ChainInfo, BlockInfo, CallContract, TransactionInfo,
		RegistryInfo, ReopenBlock, PrepareOpenBlock, ScheduleInfo, ImportSealedBlock,
		BroadcastProposalBlock, ImportBlock, StateOrBlock, StateInfo, StateClient, Call,
		AccountData, BlockChain as BlockChainTrait, BlockProducer, SealedBlockImporter,
		ClientIoMessage,
	};

	use verification::queue::kind::blocks::{Unverified};
	use rlp::{Rlp, RlpStream, DecoderError};
	use block::{OpenBlock, SealedBlock, ClosedBlock};
	use header::Header;



	#[test]
	fn add_transaction() {
		let client = TestBlockChainClient::new();

		let bad_block = Unverified {
			header: Header::default(),
			transactions: vec![],
			uncles: vec![],
			bytes: vec![1, 2, 3],
		};

		client.import_block(bad_block).unwrap();
	}
}


#[cfg(feature = "ref_000")]
mod ref_000 {
	//! Reference material

	// ethcore/transaction/src/transaction.rs
	//
	/// A set of information describing an externally-originating message call
	/// or contract creation operation.
	#[derive(Default, Debug, Clone, PartialEq, Eq)]
	pub struct Transaction {
		/// Nonce.
		pub nonce: U256,
		/// Gas price.
		pub gas_price: U256,
		/// Gas paid up front for transaction execution.
		pub gas: U256,
		/// Action, can be either call or contract create.
		pub action: Action,
		/// Transfered value.
		pub value: U256,
		/// Transaction data.
		pub data: Bytes,
	}

	// ethcore/transaction/src/transaction.rs
	//
	/// Signed transaction information without verified signature.
	#[derive(Debug, Clone, Eq, PartialEq)]
	pub struct UnverifiedTransaction {
		/// Plain Transaction.
		unsigned: Transaction,
		/// The V field of the signature; the LS bit described which half of the curve our point falls
		/// in. The MS bits describe which chain this transaction is for. If 27/28, its for all chains.
		v: u64,
		/// The R field of the signature; helps describe the point on the curve.
		r: U256,
		/// The S field of the signature; helps describe the point on the curve.
		s: U256,
		/// Hash of the transaction
		hash: H256,
	}

	/// A `UnverifiedTransaction` with successfully recovered `sender`.
	#[derive(Debug, Clone, Eq, PartialEq)]
	pub struct SignedTransaction {
		transaction: UnverifiedTransaction,
		sender: Address,
		public: Option<Public>,
	}

	// miner/src/pool/verifier.rs
	//
	/// Transaction to verify.
	#[cfg_attr(test, derive(Clone))]
	pub enum Transaction {
		/// Fresh, never verified transaction.
		///
		/// We need to do full verification of such transactions
		Unverified(transaction::UnverifiedTransaction),

		/// Transaction from retracted block.
		///
		/// We could skip some parts of verification of such transactions
		Retracted(transaction::UnverifiedTransaction),

		/// Locally signed or retracted transaction.
		///
		/// We can skip consistency verifications and just verify readiness.
		Local(transaction::PendingTransaction),
	}

	// ethcore/src/header.rs
	//
	/// A block header.
	///
	/// Reflects the specific RLP fields of a block in the chain with additional room for the seal
	/// which is non-specific.
	///
	/// Doesn't do all that much on its own.
	#[derive(Debug, Clone, Eq)]
	pub struct Header {
		/// Parent hash.
		parent_hash: H256,
		/// Block timestamp.
		timestamp: u64,
		/// Block number.
		number: BlockNumber,
		/// Block author.
		author: Address,

		/// Transactions root.
		transactions_root: H256,
		/// Block uncles hash.
		uncles_hash: H256,
		/// Block extra data.
		extra_data: Bytes,

		/// State root.
		state_root: H256,
		/// Block receipts root.
		receipts_root: H256,
		/// Block bloom.
		log_bloom: Bloom,
		/// Gas used for contracts execution.
		gas_used: U256,
		/// Block gas limit.
		gas_limit: U256,

		/// Block difficulty.
		difficulty: U256,
		/// Vector of post-RLP-encoded fields.
		seal: Vec<Bytes>,

		/// Memoized hash of that header and the seal.
		hash: Option<H256>,
	}

	// ethcore/src/verification/queue/kind.rs
	//
	/// An unverified block.
	#[derive(PartialEq, Debug)]
	pub struct Unverified {
		/// Unverified block header.
		pub header: Header,
		/// Unverified block transactions.
		pub transactions: Vec<UnverifiedTransaction>,
		/// Unverified block uncles.
		pub uncles: Vec<Header>,
		/// Raw block bytes.
		pub bytes: Bytes,
	}

	// ethcore/src/verification/verification.rs
	//
	/// Preprocessed block data gathered in `verify_block_unordered` call
	pub struct PreverifiedBlock {
		/// Populated block header
		pub header: Header,
		/// Populated block transactions
		pub transactions: Vec<SignedTransaction>,
		/// Populated block uncles
		pub uncles: Vec<Header>,
		/// Block bytes
		pub bytes: Bytes,
	}

	/// A block, encoded as it is on the block chain.
	#[derive(Default, Debug, Clone, PartialEq)]
	pub struct Block {
		/// The header of this block.
		pub header: Header,
		/// The transactions in this block.
		pub transactions: Vec<UnverifiedTransaction>,
		/// The uncles of this block.
		pub uncles: Vec<Header>,
	}

	/// An internal type for a block's common elements.
	#[derive(Clone)]
	pub struct ExecutedBlock {
		/// Executed block header.
		pub header: Header,
		/// Executed transactions.
		pub transactions: Vec<SignedTransaction>,
		/// Uncles.
		pub uncles: Vec<Header>,
		/// Transaction receipts.
		pub receipts: Vec<Receipt>,
		/// Hashes of already executed transactions.
		pub transactions_set: HashSet<H256>,
		/// Underlaying state.
		pub state: State<StateDB>,
		/// Transaction traces.
		pub traces: Tracing,
		/// Hashes of last 256 blocks.
		pub last_hashes: Arc<LastHashes>,
	}

	/// Block that is ready for transactions to be added.
	///
	/// It's a bit like a Vec<Transaction>, except that whenever a transaction is pushed, we execute it and
	/// maintain the system `state()`. We also archive execution receipts in preparation for later block creation.
	pub struct OpenBlock<'x> {
		block: ExecutedBlock,
		engine: &'x EthEngine,
	}

	/// Just like `OpenBlock`, except that we've applied `Engine::on_close_block`, finished up the non-seal header fields,
	/// and collected the uncles.
	///
	/// There is no function available to push a transaction.
	#[derive(Clone)]
	pub struct ClosedBlock {
		block: ExecutedBlock,
		unclosed_state: State<StateDB>,
	}

	/// Just like `ClosedBlock` except that we can't reopen it and it's faster.
	///
	/// We actually store the post-`Engine::on_close_block` state, unlike in `ClosedBlock` where it's the pre.
	#[derive(Clone)]
	pub struct LockedBlock {
		block: ExecutedBlock,
	}

	/// A block that has a valid seal.
	///
	/// The block's header has valid seal arguments. The block cannot be reversed into a `ClosedBlock` or `OpenBlock`.
	pub struct SealedBlock {
		block: ExecutedBlock,
	}
}


/*********************************** NOTES ************************************
*******************************************************************************
*******************************************************************************










*******************************************************************************
*******************************************************************************
******************************************************************************/
