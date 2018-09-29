//! An hbbft <-> Parity link which relays events and acts as an intermediary.

#![allow(dead_code, unused_imports, unused_variables, unused_mut, missing_docs)]

use std::sync::{Arc, Weak, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Instant, Duration};
use rand::{self, ThreadRng, Rng};
use futures::{
	Future,
	future::{self, Loop},
	sync::mpsc::Receiver,
	sync::oneshot,
};
use parking_lot::Mutex;
use hydrabadger::{Hydrabadger, Error as HydrabadgerError};
use parity_reactor::{tokio::{self, timer::Delay}, Runtime};
use hbbft::HbbftConfig;
use rlp::Encodable;
use ethkey::{Random, Generator};
use ethereum_types::{U256, Address};
use header::Header;
use client::{BlockChainClient, Client, ClientConfig, BlockId, ChainInfo, BlockInfo, PrepareOpenBlock,
	ImportSealedBlock, ImportBlock};
use verification::queue::kind::blocks::{Unverified};
use transaction::{Transaction, Action};
use block::{OpenBlock, ClosedBlock, LockedBlock, SealedBlock};
use state::{self, State, CleanupMode};
use account_provider::AccountProvider;

type Txn = Vec<u8>;

// TODO: Replace error_chain (deprecated) with failure.
error_chain! {
	types {
		Error, ErrorKind, ErrorResultExt, HbbftDaemonResult;
	}

	errors {
		#[doc = "Tokio runtime start error."]
		RuntimeStart(err: tokio::io::Error) {
			description("Snapshot error.")
			display("Tokio runtime failed to start: {:?}", err)
		}
	}
}

/// Methods for use by hbbft.
//
// The purpose of this trait is to keep our own experimental methods
// organized.
pub trait HbbftClientExt {
	fn a_specialized_method(&self);
	fn change_me_into_something_useful(&self);
	fn import_a_bad_block_and_panic(&self);

	fn set_hbbft_daemon(&self, hbbft_daemon: Arc<HbbftDaemon>);
}

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
//////////////////////////////// EXPERIMENTS //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////

/// Experiments and other junk.
//
// Add anything at all to this!
//
struct Laboratory {
	client: Arc<Client>,
	hydrabadger: Hydrabadger<Txn>,
	hdb_cfg: HbbftConfig,
	account_provider: Arc<AccountProvider>,
}

impl Laboratory {
	/// Generates random transactions.
	//
	// TODO: Make this more random-y. Add some args.
	fn gen_random_txns(&self) -> Vec<Vec<u8>> {
		(0..self.hdb_cfg.txn_gen_count).map(|_| {
			let data = rand::thread_rng().gen_iter().take(self.hdb_cfg.txn_gen_bytes).collect();

			let key = Random.generate().unwrap();

			let t = Transaction {
				action: Action::Create,
				nonce: U256::from(42),
				gas_price: U256::from(3000),
				gas: U256::from(50_000),
				value: U256::from(1),
				data,
			}.sign(&key.secret(), None);

			t.rlp_bytes().into_vec()
		}).collect::<Vec<Txn>>()
	}

	fn push_random_transactions_to_hydrabadger(&self) {
		let random_txns = self.gen_random_txns();

		match self.hydrabadger.push_user_transactions(random_txns) {
			Err(HydrabadgerError::PushUserTransactionNotValidator) => {
				info!("Unable to push random transactions: this node is not a validator");
			},
			Err(err) => unreachable!(),
			Ok(()) => {},
		}
	}

	fn play_with_blocks(&self) {
		// let author = Address::from_slice(b"0xe8ddc5c7a2d2f0d7a9798459c0104fdf5e987aca");
		let author = Address::random();
		let gas_range_target = (3141562.into(), 31415620.into());
		let extra_data = vec![];

		let key = Random.generate().unwrap();
		let txn = Transaction {
			action: Action::Call(Address::default()),
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: 1000000.into(),
			value: 5.into(),
			data: vec![],
		}.sign(&key.secret(), None);

		// Import some blocks:
		for _ in 0..20 {
			let mut open_block: OpenBlock = self.client
				.prepare_open_block(author, gas_range_target, extra_data.clone())
				.unwrap();

			for _ in 0..5 {
				open_block.push_transaction(txn.clone(), None).unwrap();
			}

			let closed_block: ClosedBlock = open_block.close().unwrap();
			let reopened_block: OpenBlock = closed_block.reopen(self.client.engine());
			let reclosed_block: ClosedBlock = reopened_block.close().unwrap();
			let locked_block: LockedBlock = reclosed_block.lock();
			let sealed_block: SealedBlock = locked_block.seal(self.client.engine(), vec![]).unwrap();

			self.client.import_sealed_block(sealed_block).unwrap();
		}

		// Import some blocks:
		for _ in 0..20 {
			let mut open_block: OpenBlock = self.client
				.prepare_open_block(author, gas_range_target, extra_data.clone())
				.unwrap();

			for _ in 0..5 {
				open_block.push_transaction(txn.clone(), None).unwrap();
			}

			let sealed_block: SealedBlock = open_block
				.close_and_lock()
				.unwrap()
				.seal(self.client.engine(), vec![])
				.unwrap();

			self.client.import_sealed_block(sealed_block).unwrap();
		}
	}

	fn demonstrate_client_extension_methods(&self) {
		self.client.a_specialized_method();
		self.client.change_me_into_something_useful();
	}

	/// Runs all experiments.
	//
	// Call your experiments here.
	fn run_experiments(&mut self) {
		self.push_random_transactions_to_hydrabadger();
		// self.play_with_blocks();
		self.demonstrate_client_extension_methods();
	}
}

///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////

/// An hbbft <-> Parity link which relays events and acts as an intermediary.
pub struct HbbftDaemon {
	// Unused:
	client: Weak<Client>,

	hydrabadger: Hydrabadger<Txn>,

	// Temporary until tokio changes are merged upstream:
	shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
	runtime_th: thread::JoinHandle<()>,
}

impl HbbftDaemon {
	/// Returns a new `HbbftDaemon`.
	pub fn new(
		client: Arc<Client>,
		cfg: &HbbftConfig,
		account_provider: Arc<AccountProvider>
	) -> Result<HbbftDaemon, Error> {
		// Hydrabadger
		let hydrabadger = Hydrabadger::<Txn>::new(cfg.bind_address, cfg.to_hydrabadger());

		let (shutdown_tx, shutdown_rx) = oneshot::channel();

		// Create Tokio runtime:
		let mut runtime = Runtime::new().map_err(ErrorKind::RuntimeStart)?;
		let executor = runtime.executor();

		let hdb_clone = hydrabadger.clone();
		let hdb_peers = cfg.remote_addresses.clone();

		// Spawn runtime on its own thread:
		let runtime_th = thread::Builder::new().name("tokio-runtime".to_string()).spawn(move || {
			runtime.spawn(future::lazy(move || hdb_clone.node(Some(hdb_peers), None)));
			runtime.block_on(shutdown_rx).expect("Tokio runtime error");
			runtime.shutdown_now().wait().expect("Error shutting down tokio runtime");
		}).map_err(|err| format!("Error creating thread: {:?}", err))?;

		info!("Starting HbbftDaemon...");

		let client_clone = client.clone();
		let hdb_clone = hydrabadger.clone();
		let cfg_clone = cfg.clone();

		let lab = Laboratory {
			client: client.clone(),
			hydrabadger: hydrabadger.clone(),
			hdb_cfg: cfg.clone(),
			account_provider,
		};

		// Spawn experimentation loop:
		executor.spawn(future::loop_fn(lab, move |mut lab| {
			// Entry point for experiments:
			lab.run_experiments();

			Delay::new(Instant::now() + Duration::from_millis(5000))
				.map(|_| Loop::Continue(lab))
				.map_err(|err| panic!("{:?}", err))
		}));

		Ok(HbbftDaemon {
			client: Arc::downgrade(&client),
			hydrabadger,
			shutdown_tx: Mutex::new(Some(shutdown_tx)),
			runtime_th,
		})
	}

	/// Sends a shutdown single to the associated tokio runtime.
	//
	// Only needed until a proper global runtime is used.
	pub fn shutdown(&self) {
		self.shutdown_tx.lock().take().map(|tx| tx.send(()));
	}
}

impl Drop for HbbftDaemon {
	fn drop(&mut self) {
		self.shutdown();
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