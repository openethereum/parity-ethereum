// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Private transactions module.

// Recursion limit required because of
// error_chain foreign_links.
#![recursion_limit="256"]

mod encryptor;
mod private_transactions;
mod messages;
mod error;

extern crate ethcore;
extern crate ethcore_bytes as bytes;
extern crate ethcore_crypto as crypto;
extern crate ethcore_io as io;
extern crate ethcore_miner;
extern crate ethcore_transaction as transaction;
extern crate ethabi;
extern crate ethereum_types;
extern crate ethkey;
extern crate ethjson;
extern crate fetch;
extern crate futures;
extern crate keccak_hash as hash;
extern crate parking_lot;
extern crate patricia_trie as trie;
extern crate rlp;
extern crate url;
extern crate rustc_hex;
#[macro_use]
extern crate log;
#[macro_use]
extern crate ethabi_derive;
#[macro_use]
extern crate ethabi_contract;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate rlp_derive;

#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate ethcore_logger;

pub use encryptor::{Encryptor, SecretStoreEncryptor, EncryptorConfig, NoopEncryptor};
pub use private_transactions::{PrivateTransactionDesc, VerificationStore, PrivateTransactionSigningDesc, SigningStore};
pub use messages::{PrivateTransaction, SignedPrivateTransaction};
pub use error::{Error, ErrorKind};

use std::sync::{Arc, Weak};
use std::collections::{HashMap, HashSet};
use ethereum_types::{H128, H256, U256, Address};
use hash::keccak;
use rlp::*;
use parking_lot::{Mutex, RwLock};
use bytes::Bytes;
use ethkey::{Signature, recover, public_to_address};
use io::IoChannel;
use ethcore::executive::{Executive, TransactOptions};
use ethcore::executed::{Executed};
use transaction::{SignedTransaction, Transaction, Action, UnverifiedTransaction};
use ethcore::{contract_address as ethcore_contract_address};
use ethcore::client::{
	Client, ChainNotify, ChainMessageType, ClientIoMessage, BlockId,
	MiningBlockChainClient, ChainInfo, Nonce, CallContract
};
use ethcore::account_provider::AccountProvider;
use ethcore_miner::transaction_queue::{TransactionDetailsProvider as TransactionQueueDetailsProvider, AccountDetails};
use ethcore::miner::MinerService;
use ethcore::trace::{Tracer, VMTracer};
use rustc_hex::FromHex;

// Source avaiable at https://github.com/parity-contracts/private-tx/blob/master/contracts/PrivateContract.sol
const DEFAULT_STUB_CONTRACT: &'static str = include_str!("../res/private.evm");

use_contract!(private, "PrivateContract", "res/private.json");

/// Initialization vector length.
const INIT_VEC_LEN: usize = 16;

struct TransactionDetailsProvider<'a> {
	client: &'a MiningBlockChainClient,
}

impl<'a> TransactionDetailsProvider<'a> {
	pub fn new(client: &'a MiningBlockChainClient) -> Self {
		TransactionDetailsProvider {
			client: client,
		}
	}
}

impl<'a> TransactionQueueDetailsProvider for TransactionDetailsProvider<'a> {
	fn fetch_account(&self, address: &Address) -> AccountDetails {
		AccountDetails {
			nonce: self.client.latest_nonce(address),
			balance: self.client.latest_balance(address),
		}
	}

	fn estimate_gas_required(&self, tx: &SignedTransaction) -> U256 {
		tx.gas_required(&self.client.latest_schedule()).into()
	}

	fn is_service_transaction_acceptable(&self, _tx: &SignedTransaction) -> Result<bool, String> {
		Ok(false)
	}
}

/// Configurtion for private transaction provider
#[derive(Default, PartialEq, Debug, Clone)]
pub struct ProviderConfig {
	/// Accounts that can be used for validation
	pub validator_accounts: Vec<Address>,
	/// Account used for signing public transactions created from private transactions
	pub signer_account: Option<Address>,
	/// Passwords used to unlock accounts
	pub passwords: Vec<String>,
}

#[derive(Debug)]
/// Private transaction execution receipt.
pub struct Receipt {
	/// Private transaction hash.
	pub hash: H256,
	/// Created contract address if any.
	pub contract_address: Option<Address>,
	/// Execution status.
	pub status_code: u8,
}

/// Manager of private transactions
pub struct Provider {
	encryptor: Box<Encryptor>,
	validator_accounts: HashSet<Address>,
	signer_account: Option<Address>,
	passwords: Vec<String>,
	notify: RwLock<Vec<Weak<ChainNotify>>>,
	transactions_for_signing: Mutex<SigningStore>,
	transactions_for_verification: Mutex<VerificationStore>,
	client: Arc<Client>,
	accounts: Arc<AccountProvider>,
	channel: IoChannel<ClientIoMessage>,
}

#[derive(Debug)]
pub struct PrivateExecutionResult<T, V> where T: Tracer, V: VMTracer {
	code: Option<Bytes>,
	state: Bytes,
	contract_address: Option<Address>,
	result: Executed<T::Output, V::Output>,
}

impl Provider where {
	/// Create a new provider.
	pub fn new(
		client: Arc<Client>,
		accounts: Arc<AccountProvider>,
		encryptor: Box<Encryptor>,
		config: ProviderConfig,
		channel: IoChannel<ClientIoMessage>,
	) -> Result<Self, Error> {
		Ok(Provider {
			encryptor,
			validator_accounts: config.validator_accounts.into_iter().collect(),
			signer_account: config.signer_account,
			passwords: config.passwords,
			notify: RwLock::default(),
			transactions_for_signing: Mutex::default(),
			transactions_for_verification: Mutex::default(),
			client,
			accounts,
			channel,
		})
	}

	// TODO [ToDr] Don't use `ChainNotify` here!
	// Better to create a separate notification type for this.
	/// Adds an actor to be notified on certain events
	pub fn add_notify(&self, target: Arc<ChainNotify>) {
		self.notify.write().push(Arc::downgrade(&target));
	}

	fn notify<F>(&self, f: F) where F: Fn(&ChainNotify) {
		for np in self.notify.read().iter() {
			if let Some(n) = np.upgrade() {
				f(&*n);
			}
		}
	}

	/// 1. Create private transaction from the signed transaction
	/// 2. Executes private transaction
	/// 3. Save it with state returned on prev step to the queue for signing
	/// 4. Broadcast corresponding message to the chain
	pub fn create_private_transaction(&self, signed_transaction: SignedTransaction) -> Result<Receipt, Error> {
		trace!("Creating private transaction from regular transaction: {:?}", signed_transaction);
		if self.signer_account.is_none() {
			trace!("Signing account not set");
			bail!(ErrorKind::SignerAccountNotSet);
		}
		let tx_hash = signed_transaction.hash();
		match signed_transaction.action {
			Action::Create => {
				bail!(ErrorKind::BadTransactonType);
			}
			Action::Call(contract) => {
				let data = signed_transaction.rlp_bytes();
				let encrypted_transaction = self.encrypt(&contract, &Self::iv_from_transaction(&signed_transaction), &data)?;
				let private = PrivateTransaction {
					encrypted: encrypted_transaction,
					contract,
				};
				// TODO [ToDr] Using BlockId::Latest is bad here,
				// the block may change in the middle of execution
				// causing really weird stuff to happen.
				// We should retrieve hash and stick to that. IMHO
				// best would be to change the API and only allow H256 instead of BlockID
				// in private-tx to avoid such mistakes.
				let contract_nonce = self.get_contract_nonce(&contract, BlockId::Latest)?;
				let private_state = self.execute_private_transaction(BlockId::Latest, &signed_transaction)?;
				trace!("Private transaction created, encrypted transaction: {:?}, private state: {:?}", private, private_state);
				let contract_validators = self.get_validators(BlockId::Latest, &contract)?;
				trace!("Required validators: {:?}", contract_validators);
				let private_state_hash = self.calculate_state_hash(&private_state, contract_nonce);
				trace!("Hashed effective private state for sender: {:?}", private_state_hash);
				self.transactions_for_signing.lock().add_transaction(private.hash(), signed_transaction, contract_validators, private_state, contract_nonce)?;
				self.broadcast_private_transaction(private.rlp_bytes().into_vec());
				Ok(Receipt {
					hash: tx_hash,
					contract_address: None,
					status_code: 0,
				})
			}
		}
	}

	/// Calculate hash from united private state and contract nonce
	pub fn calculate_state_hash(&self, state: &Bytes, nonce: U256) -> H256 {
		let state_hash = keccak(state);
		let mut state_buf = [0u8; 64];
		state_buf[..32].clone_from_slice(&state_hash);
		state_buf[32..].clone_from_slice(&H256::from(nonce));
		keccak(&state_buf.as_ref())
	}

	/// Extract signed transaction from private transaction
	fn extract_original_transaction(&self, private: PrivateTransaction, contract: &Address) -> Result<UnverifiedTransaction, Error> {
		let encrypted_transaction = private.encrypted;
		let transaction_bytes = self.decrypt(contract, &encrypted_transaction)?;
		let original_transaction: UnverifiedTransaction = UntrustedRlp::new(&transaction_bytes).as_val()?;
		Ok(original_transaction)
	}

	/// Process received private transaction
	pub fn import_private_transaction(&self, rlp: &[u8]) -> Result<(), Error> {
		trace!("Private transaction received");
		let private_tx: PrivateTransaction = UntrustedRlp::new(rlp).as_val()?;
		let contract = private_tx.contract;
		let contract_validators = self.get_validators(BlockId::Latest, &contract)?;

		let validation_account = contract_validators
			.iter()
			.find(|address| self.validator_accounts.contains(address));

		match validation_account {
			None => {
				// Not for verification, broadcast further to peers
				self.broadcast_private_transaction(rlp.into());
				return Ok(());
			},
			Some(&validation_account) => {
				let hash = private_tx.hash();
				trace!("Private transaction taken for verification");
				let original_tx = self.extract_original_transaction(private_tx, &contract)?;
				trace!("Validating transaction: {:?}", original_tx);
				let details_provider = TransactionDetailsProvider::new(&*self.client as &MiningBlockChainClient);
				let insertion_time = self.client.chain_info().best_block_number;
				// Verify with the first account available
				trace!("The following account will be used for verification: {:?}", validation_account);
				self.transactions_for_verification.lock()
					.add_transaction(original_tx, contract, validation_account, hash, &details_provider, insertion_time)?;
				self.channel.send(ClientIoMessage::NewPrivateTransaction).map_err(|_| ErrorKind::ClientIsMalformed.into())
			}
		}
	}

	/// Private transaction for validation added into queue
	pub fn on_private_transaction_queued(&self) -> Result<(), Error> {
		self.process_queue()
	}

	/// Retrieve and verify the first available private transaction for every sender
	fn process_queue(&self) -> Result<(), Error> {
		let mut verification_queue = self.transactions_for_verification.lock();
		let ready_transactions = verification_queue.ready_transactions();
		let fetch_nonce = |a: &Address| self.client.latest_nonce(a);
		for transaction in ready_transactions {
			let transaction_hash = transaction.hash();
			match verification_queue.private_transaction_descriptor(&transaction_hash) {
				Ok(desc) => {
					if !self.validator_accounts.contains(&desc.validator_account) {
						trace!("Cannot find validator account in config");
						bail!(ErrorKind::ValidatorAccountNotSet);
					}
					let account = desc.validator_account;
					if let Action::Call(contract) = transaction.action {
						let contract_nonce = self.get_contract_nonce(&contract, BlockId::Latest)?;
						let private_state = self.execute_private_transaction(BlockId::Latest, &transaction)?;
						let private_state_hash = self.calculate_state_hash(&private_state, contract_nonce);
						trace!("Hashed effective private state for validator: {:?}", private_state_hash);
						let password = find_account_password(&self.passwords, &*self.accounts, &account);
						let signed_state = self.accounts.sign(account, password, private_state_hash)?;
						let signed_private_transaction = SignedPrivateTransaction::new(desc.private_hash, signed_state, None);
						trace!("Sending signature for private transaction: {:?}", signed_private_transaction);
						self.broadcast_signed_private_transaction(signed_private_transaction.rlp_bytes().into_vec());
					} else {
						trace!("Incorrect type of action for the transaction");
						bail!(ErrorKind::BadTransactonType);
					}
				},
				Err(e) => {
					trace!("Cannot retrieve descriptor for transaction with error {:?}", e);
					bail!(e);
				}
			}
			verification_queue.remove_private_transaction(&transaction_hash, &fetch_nonce);
		}
		Ok(())
	}

	/// Add signed private transaction into the store
	/// Creates corresponding public transaction if last required singature collected and sends it to the chain
	pub fn import_signed_private_transaction(&self, rlp: &[u8]) -> Result<(), Error> {
		let tx: SignedPrivateTransaction = UntrustedRlp::new(rlp).as_val()?;
		trace!("Signature for private transaction received: {:?}", tx);
		let private_hash = tx.private_transaction_hash();
		let desc = match self.transactions_for_signing.lock().get(&private_hash) {
			None => {
				// Not our transaction, broadcast further to peers
				self.broadcast_signed_private_transaction(rlp.into());
				return Ok(());
			},
			Some(desc) => desc,
		};

		let last = self.last_required_signature(&desc, tx.signature())?;

		if last {
			let mut signatures = desc.received_signatures.clone();
			signatures.push(tx.signature());
			let rsv: Vec<Signature> = signatures.into_iter().map(|sign| sign.into_electrum().into()).collect();
			//Create public transaction
			let public_tx = self.public_transaction(
				desc.state.clone(),
				&desc.original_transaction,
				&rsv,
				desc.original_transaction.nonce,
				desc.original_transaction.gas_price
			)?;
			trace!("Last required signature received, public transaction created: {:?}", public_tx);
			//Sign and add it to the queue
			let chain_id = desc.original_transaction.chain_id();
			let hash = public_tx.hash(chain_id);
			let signer_account = self.signer_account.ok_or_else(|| ErrorKind::SignerAccountNotSet)?;
			let password = find_account_password(&self.passwords, &*self.accounts, &signer_account);
			let signature = self.accounts.sign(signer_account, password, hash)?;
			let signed = SignedTransaction::new(public_tx.with_signature(signature, chain_id))?;
			match self.client.miner().import_own_transaction(&*self.client, signed.into()) {
				Ok(_) => trace!("Public transaction added to queue"),
				Err(err) => {
					trace!("Failed to add transaction to queue, error: {:?}", err);
					bail!(err);
				}
			}
			//Remove from store for signing
			match self.transactions_for_signing.lock().remove(&private_hash) {
				Ok(_) => {}
				Err(err) => {
					trace!("Failed to remove transaction from signing store, error: {:?}", err);
					bail!(err);
				}
			}
		} else {
			//Add signature to the store
			match self.transactions_for_signing.lock().add_signature(&private_hash, tx.signature()) {
				Ok(_) => trace!("Signature stored for private transaction"),
				Err(err) => {
					trace!("Failed to add signature to signing store, error: {:?}", err);
					bail!(err);
				}
			}
		}
		Ok(())
	}

	fn last_required_signature(&self, desc: &PrivateTransactionSigningDesc, sign: Signature) -> Result<bool, Error>  {
		if desc.received_signatures.contains(&sign) {
			return Ok(false);
		}
		let state_hash = self.calculate_state_hash(&desc.state, desc.contract_nonce);
		match recover(&sign, &state_hash) {
			Ok(public) => {
				let sender = public_to_address(&public);
				match desc.validators.contains(&sender) {
					true => {
						Ok(desc.received_signatures.len() + 1 == desc.validators.len())
					}
					false => {
						trace!("Sender's state doesn't correspond to validator's");
						bail!(ErrorKind::StateIncorrect);
					}
				}
			}
			Err(err) => {
				trace!("Sender's state doesn't correspond to validator's, error {:?}", err);
				bail!(err);
			}
		}
	}

	/// Broadcast the private transaction message to the chain
	fn broadcast_private_transaction(&self, message: Bytes) {
		self.notify(|notify| notify.broadcast(ChainMessageType::PrivateTransaction(message.clone())));
	}

	/// Broadcast signed private transaction message to the chain
	fn broadcast_signed_private_transaction(&self, message: Bytes) {
		self.notify(|notify| notify.broadcast(ChainMessageType::SignedPrivateTransaction(message.clone())));
	}

	fn iv_from_transaction(transaction: &SignedTransaction) -> H128 {
		let nonce = keccak(&transaction.nonce.rlp_bytes());
		let (iv, _) = nonce.split_at(INIT_VEC_LEN);
		H128::from_slice(iv)
	}

	fn iv_from_address(contract_address: &Address) -> H128 {
		let address = keccak(&contract_address.rlp_bytes());
		let (iv, _) = address.split_at(INIT_VEC_LEN);
		H128::from_slice(iv)
	}

	fn encrypt(&self, contract_address: &Address, initialisation_vector: &H128, data: &[u8]) -> Result<Bytes, Error> {
		trace!("Encrypt data using key(address): {:?}", contract_address);
		Ok(self.encryptor.encrypt(contract_address, &*self.accounts, initialisation_vector, data)?)
	}

	fn decrypt(&self, contract_address: &Address, data: &[u8]) -> Result<Bytes, Error> {
		trace!("Decrypt data using key(address): {:?}", contract_address);
		Ok(self.encryptor.decrypt(contract_address, &*self.accounts, data)?)
	}

	fn get_decrypted_state(&self, address: &Address, block: BlockId) -> Result<Bytes, Error> {
		let contract = private::PrivateContract::default();
		let state = contract.functions()
			.state()
			.call(&|data| self.client.call_contract(block, *address, data))
			.map_err(|e| ErrorKind::Call(format!("Contract call failed {:?}", e)))?;

		self.decrypt(address, &state)
	}

	fn get_decrypted_code(&self, address: &Address, block: BlockId) -> Result<Bytes, Error> {
		let contract = private::PrivateContract::default();
		let code = contract.functions()
			.code()
			.call(&|data| self.client.call_contract(block, *address, data))
			.map_err(|e| ErrorKind::Call(format!("Contract call failed {:?}", e)))?;

		self.decrypt(address, &code)
	}

	pub fn get_contract_nonce(&self, address: &Address, block: BlockId) -> Result<U256, Error> {
		let contract = private::PrivateContract::default();
		Ok(contract.functions()
			.nonce()
			.call(&|data| self.client.call_contract(block, *address, data))
			.map_err(|e| ErrorKind::Call(format!("Contract call failed {:?}", e)))?)
	}

	fn snapshot_to_storage(raw: Bytes) -> HashMap<H256, H256> {
		let items = raw.len() / 64;
		(0..items).map(|i| {
			let offset = i * 64;
			let key = H256::from_slice(&raw[offset..(offset + 32)]);
			let value = H256::from_slice(&raw[(offset + 32)..(offset + 64)]);
			(key, value)
		}).collect()
	}

	fn snapshot_from_storage(storage: &HashMap<H256, H256>) -> Bytes {
		let mut raw = Vec::with_capacity(storage.len() * 64);
		for (key, value) in storage {
			raw.extend_from_slice(key);
			raw.extend_from_slice(value);
		};
		raw
	}

	pub fn execute_private<T, V>(&self, transaction: &SignedTransaction, options: TransactOptions<T, V>, block: BlockId) -> Result<PrivateExecutionResult<T, V>, Error>
		where
			T: Tracer,
			V: VMTracer,
	{
		let mut env_info = self.client.env_info(block).ok_or(ErrorKind::StatePruned)?;
		env_info.gas_limit = transaction.gas;

		let mut state = self.client.state_at(block).ok_or(ErrorKind::StatePruned)?;
		// TODO: in case of BlockId::Latest these need to operate on the same state
		let contract_address = match transaction.action {
			Action::Call(ref contract_address) => {
				let contract_code = Arc::new(self.get_decrypted_code(contract_address, block)?);
				let contract_state = self.get_decrypted_state(contract_address, block)?;
				trace!("Patching contract at {:?}, code: {:?}, state: {:?}", contract_address, contract_code, contract_state);
				state.patch_account(contract_address, contract_code, Self::snapshot_to_storage(contract_state))?;
				Some(*contract_address)
			},
			Action::Create => None,
		};

		let engine = self.client.engine();
		let contract_address = contract_address.or({
			let sender = transaction.sender();
			let nonce = state.nonce(&sender)?;
			let (new_address, _) = ethcore_contract_address(engine.create_address_scheme(env_info.number), &sender, &nonce, &transaction.data);
			Some(new_address)
		});
		let result = Executive::new(&mut state, &env_info, engine.machine()).transact_virtual(transaction, options)?;
		let (encrypted_code, encrypted_storage) = match contract_address {
			None => bail!(ErrorKind::ContractDoesNotExist),
			Some(address) => {
				let (code, storage) = state.into_account(&address)?;
				let enc_code = match code {
					Some(c) => Some(self.encrypt(&address, &Self::iv_from_address(&address), &c)?),
					None => None,
				};
				(enc_code, self.encrypt(&address, &Self::iv_from_transaction(transaction), &Self::snapshot_from_storage(&storage))?)
			},
		};
		trace!("Private contract executed. code: {:?}, state: {:?}, result: {:?}", encrypted_code, encrypted_storage, result.output);
		Ok(PrivateExecutionResult {
			code: encrypted_code,
			state: encrypted_storage,
			contract_address,
			result,
		})
	}

	fn generate_constructor(validators: &[Address], code: Bytes, storage: Bytes) -> Bytes {
		let constructor_code = DEFAULT_STUB_CONTRACT.from_hex().expect("Default contract code is valid");
		let private = private::PrivateContract::default();
		private.constructor(constructor_code, validators.iter().map(|a| *a).collect::<Vec<Address>>(), code, storage)
	}

	fn generate_set_state_call(signatures: &[Signature], storage: Bytes) -> Bytes {
		let private = private::PrivateContract::default();
		private.functions().set_state().input(
			storage,
			signatures.iter().map(|s| {
				let mut v: [u8; 32] = [0; 32];
				v[31] = s.v();
				v
			}).collect::<Vec<[u8; 32]>>(),
			signatures.iter().map(|s| s.r()).collect::<Vec<&[u8]>>(),
			signatures.iter().map(|s| s.s()).collect::<Vec<&[u8]>>()
		)
	}

	/// Returns the key from the key server associated with the contract
	pub fn contract_key_id(&self, contract_address: &Address) -> Result<H256, Error> {
		//current solution uses contract address extended with 0 as id
		let contract_address_extended: H256 = contract_address.into();

		Ok(H256::from_slice(&contract_address_extended))
	}

	/// Create encrypted public contract deployment transaction.
	pub fn public_creation_transaction(&self, block: BlockId, source: &SignedTransaction, validators: &[Address], gas_price: U256) -> Result<(Transaction, Option<Address>), Error> {
		if let Action::Call(_) = source.action {
			bail!(ErrorKind::BadTransactonType);
		}
		let sender = source.sender();
		let state = self.client.state_at(block).ok_or(ErrorKind::StatePruned)?;
		let nonce = state.nonce(&sender)?;
		let executed = self.execute_private(source, TransactOptions::with_no_tracing(), block)?;
		let gas: u64 = 650000 +
			validators.len() as u64 * 30000 +
			executed.code.as_ref().map_or(0, |c| c.len() as u64) * 8000 +
			executed.state.len() as u64 * 8000;
		Ok((Transaction {
			nonce: nonce,
			action: Action::Create,
			gas: gas.into(),
			gas_price: gas_price,
			value: source.value,
			data: Self::generate_constructor(validators, executed.code.unwrap_or_default(), executed.state)
		},
		executed.contract_address))
	}

	/// Create encrypted public contract deployment transaction. Returns updated encrypted state.
	pub fn execute_private_transaction(&self, block: BlockId, source: &SignedTransaction) -> Result<Bytes, Error> {
		if let Action::Create = source.action {
			bail!(ErrorKind::BadTransactonType);
		}
		let result = self.execute_private(source, TransactOptions::with_no_tracing(), block)?;
		Ok(result.state)
	}

	/// Create encrypted public transaction from private transaction.
	pub fn public_transaction(&self, state: Bytes, source: &SignedTransaction, signatures: &[Signature], nonce: U256, gas_price: U256) -> Result<Transaction, Error> {
		let gas: u64 = 650000 + state.len() as u64 * 8000 + signatures.len() as u64 * 50000;
		Ok(Transaction {
			nonce: nonce,
			action: source.action.clone(),
			gas: gas.into(),
			gas_price: gas_price,
			value: 0.into(),
			data: Self::generate_set_state_call(signatures, state)
		})
	}

	/// Call into private contract.
	pub fn private_call(&self, block: BlockId, transaction: &SignedTransaction) -> Result<Executed, Error> {
		let result = self.execute_private(transaction, TransactOptions::with_no_tracing(), block)?;
		Ok(result.result)
	}

	/// Returns private validators for a contract.
	pub fn get_validators(&self, block: BlockId, address: &Address) -> Result<Vec<Address>, Error> {
		let contract = private::PrivateContract::default();
		Ok(contract.functions()
			.get_validators()
			.call(&|data| self.client.call_contract(block, *address, data))
			.map_err(|e| ErrorKind::Call(format!("Contract call failed {:?}", e)))?)
	}
}

/// Try to unlock account using stored password, return found password if any
fn find_account_password(passwords: &Vec<String>, account_provider: &AccountProvider, account: &Address) -> Option<String> {
	for password in passwords {
		if let Ok(true) = account_provider.test_password(account, password) {
			return Some(password.clone());
		}
	}
	None
}

impl ChainNotify for Provider {
	fn new_blocks(&self, imported: Vec<H256>, _invalid: Vec<H256>, _enacted: Vec<H256>, _retracted: Vec<H256>, _sealed: Vec<H256>, _proposed: Vec<Bytes>, _duration: u64) {
		if !imported.is_empty() {
			trace!("New blocks imported, try to prune the queue");
			if let Err(err) = self.process_queue() {
				trace!("Cannot prune private transactions queue. error: {:?}", err);
			}
		}
	}
}
