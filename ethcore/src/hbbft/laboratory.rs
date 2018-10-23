//! Experiments and test stuff.

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
use parity_runtime::Runtime;
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
use transaction::{Transaction, Action, SignedTransaction, Error as TransactionError};
use block::{OpenBlock, ClosedBlock, IsBlock, LockedBlock, SealedBlock};
use state::{self, State, CleanupMode};
use account_provider::AccountProvider;
use super::hbbft_daemon::{HbbftDaemon, Contribution, Error, ErrorKind, HbbftClientExt, CONTRIBUTION_PUSH_DELAY_MS};

const RICHIE_ACCT: &'static str = "0x002eb83d1d04ca12fe1956e67ccaa195848e437f";
const RICHIE_PWD: &'static str =  "richie";
// const NODE0_ACCT: &'static str = "0x00bd138abd70e2f00903268f3db08f2d25677c9e";
// const NODE0_PWD: &'static str =  "node0";

const TXN_AMOUNT_MAX: usize = 1000;


///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
//////////////////////////////// EXPERIMENTS //////////////////////////////////
///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////


/// You can use this to create an account within Parity. This method does the exact same
/// thing as using the JSON-RPC to create an account. The password and passphrase will be
/// set to the account name e.g. "richie" or "node0".
fn create_account(account_provider: &AccountProvider, name: &str)
	-> Result<(Address, Password), ethstore::Error>
{
	let passphrase = name.to_string();
	let pwd = Password::from(name);
	let key_pair = Brain::new(passphrase).generate().unwrap();
	let sk = key_pair.secret().clone();
	let addr = account_provider.insert_account(sk, &pwd)?;
	Ok((addr, pwd))
}

/// Account info.
#[derive(Clone, Debug)]
struct Account {
	address: Address,
	password: Password,
	balance: U256,
	nonce: U256,
	/// The number of times this account has been out of sync.
	//
	// TODO (c0gent): Eliminate this field and ensure that transactions can
	// not get lost.
	retries: usize,
}


/// Node-specific accounts to be used in transaction generation.
#[derive(Clone, Debug)]
pub(super) struct Accounts {
	accounts: Vec<Account>,
	stage_size: usize,
	stage_count: usize,
	next_stage: usize,
}

impl Accounts {
	pub(super) fn new(account_provider: &AccountProvider, client: &Client, node_id: &str, txn_gen_count: usize,
		stage_count: usize) -> Result<Accounts, Error>
	{
		let (richie_addr, richie_pwd) = create_account(account_provider, RICHIE_PWD)
			.map_err(|err| ErrorKind::EthstoreAccountInitRichie(err))?;
		assert!(richie_addr == Address::from(RICHIE_ACCT) && richie_pwd == Password::from(RICHIE_PWD));
		account_provider.unlock_account_permanently(richie_addr, richie_pwd)
			.map_err(|err| ErrorKind::EthstoreAccountInitRichie(err))?;

		let accounts = (0..(txn_gen_count * stage_count)).map(|i| {
			let name = format!("{}_{}", node_id, i);
			let (address, password) = create_account(account_provider, &name)
				.map_err(|err| ErrorKind::EthstoreAccountInitNode(err))?;
			account_provider.unlock_account_permanently(address, password.clone())
				.map_err(|err| ErrorKind::EthstoreAccountInitNode(err))?;
			let balance = client.state().balance(&address).unwrap();
			let nonce = client.state().nonce(&address).unwrap();
			debug!("######## Accounts::new: Account created with name: {}", name);
			Ok(Account { address, password, balance, nonce, retries: 0 })
		}).collect::<Result<Vec<_>, Error>>()?;

		Ok(Accounts {
			accounts,
			stage_size: txn_gen_count,
			stage_count,
			next_stage: 0,
		})
	}

	fn account_mut(&mut self, address: &Address) -> Option<&mut Account> {
		self.accounts.iter_mut().find(|acc| &acc.address == address)
	}

	/// Returns the first account with a balance below `balance`.
	fn account_below(&self, balance: U256) -> Option<&Account> {
		self.accounts.iter().find(|acc| acc.balance < balance)
	}

	fn accounts(&self) -> &[Account] {
		&self.accounts
	}

	/// Returns a slice of the accounts in the 'stage' specified.
	fn next_stage(&self) -> &[Account] {
		let idz = self.next_stage * self.stage_size;
		let idn = idz + self.stage_size;
		&self.accounts[idz..idn]
	}

	/// Increments the stage counter.
	fn incr_stage(&mut self) {
		self.next_stage += 1;
		if self.next_stage == self.stage_count { self.next_stage = 0 }
	}
}


/// Experiments and other junk.
//
// Add anything at all to this!
//
pub(super) struct Laboratory {
	pub(super) client: Arc<Client>,
	pub(super) hydrabadger: Hydrabadger<Contribution>,
	pub(super) hdb_cfg: HbbftConfig,
	pub(super) account_provider: Arc<AccountProvider>,
	pub(super) accounts: Accounts,
	pub(super) block_counter: Arc<AtomicIsize>,
	pub(super) last_block: isize,
	pub(super) gen_counter: usize,
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
		value_range: &mut RandRange<usize>, rng: &mut OsRng) -> (Address, SignedTransaction)
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

		debug!("######## LABORATORY: Transaction generated: {:?}", txn);

		(sender, self.sign_txn(sender, sender_pwd, txn))
	}

	/// Panics if the account does not exist, if the password is incorrect, or
	/// on any other error.
	fn test_password(&self, addr: &Address, pwd: &Password) {
		match self.account_provider.test_password(addr, pwd) {
			Ok(false) => panic!("Bad password while pushing random transactions to Hydrabadger."),
			Ok(true) => {},
			Err(ethstore::Error::InvalidAccount) => {
				panic!("Transaction sender account does not exist. Skipping hydrabadger contribution push.");
			},
			err => panic!("{:?}", err),
		}
	}

	/// Generates a set of random-ish transactions.
	///
	/// If any account in `self.accounts` is below a minimum balance, this
	/// will generate a transaction to send money to it. Currently this
	/// process can fail.
	fn gen_random_transactions(&mut self, receiver: Address, receiver_pwd: Password,
		value_range: &mut RandRange<usize>) -> Vec<SignedTransaction>
	{
		let mut rng = OsRng::new().expect("Error creating OS Rng");

		// Determine the pseudo node id:
		let node_id = self.hdb_cfg.bind_address.port() % 100;

		// Add ourselves to the count.
		let validator_count = 1 + self.hydrabadger.peers().count_validators() as u64;

		// This is total hackfoolery to ensure that each node's sender account
		// gets a starting balance (will break when nodes > 3):
		let txns = match self.accounts.account_below(U256::from(TXN_AMOUNT_MAX)).cloned() {
			// If an account is below the minimum and it's 'our turn' (sketchy):
			Some(ref acct) => {
				debug!("######## LABORATORY: An account is below the minimum balance.");
				if U256::from(node_id) == (self.gen_counter as u64 % validator_count).into() {
					let receiver_nonce = self.client.state().nonce(&receiver)
						.unwrap_or_else(|_| panic!("Unable to determine nonce for account: {}", receiver));

					info!("######## LABORATORY: Sending funds to {:?}", acct.address);
					// Add a contribution to initialize account:
					let amt = (1000000000000000000 as u64).into();
					self.accounts.account_mut(&acct.address).unwrap().balance += amt;

					vec![self.sign_txn(receiver, receiver_pwd.clone(), Transaction {
						action: Action::Call(acct.address),
						nonce: receiver_nonce,
						gas_price: 0.into(),
						gas: self.client.miner().sensible_gas_limit(),
						value: amt,
						data: vec![],
					})]
				} else {
					info!("########### LABORATORY: Not sending funds. \
						(node_id: {}, gen_counter: {}, validator_count: {}, gen_counter % validator_count: {})",
						node_id, self.gen_counter, validator_count, self.gen_counter as u64 % validator_count);
					vec![]
				}
			},
			_ => {
				debug!("######## LABORATORY: All accounts above minimum balance.");
				let mut txns = Vec::with_capacity(8);

				for sender in self.accounts.next_stage() {
					// Ensure there is enough balance in the sender account:
					if sender.balance >= U256::from(TXN_AMOUNT_MAX) {
						//  TODO: Use `miner.next_nonce<C>(&self, chain: &C,
						//  address: &Address)` instead.
						let sender_nonce = self.client.state().nonce(&sender.address)
							.unwrap_or_else(|err| panic!("Unable to determine nonce for account: {} ({:?})",
								sender.address, err));

						// Generate random txns normally:
						let txn = self.gen_random_txn(sender_nonce, sender.address, sender.password.clone(),
							receiver, value_range, &mut rng);
						txns.push(txn);
					} else {
						panic!("######## LABORATORY: Account with insufficient balance: {}", sender.address);
					}
				}
				self.accounts.incr_stage();

				let txns = txns.into_iter().map(|(sender, txn)| {
					// Adjust cached account balance and nonce:
					let acct = self.accounts.account_mut(&sender).unwrap();
					acct.balance -= txn.value;
					acct.nonce += 1.into();
					txn
				}).collect::<Vec<_>>();

				info!("######## LABORATORY: {} transactions generated", txns.len());

				txns
			}
		};

		txns
	}

	fn export_transactions_to_miner(&mut self) {
		let block_counter = self.block_counter.load(Ordering::Acquire);

		// Don't do anything if hydrabadger is not connected as a validator or
		// until the block progresses (ensures that we don't generate a new
		// contribution until the previous one is imported by the miner).
		if !self.hydrabadger.is_validator() {
			debug!("Unable to generate contribution: this node is not a validator");
			return;
		} else if !(self.last_block < block_counter || (self.last_block == 0 && block_counter == -1)) {
			info!("####### LABORATORY: Block state has not progressed. Cancelling contribution push. \
				(self.last_block: {}, block_counter: {})", self.last_block, block_counter);
			return;
		}

		debug!("######## LABORATORY: Checking account data...");

		// Keep account data up to date:
		for acct in self.accounts.accounts.iter_mut() {
			let balance_state = self.client.state().balance(&acct.address).unwrap();
			let nonce_state = self.client.state().nonce(&acct.address).unwrap();

			if balance_state != acct.balance || nonce_state != acct.nonce {
				acct.retries += 1;
			}

			if acct.retries == 3 {
				debug!("######## LABORATORY: Refreshing account info for: {}", acct.address);
				acct.balance = self.client.state().balance(&acct.address).unwrap();
				acct.nonce = self.client.state().nonce(&acct.address).unwrap();
				acct.retries = 0;
			}
		}

		let receiver_addr = Address::from(RICHIE_ACCT);
		let receiver_pwd = Password::from(RICHIE_PWD);

		// Ensure all of our accounts are set up properly:
		for acct in self.accounts.accounts().iter() {
			self.test_password(&acct.address, &acct.password);
		}
		self.test_password(&receiver_addr, &receiver_pwd);

		debug!("######## LABORATORY: Generating transactions...");

		let txns = self.gen_random_transactions(receiver_addr, receiver_pwd,
			&mut RandRange::new(100, 1000));

		if !txns.is_empty() {
			// Update our 'last_block' (it may skip blocks).
			self.last_block = if block_counter == -1 { 0 } else { block_counter };
		}

		for txn in txns {
			match self.client.miner().import_claimed_local_transaction(&*self.client, txn.into(), false) {
				Ok(()) => {},
				Err(TransactionError::AlreadyImported) => {},
				// TODO: Remove this at some point:
				Err(TransactionError::Old) => {},
				err => panic!("Unable to import generated transaction: {:?}", err),
			}
		}

		self.gen_counter = self.gen_counter.wrapping_add(1);
	}

	fn play_with_blocks(&self) {
		let mut rng = OsRng::new().expect("Error creating OS Rng");
		let mut value_range = RandRange::new(100, TXN_AMOUNT_MAX);

		let sender_addr = Address::from(RICHIE_ACCT);
		let sender_pwd = Password::from(RICHIE_PWD);
		let receiver_addr = self.accounts.accounts()[0].address;

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

			let (_, txn) = self.gen_random_txn(sender_acct_nonce, sender_addr, sender_pwd.clone(), receiver_addr,
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

			let (_, txn) = self.gen_random_txn(sender_acct_nonce, sender_addr, sender_pwd.clone(),
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
	pub(super) fn run_experiments(&mut self) {
		self.export_transactions_to_miner();
		// self.play_with_blocks();
		self.demonstrate_client_extension_methods();
	}

	pub(super) fn into_loop(self) -> impl Future<Item = (), Error = ()> + Send {
		future::loop_fn(self, |mut lab| {
			// Entry point for experiments:
			lab.run_experiments();

			let loop_delay = CONTRIBUTION_PUSH_DELAY_MS * 50;

			Delay::new(Instant::now() + Duration::from_millis(loop_delay))
				.map(|_| Loop::Continue(lab))
				.map_err(|err| panic!("{:?}", err))
		})
	}

}
