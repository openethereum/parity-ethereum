//! An hbbft <-> Parity link which relays events and acts as an intermediary.

#![allow(dead_code, unused_imports, unused_variables, missing_docs)]

use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use futures::{
	Future,
	future,
	sync::mpsc::Receiver,
	sync::oneshot,
};
use client::{Client, ImportBlock};
use parity_reactor::{tokio, Runtime};
use verification::queue::kind::blocks::{Unverified};
use header::Header;
use hbbft::HbbftConfig;
use hydrabadger::{Hydrabadger};

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
///
/// The purpose of this trait is to keep our own experimental methods
/// organized.
pub trait HbbftClientExt {
	fn a_specialized_method(&self);
	fn change_me_into_something_useful(&self);
	fn import_a_bad_block_and_panic(&self);
}

/// Coordinates shutdown between the tokio runtime and the experimentation
/// thread (below).
#[derive(Debug)]
struct Shutdown {
	tx: Option<oneshot::Sender<()>>,
	sig: Arc<AtomicBool>,
}

impl Shutdown {
	/// Returns a new `Shutdown`.
	fn new() -> (Shutdown, oneshot::Receiver<()>) {
		let (tx, rx) = oneshot::channel();
		let sd = Shutdown {
			tx: Some(tx),
			sig: Arc::new(AtomicBool::new(false)),
		};
		(sd, rx )
	}

	/// Sends shutdown signals.
	fn shutdown(&mut self) {
		self.sig.store(true, Ordering::Release);
		self.tx.take().map(|tx| tx.send(()));
	}
}


/// An hbbft <-> Parity link which relays events and acts as an intermediary.
pub struct HbbftDaemon {
	client: Arc<Client>,
	runtime_th: thread::JoinHandle<()>,
	th: thread::JoinHandle<()>,
	shutdown: Shutdown,
}

impl HbbftDaemon {
	/// Returns a new `HbbftDaemon`.
	pub fn new(client: Arc<Client>, cfg: &HbbftConfig) -> Result<HbbftDaemon, Error> {
		// Hydrabadger
		let hdb = Hydrabadger::<u8>::new(cfg.bind_address, cfg.to_hydrabadger());
		let hdb_peers = cfg.remote_addresses.clone();

		let (shutdown, shutdown_rx) = Shutdown::new();

		// Create Tokio runtime:
		let mut runtime = Runtime::new().map_err(ErrorKind::RuntimeStart)?;

		// Spawn runtime on its own thread:
		let runtime_th = thread::Builder::new().name("tokio-runtime".to_string()).spawn(move || {
			runtime.spawn(future::lazy(move || hdb.clone().node(Some(hdb_peers)) ));
			runtime.block_on(shutdown_rx).expect("Tokio runtime error");
			runtime.shutdown_now().wait().expect("Error shutting down tokio runtime");
		}).map_err(|err| format!("Error creating thread: {:?}", err))?;

		info!("Starting HbbftDaemon...");

		let client_clone = client.clone();
		let shutdown_sig = shutdown.sig.clone();

		// Spawn experemintation thread:
		let th = thread::Builder::new().name("hbbft-daemon".to_string()).spawn(move || {
			let client = client_clone;

			while !shutdown_sig.load(Ordering::Acquire) {

				// Call experimental methods:
				client.a_specialized_method();
				client.change_me_into_something_useful();
				// client.import_a_bad_block_and_panic();

				thread::sleep(Duration::from_millis(5000));
			}
		}).unwrap();

		Ok(HbbftDaemon {
			client,
			runtime_th,
			th,
			shutdown,
		})
	}
}

impl Drop for HbbftDaemon {
	fn drop(&mut self) {
		self.shutdown.shutdown();
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
}