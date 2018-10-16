//! An hbbft <-> Parity link which relays events and acts as an intermediary.

#![allow(dead_code, unused_imports, unused_variables, unused_mut, missing_docs)]

use std::collections::HashMap;
use std::sync::{Arc, Weak, atomic::{AtomicBool, Ordering}};
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
use parity_reactor::{tokio::{self, timer::Delay}, Runtime};
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
use miner::Miner;
use verification::queue::kind::blocks::{Unverified};
use transaction::{Transaction, Action, SignedTransaction};
use block::{OpenBlock, ClosedBlock, IsBlock, LockedBlock, SealedBlock};
use state::{self, State, CleanupMode};
use account_provider::AccountProvider;

const RICHIE_ACCT: &'static str = "0x002eb83d1d04ca12fe1956e67ccaa195848e437f";
const RICHIE_PWD: &'static str =  "richie";
// const NODE0_ACCT: &'static str = "0x00bd138abd70e2f00903268f3db08f2d25677c9e";
// const NODE0_PWD: &'static str =  "node0";

const TXN_AMOUNT_MAX: usize = 1000;

type NodeId = Uid;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
struct Contribution {
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
	hydrabadger: Hydrabadger<Contribution>,
	hdb_cfg: HbbftConfig,
	account_provider: Arc<AccountProvider>,
	account_addr: Address,
	account_pwd: Password,
}

impl Laboratory {
	/// Returns each Parity account's address and metadata.
	fn get_accounts(&self) -> HashMap<Address, AccountMeta> {
		self.account_provider.accounts_info().unwrap()
	}

	/// Converts an unsigned `Transaction` to a `SignedTransaction`.
	fn sign_txn(&self, sender: Address, password: Password, txn: Transaction) -> SignedTransaction {
		let chain_id = self.client.signing_chain_id();
		let txn_hash = txn.hash(chain_id);
		let sig = self.account_provider.sign(sender, Some(password), txn_hash)
			.unwrap_or_else(|e| panic!("[hbbft-lab] failed to sign txn: {:?}", e));
		let unverified_txn = txn.with_signature(sig, chain_id);
		SignedTransaction::new(unverified_txn).unwrap()
	}

	/// Generates a random-ish transaction.
	fn gen_random_txn(&self, nonce: U256, sender: Address, sender_pwd: Password, receiver: Address,
		value_range: &mut RandRange<usize>, rng: &mut OsRng) -> SignedTransaction
	{
		let data = rng.gen_iter().take(self.hdb_cfg.txn_gen_bytes).collect();
		let txn = Transaction {
			action: Action::Call(receiver),
			nonce,
			gas_price: 0.into(),
			gas: 1000000.into(),
			value: value_range.sample(rng).into(),
			data,
		};

		self.sign_txn(sender, sender_pwd, txn)
	}

	/// Generates a set of random-ish transactions.
	fn gen_random_contribution(&self, sender: Address, sender_pwd: Password, receiver: Address,
		receiver_pwd: Password, value_range: &mut RandRange<usize>) -> Contribution
	{
		let mut rng = OsRng::new().expect("Error creating OS Rng");

		let sender_start_nonce = self.client.state().nonce(&sender)
			.unwrap_or_else(|_| panic!("Unable to determine nonce for account: {}", sender));

		let receiver_start_nonce = self.client.state().nonce(&receiver)
			.unwrap_or_else(|_| panic!("Unable to determine nonce for account: {}", sender));

		// Determine the psuedo node id:
		let node_id = self.hdb_cfg.bind_address.port() % 100;

		// This is total hackfoolery to ensure that each node's sender account
		// gets a starting balance:
		let txns = if U256::from(node_id) == receiver_start_nonce {
			debug!("######## LABORATORY: Sending funds to {:?}", sender);
			// Add a contribution to initialize account:
			vec![self.sign_txn(receiver, receiver_pwd.clone(), Transaction {
				action: Action::Call(sender),
				nonce: receiver_start_nonce,
				gas_price: 0.into(),
				gas: 1000000.into(),
				value: (1000000000000000000 as u64).into(),
				data: vec![],
			}).rlp_bytes()]
		} else {
			// Ensure there is enough balance in the sender account:
			if self.client.state().balance(&sender).unwrap() > U256::from(TXN_AMOUNT_MAX * self.hdb_cfg.txn_gen_count) {
				// Generate random txns normally:
				(0..self.hdb_cfg.txn_gen_count).map(|i| {
					self.gen_random_txn(sender_start_nonce + i, sender, sender_pwd.clone(), receiver, value_range, &mut rng)
						.rlp_bytes()
				}).collect()
			} else {
				// TEMPRORARY: This will break when nodes > 3:
				if receiver_start_nonce > U256::from(3) {
					panic!("LABORATORY: Error initializing sender account balance");
				}
				debug!("######## LABORATORY: receiver_start_nonce: {}", receiver_start_nonce);
				vec![]
			}
		};

		Contribution {
			transactions: txns,
			timestamp: UNIX_EPOCH.elapsed().expect("Valid time has to be set in your system.").as_secs(),
		}
	}

	/// Returns false if the account does not exist, true if the account
	/// exists and the password is correct, and panics if the account exists
	/// but the password is incorrect.
	fn test_password(&self, addr: &Address, pwd: &Password) -> bool {
		match self.account_provider.test_password(addr, pwd) {
			Ok(false) => panic!("Bad password while pushing random transactions to Hydrabadger."),
			Ok(true) => {},
			Err(ethstore::Error::InvalidAccount) => {
				error!("Transaction sender account does not exist. Skipping hydrabadger contribution push.");
				return false;
			},
			err => panic!("{:?}", err),
		}
		true
	}

	fn push_random_transactions_to_hydrabadger(&self) {
		let sender_addr = self.account_addr;
		let sender_pwd = self.account_pwd.clone();
		let receiver_addr = Address::from(RICHIE_ACCT);
		let receiver_pwd = Password::from(RICHIE_PWD);

		if !(self.test_password(&sender_addr, &sender_pwd)
			&& self.test_password(&receiver_addr, &receiver_pwd))
		{
			return;
		}

		let (state, _, _) = self.hydrabadger.state_info_stale();
		if state == StateDsct::Validator {
			let contribution = self.gen_random_contribution(sender_addr, sender_pwd, receiver_addr, receiver_pwd,
				&mut RandRange::new(100, 1000));

			match self.hydrabadger.push_user_contribution(contribution) {
				Err(HydrabadgerError::PushUserContributionNotValidator) => {
					debug!("Unable to push contribution: this node is not a validator");
				},
				Err(err) => unreachable!(),
				Ok(()) => {},
			}
		} else {
			debug!("Unable to generate contribution: this node is not a validator");
		}
	}

	fn play_with_blocks(&self) {
		let mut rng = OsRng::new().expect("Error creating OS Rng");
		let mut value_range = RandRange::new(100, TXN_AMOUNT_MAX);

		let sender_addr = Address::from(RICHIE_ACCT);
		let sender_pwd = Password::from(RICHIE_PWD);
		let receiver_addr = self.account_addr;

		match self.account_provider.test_password(&sender_addr, &sender_pwd) {
			Ok(false) => panic!("Bad password while playing with blocks."),
			Ok(true) => {},
			Err(ethstore::Error::InvalidAccount) => {
				error!("Transaction sender account does not exist. Skipping playing with blocks.");
				return;
			},
			err => panic!("{:?}", err),
		}

		let block_author = Address::default();
		let gas_range_target = (3141562.into(), 31415620.into());
		let extra_data = vec![];

		let mut sender_acct_nonce: U256 = self.client.state().nonce(&sender_addr)
			.unwrap_or_else(|_| panic!("Unable to determine nonce for account: {}", sender_addr));

		// Import some blocks:
		for i in 0..0 {
			let mut open_block: OpenBlock = self.client
				.prepare_open_block(block_author, gas_range_target, extra_data.clone())
				.unwrap();

			let txn = self.gen_random_txn(sender_acct_nonce, sender_addr, sender_pwd.clone(), receiver_addr,
				&mut value_range, &mut rng);
			sender_acct_nonce += 1.into();

			open_block.push_transaction(txn, None).unwrap();

			let closed_block: ClosedBlock = open_block.close().unwrap();
			let reopened_block: OpenBlock = closed_block.reopen(self.client.engine());
			let reclosed_block: ClosedBlock = reopened_block.close().unwrap();
			let locked_block: LockedBlock = reclosed_block.lock();
			let sealed_block: SealedBlock = locked_block.seal(self.client.engine(), vec![]).unwrap();

			self.client.import_sealed_block(sealed_block).unwrap();
		}

		// Import some blocks:
		for _ in 0..1 {
			let miner = self.client.miner();
			let mut open_block: OpenBlock = miner.prepare_new_block(&*self.client).unwrap();

			let txn: SignedTransaction = self.gen_random_txn(sender_acct_nonce, sender_addr, sender_pwd.clone(),
				receiver_addr, &mut value_range, &mut rng);
			sender_acct_nonce += 1.into();

			let min_tx_gas = u64::max_value().into();
			let block: ClosedBlock = miner.prepare_block_from(open_block, vec![txn], &*self.client, min_tx_gas).unwrap();

			info!("Importing block {} (#{}, experimentally generated)", block.hash(), block.block().header.number());
			if !miner.seal_and_import_block_internally(&*self.client, block) {
				warn!("Failed to seal and import block.");
			}
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


/// Handles honey badger batch outputs.
//
// TODO: Create a transaction queue semaphore to allow/disallow transactions
// from being streamed into hydrabadger and manipulate its state from here.
struct BatchHandler {
	batch_rx: BatchRx<Contribution>,
	client: Weak<Client>,
	hydrabadger: Hydrabadger<Contribution>,
}

impl BatchHandler {
	fn new(batch_rx: BatchRx<Contribution>, client: Weak<Client>, hydrabadger: Hydrabadger<Contribution>) -> BatchHandler {
		BatchHandler { batch_rx, client, hydrabadger }
	}

	/// Handles a batch of transactions output by the Honey Badger BFT
	/// algorithm.
	fn handle_batch(&mut self, batch: Batch<Contribution, NodeId>) {
		let epoch = batch.epoch();
		let block_num = epoch + 1;

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

		// TODO (c0gent/drpete): Make sure this produces identical blocks in
		// all validators. (Probably at least `params.author` needs to be
		// changed.)
		let mut open_block = miner.prepare_new_block(&*client).expect("TODO");
		if open_block.header().number() == block_num {
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
				block.hash(), block.block().header.number(), batch.epoch(), txn_count);

			// TODO: Does this remove the block's transactions from the queue? If
			// not, we need to do so.
			//
			// TODO (afck/drpete): Replace instant sealing with a threshold signature.
			if !miner.seal_and_import_block_internally(&*client, block) {
				warn!("Failed to seal and import block.");
			}
		} else if open_block.header().number() < block_num {
			println!("Can't produce block: missing parent.");
		} else {
			println!("Block {} already imported.", block_num);
		}

		// client.clear_queue();
		// client.flush_queue();

		// Select new transactions and propose them for the next block.
		//
		// TODO (c0gent): Pull this from cfg.
		let batch_size = 50;
		// TODO (c0gent): `batch_size / num_validators`
		let contrib_size = batch_size / 5;

		let pending = miner.pending_transactions_from_queue(&*client, batch_size);

		// miner.clear();

		let mut rng = rand::thread_rng();
		let txns = if pending.len() <= contrib_size {
			pending
		} else {
			rand::seq::sample_slice(&mut rng, &pending, contrib_size)
		};
		let ser_txns: Vec<_> = txns.into_iter().map(|txn| txn.signed().rlp_bytes()).collect();
		let contribution = Contribution {
			transactions: ser_txns,
			timestamp: UNIX_EPOCH.elapsed().expect("Valid time has to be set in your system.").as_secs(),
		};
		info!("Proposing {} transactions for epoch {}.", contribution.transactions.len(), batch.epoch() + 1);

		self.hydrabadger.push_user_contribution(contribution).expect("TODO");
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


/// You can use this to create an account within Parity. This method does the exact same
/// thing as using the JSON-RPC to create an account. The password and passphrase will be
/// set to the account name e.g. "richie" or "node0".
fn create_account(account_provider: &Arc<AccountProvider>, name: &str)
	-> Result<(Address, Password), ethstore::Error>
{
	let passphrase = name.to_string();
	let pwd = Password::from(name);
	let key_pair = Brain::new(passphrase).generate().unwrap();
	let sk = key_pair.secret().clone();
	let addr = account_provider.insert_account(sk, &pwd)?;
	Ok((addr, pwd))
}


/// Creates a node-specific account to use with generated transactions in
/// addition to unlocking the "richie" account.
fn initialize_accounts(account_provider: &Arc<AccountProvider>, name: &str)
	-> Result<(Address, Password), Error>
{
	let (addr, pwd) = create_account(account_provider, name)
		.map_err(|err| ErrorKind::EthstoreAccountInitNode(err))?;
	account_provider.unlock_account_permanently(addr, pwd.clone())
		.map_err(|err| ErrorKind::EthstoreAccountInitNode(err))?;

	let (richie_addr, richie_pwd) = create_account(account_provider, RICHIE_PWD)
		.map_err(|err| ErrorKind::EthstoreAccountInitRichie(err))?;
	assert!(richie_addr == Address::from(RICHIE_ACCT) && richie_pwd == Password::from(RICHIE_PWD));
	account_provider.unlock_account_permanently(richie_addr, richie_pwd)
		.map_err(|err| ErrorKind::EthstoreAccountInitRichie(err))?;

	Ok((addr, pwd))
}


/// An hbbft <-> Parity link which relays events and acts as an intermediary.
pub struct HbbftDaemon {
	// Unused:
	client: Weak<Client>,

	hydrabadger: Hydrabadger<Contribution>,

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
		let hydrabadger = Hydrabadger::<Contribution>::new(cfg.bind_address, cfg.to_hydrabadger());

		let batch_handler = BatchHandler::new(
			hydrabadger.batch_rx()
				.expect("The Hydrabadger batch receiver can not be `None` immediately after creation; qed \
					These proofs are bullshit and prove nothing; qed"),
			Arc::downgrade(&client),
			hydrabadger.clone(),
		);

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

		executor.spawn(batch_handler.map_err(|err| panic!("Unhandled batch handler error: {:?}", err)));

		info!("HbbftDaemon has been spawned.");

		// Set up an account to use for txn gen.
		let (account_addr, account_pwd) = initialize_accounts(&account_provider,
			&cfg.bind_address.to_string())?;

		let client_clone = client.clone();
		let hdb_clone = hydrabadger.clone();
		let cfg_clone = cfg.clone();

		let lab = Laboratory {
			client: client.clone(),
			hydrabadger: hydrabadger.clone(),
			hdb_cfg: cfg.clone(),
			account_provider,
			account_addr,
			account_pwd,
		};

		// Spawn experimentation loop:
		executor.spawn(future::loop_fn(lab, move |mut lab| {
			// Entry point for experiments:
			lab.run_experiments();

			Delay::new(Instant::now() + Duration::from_millis(7000))
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
