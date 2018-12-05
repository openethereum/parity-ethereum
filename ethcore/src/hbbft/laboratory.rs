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
use rustc_hex::FromHex;
use ethstore;
use ethjson::misc::AccountMeta;
use ethkey::{Brain, Generator, Password, Random};
use ethereum_types::{U256, H256, Address};
use types::{ids::TransactionId, receipt::LocalizedReceipt};
use header::Header;
use client::{BlockChainClient, Client, ClientConfig, BlockId, ChainInfo, BlockInfo, PrepareOpenBlock,
	ImportSealedBlock, ImportBlock, CallContract};
use miner::{Miner, MinerService};
use verification::queue::kind::blocks::{Unverified};
use transaction::{Transaction, Action, SignedTransaction, Error as TransactionError};
use block::{OpenBlock, ClosedBlock, IsBlock, LockedBlock, SealedBlock};
use state::{self, State, CleanupMode};
use account_provider::AccountProvider;
use ethabi::FunctionOutputDecoder;
use super::daemon::{HbbftDaemon, Contribution, Error, ErrorKind, HbbftClientExt};

const RICHIE_ACCT: &'static str = "0x002eb83d1d04ca12fe1956e67ccaa195848e437f";
const RICHIE_PWD: &'static str =  "richie";
// const NODE0_ACCT: &'static str = "0x00bd138abd70e2f00903268f3db08f2d25677c9e";
// const NODE0_PWD: &'static str =  "node0";

const TXN_AMOUNT_MAX: usize = 1000;

use_contract!(test_junk_contract, "res/contracts/hbbft/test_junk_contract.json");


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
	contract_acct_idx: usize,
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

		let accounts = (0..((txn_gen_count * stage_count) + 1)).map(|i| {
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

		let contract_acct_idx = accounts.len() - 1;

		Ok(Accounts {
			accounts,
			stage_size: txn_gen_count,
			stage_count,
			next_stage: 0,
			contract_acct_idx,
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

	/// Returns the account used for creating contracts.
	fn contract_account(&self) -> &Account {
		&self.accounts[self.contract_acct_idx]
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


/// Contract testing.
pub(super) struct Contract {
	deploy_txn_id: Option<TransactionId>,
	receipt: Option<LocalizedReceipt>,
	new_owner_address: Option<(Address, TransactionId)>,
}

impl Contract {
	pub fn new() -> Contract {
		Contract {
			deploy_txn_id: None,
			receipt: None,
			new_owner_address: None,
		}
	}

	fn create_txn(&self, client: &Client) -> Transaction {
		Transaction {
			action: Action::Create,
			nonce: 0.into(),
			gas_price: 0.into(),
			gas: client.miner().sensible_gas_limit(),
			value: 0.into(),
			data: TEST_CONTRACT_BINARY.from_hex().unwrap(),
		}
	}
}


/// Experiments and other junk.
//
// Add anything at all to this!
//
pub(super) struct Laboratory {
	client: Weak<Client>,
	hydrabadger: Hydrabadger<Contribution>,
	hdb_cfg: HbbftConfig,
	account_provider: Arc<AccountProvider>,
	accounts: Accounts,
	block_counter: Arc<AtomicIsize>,
	last_block: isize,
	gen_counter: usize,
	contract: Contract,
}

impl Laboratory {
	pub fn new(
		client: Weak<Client>,
		hydrabadger: Hydrabadger<Contribution>,
		hdb_cfg: HbbftConfig,
		account_provider: Arc<AccountProvider>,
		accounts: Accounts,
		block_counter: Arc<AtomicIsize>,
	) -> Laboratory {
		Laboratory {
			client,
			hydrabadger,
			hdb_cfg,
			account_provider,
			accounts,
			block_counter,
			last_block: 0,
			gen_counter: 0,
			contract: Contract::new(),
		}
	}

	/// Returns each Parity account's address and metadata.
	fn get_accounts(&self) -> HashMap<Address, AccountMeta> {
		self.account_provider.accounts_info().unwrap()
	}

	/// Converts an unsigned `Transaction` to a `SignedTransaction`.
	fn sign_txn(&self, sender: Address, password: Password, chain_id: Option<u64>, txn: Transaction) -> SignedTransaction {
		// let chain_id = self.client.signing_chain_id();
		let txn_hash = txn.hash(chain_id);
		let sig = self.account_provider.sign(sender, Some(password), txn_hash)
			.unwrap_or_else(|e| panic!("[hbbft-lab] failed to sign txn: {:?}", e));
		let unverified_txn = txn.with_signature(sig, chain_id);
		SignedTransaction::new(unverified_txn).unwrap()
	}

	/// Generates a random-ish transaction.
	fn gen_random_txn(&self, nonce: U256, sender: Address, sender_pwd: Password, receiver: Address,
		chain_id: Option<u64>, value_range: &mut RandRange<usize>, rng: &mut OsRng) -> (Address, SignedTransaction)
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

		(sender, self.sign_txn(sender, sender_pwd, chain_id, txn))
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
		value_range: &mut RandRange<usize>, client: &Client) -> Vec<SignedTransaction>
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
					let receiver_nonce = client.state().nonce(&receiver)
						.unwrap_or_else(|_| panic!("Unable to determine nonce for account: {}", receiver));

					info!("######## LABORATORY: Sending funds to {:?}", acct.address);
					// Add a contribution to initialize account:
					let amt = (1000000000000000000 as u64).into();
					self.accounts.account_mut(&acct.address).unwrap().balance += amt;

					vec![self.sign_txn(receiver, receiver_pwd.clone(), client.signing_chain_id(),
						Transaction {
							action: Action::Call(acct.address),
							nonce: receiver_nonce,
							gas_price: 0.into(),
							gas: client.miner().sensible_gas_limit(),
							value: amt,
							data: vec![],
						}
					)]
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
						let sender_nonce = client.state().nonce(&sender.address)
							.unwrap_or_else(|err| panic!("Unable to determine nonce for account: {} ({:?})",
								sender.address, err));

						// Generate random txns normally:
						let txn = self.gen_random_txn(sender_nonce, sender.address, sender.password.clone(),
							receiver, client.signing_chain_id(), value_range, &mut rng);
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

		let client = match self.client.upgrade() {
			Some(client) => client,
			None => return,
		};

		debug!("######## LABORATORY: Checking account data...");

		// Keep account data up to date:
		for acct in self.accounts.accounts.iter_mut() {
			let balance_state = client.state().balance(&acct.address).unwrap();
			let nonce_state = client.state().nonce(&acct.address).unwrap();

			if balance_state != acct.balance || nonce_state != acct.nonce {
				acct.retries += 1;
			}

			if acct.retries == 3 {
				debug!("######## LABORATORY: Refreshing account info for: {}", acct.address);
				acct.balance = client.state().balance(&acct.address).unwrap();
				acct.nonce = client.state().nonce(&acct.address).unwrap();
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
			&mut RandRange::new(100, 1000), &client);

		if !txns.is_empty() {
			// Update our 'last_block' (it may skip blocks).
			self.last_block = if block_counter == -1 { 0 } else { block_counter };
		}

		for txn in txns {
			match client.miner().import_claimed_local_transaction(&*client, txn.into(), false) {
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

		let client = match self.client.upgrade() {
			Some(client) => client,
			None => return,
		};

		let mut sender_acct_nonce: U256 = client.state().nonce(&sender_addr)
			.unwrap_or_else(|_| panic!("Unable to determine nonce for account: {}", sender_addr));

		// Import some blocks:
		for i in 0..0 {
			let mut open_block: OpenBlock = client
				.prepare_open_block(block_author, gas_range_target, extra_data.clone())
					.unwrap();

			let (_, txn) = self.gen_random_txn(sender_acct_nonce, sender_addr, sender_pwd.clone(), receiver_addr,
				client.signing_chain_id(), &mut value_range, &mut rng);
			sender_acct_nonce += 1.into();

			open_block.push_transaction(txn, None).unwrap();

			let closed_block: ClosedBlock = open_block.close().unwrap();
			let reopened_block: OpenBlock = closed_block.reopen(client.engine());
			let reclosed_block: ClosedBlock = reopened_block.close().unwrap();
			let locked_block: LockedBlock = reclosed_block.lock();
			let sealed_block: SealedBlock = locked_block.seal(client.engine(), vec![]).unwrap();

			client.import_sealed_block(sealed_block).unwrap();
		}

		// Import some blocks:
		for _ in 0..1 {
			let miner = client.miner();
			let mut open_block: OpenBlock = miner.prepare_new_block(&*client).unwrap();

			let (_, txn) = self.gen_random_txn(sender_acct_nonce, sender_addr, sender_pwd.clone(),
				receiver_addr, client.signing_chain_id(), &mut value_range, &mut rng);
			sender_acct_nonce += 1.into();

			let min_tx_gas = u64::max_value().into();
			let block: ClosedBlock = miner.prepare_block_from(open_block, vec![txn], &*client, min_tx_gas).unwrap();

			info!("Importing block {} (#{}, experimentally generated)", block.hash(), block.block().header.number());
			if !miner.seal_and_import_block_internally(&*client, block) {
				warn!("Failed to seal and import block.");
			}
		}
	}

	fn demonstrate_client_extension_methods(&self) {
		let client = match self.client.upgrade() {
			Some(client) => client,
			None => return,
		};

		client.a_specialized_method();

		client.change_me_into_something_useful();
	}

	/// Mess with contracts.
	fn contract_stuff(&mut self) {
		if !self.hydrabadger.is_validator() { return; }

		let client = match self.client.upgrade() {
			Some(client) => client,
			None => return,
		};

		// Deploy contract:
		if self.contract.deploy_txn_id.is_none() {
			let sender = self.accounts.contract_account();
			let sender_nonce = client.state().nonce(&sender.address)
				.unwrap_or_else(|_| panic!("Unable to determine nonce for account: {}", sender.address));

			let txn_signed = self.sign_txn(sender.address, sender.password.clone(), client.signing_chain_id(),
				Transaction {
					action: Action::Create,
					nonce: sender_nonce,
					gas_price: 0.into(),
					gas: client.miner().sensible_gas_limit(),
					value: 0.into(),
					data: TEST_CONTRACT_BINARY.from_hex().unwrap(),
				}
			);

			let txn_hash = txn_signed.hash();

			match client.miner().import_claimed_local_transaction(&*client, txn_signed.into(), false) {
				Ok(()) => {},
				Err(ref err) => error!("Unable to import deploy transaction: {:?}", err),
			}

			self.contract.deploy_txn_id = Some(TransactionId::Hash(txn_hash));
			info!("Test contract deployed: {}", txn_hash);
		}

		if self.contract.receipt.is_none() {
			// Wait for receipt.
			if let Some(receipt) = client.transaction_receipt(self.contract.deploy_txn_id.clone().unwrap()) {
				match receipt.contract_address.clone() {
					Some(addr) => {
						info!("Test contract created with address: {}", addr);
						self.contract.receipt = Some(receipt);
					},
					None => panic!("Contract creation transaction has no contract address"),
				}
			}
		} else {
			// Verify owner address.
			let receipt = self.contract.receipt.as_ref().unwrap();
			// let block = BlockId::Number(receipt.block_number);
			let block = BlockId::Latest;
			let address = receipt.contract_address.clone().unwrap();
			let (data, decoder) = test_junk_contract::functions::get_owner::call();
			let value = client.call_contract(block, address, data)
				.expect("Error calling test contract");
			let owner_addr = decoder.decode(&value)
				.expect("Error decoding test contract return value");
			info!("Test contract owner address: {} (orig: {})", owner_addr, self.accounts.contract_account().address);

			match self.contract.new_owner_address {
				Some((new_addr, ref txn_id)) => {
					// Presumably this could fail if the block state changes
					// between the above call to `::call_contract` and this
					// call to `::transaction_receipt`.
					match client.transaction_receipt(txn_id.clone()) {
						Some(_receipt) => assert_eq!(owner_addr, new_addr),
						None => assert_eq!(owner_addr, self.accounts.contract_account().address),
					}
				},
				None => {
					assert_eq!(owner_addr, self.accounts.contract_account().address)
				},
			}
		}

		// Modify owner address.
		if self.contract.receipt.is_some() && self.contract.new_owner_address.is_none() {
			let sender = self.accounts.contract_account();
			let sender_nonce = client.state().nonce(&sender.address)
				.unwrap_or_else(|_| panic!("Unable to determine nonce for account: {}", sender.address));
			let contract_addr = self.contract.receipt.as_ref().unwrap().contract_address.clone().unwrap();
			let new_addr = self.accounts.accounts()[0].address.clone();
			let data = test_junk_contract::functions::set_owner::encode_input(new_addr);

			let txn_signed = self.sign_txn(sender.address, sender.password.clone(), client.signing_chain_id(),
				Transaction {
					action: Action::Call(contract_addr),
					nonce: sender_nonce,
					gas_price: 0.into(),
					gas: client.miner().sensible_gas_limit(),
					value: 0.into(),
					data,
				}
			);

			let txn_hash = txn_signed.hash();

			match client.miner().import_claimed_local_transaction(&*client, txn_signed.into(), false) {
				Ok(()) => {},
				Err(ref err) => error!("Unable to import setOwner transaction: {:?}", err),
			}

			info!("Test contract: Setting owner address to {} (orig: {})", new_addr,
				self.accounts.contract_account().address);

			self.contract.new_owner_address = Some((new_addr, TransactionId::Hash(txn_hash)));
		}
	}

	/// Runs all experiments.
	//
	// Call your experiments here.
	pub(super) fn run_experiments(&mut self) {
		self.export_transactions_to_miner();
		// self.play_with_blocks();
		self.demonstrate_client_extension_methods();

		self.contract_stuff();
	}

	pub(super) fn into_loop(self) -> impl Future<Item = (), Error = ()> + Send {
		future::loop_fn(self, |mut lab| {
			// Entry point for experiments:
			lab.run_experiments();

			let loop_delay = lab.hdb_cfg.contribution_delay_ms * 50;

			Delay::new(Instant::now() + Duration::from_millis(loop_delay))
				.map(|_| Loop::Continue(lab))
				.map_err(|err| panic!("{:?}", err))
		})
	}

}

const TEST_CONTRACT_BINARY: &str = r#"608060405234801561001057600080fd5b5060008054600160a060020a03191633179055610142806100326000396000f30060806040526004361061004b5763ffffffff7c010000000000000000000000000000000000000000000000000000000060003504166313af40358114610050578063893d20e814610080575b600080fd5b34801561005c57600080fd5b5061007e73ffffffffffffffffffffffffffffffffffffffff600435166100be565b005b34801561008c57600080fd5b506100956100fa565b6040805173ffffffffffffffffffffffffffffffffffffffff9092168252519081900360200190f35b6000805473ffffffffffffffffffffffffffffffffffffffff191673ffffffffffffffffffffffffffffffffffffffff92909216919091179055565b60005473ffffffffffffffffffffffffffffffffffffffff16905600a165627a7a72305820d57cdcf8acc8736a2ad737c6c4b5b69e7fefe49b812f7a110f52cfb31b819b4a0029"#;

const TEST_CONTRACT_GAS: usize = 4700000;

/*********************** TEST CONTRACT SOURCE **********************

pragma solidity ^0.4.25;

contract MyContract {
    address owner;

    constructor() public {
        owner = msg.sender;
    }

    function getOwner() public constant returns(address) {
        return owner;
    }

    function setOwner(address newOwner) public {
        owner = newOwner;
    }
}

***************************************************************/
