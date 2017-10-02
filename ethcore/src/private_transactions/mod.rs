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
/// Export the private_transactions module.
pub mod private_transactions;

pub use self::private_transactions::*;

use std::iter::repeat;
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use bigint::hash::H256;
use bigint::prelude::U256;
use util::Address;
use executive::{Executive, TransactOptions};
use executed::{Executed};
use transaction::{SignedTransaction, Transaction, Action};
use client::{Client, ChainNotify, ChainMessageType, BlockId};
use client::BlockChainClient;
use error::{PrivateTransactionError};
use transaction::UnverifiedTransaction;
use error::Error as EthcoreError;
use ethkey::{Signature, Error as EthkeyError};
use rlp::*;
use bigint::prelude::U256;
use bigint::hash::H256;
use hash::keccak;
use rand::{Rng, OsRng};
use parking_lot::{Mutex, RwLock};
use bytes::Bytes;
use util::Address;
use ethcrypto::aes::{encrypt, decrypt};
use native_contracts::Private as Contract;
use futures::{self, Future};
use trace;
use ethabi;
//TODO: to remove this use
use rustc_hex::FromHex;

/// Initialization vector length.
const INIT_VEC_LEN: usize = 16;

const DEFAULT_STUB_CONTRACT: &'static str = "6060604052341561000f57600080fd5b6040516109ce3803806109ce83398101604052808051820191906020018051820191906020018051820191905050826000908051906020019061005392919061008a565b50816002908051906020019061006a929190610114565b508060019080519060200190610081929190610114565b505050506101fc565b828054828255906000526020600020908101928215610103579160200282015b828111156101025782518260006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550916020019190600101906100aa565b5b5090506101109190610194565b5090565b828054600181600116156101000203166002900490600052602060002090601f016020900481019282601f1061015557805160ff1916838001178555610183565b82800160010185558215610183579182015b82811115610182578251825591602001919060010190610167565b5b50905061019091906101d7565b5090565b6101d491905b808211156101d057600081816101000a81549073ffffffffffffffffffffffffffffffffffffffff02191690555060010161019a565b5090565b90565b6101f991905b808211156101f55760008160009055506001016101dd565b5090565b90565b6107c38061020b6000396000f30060606040526000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff16806317ac53a21461005e5780631865c57d1461017b578063b7ab4db514610209578063ea8796341461027357600080fd5b341561006957600080fd5b610179600480803590602001908201803590602001908080601f01602080910402602001604051908101604052809392919081815260200183838082843782019150505050505091908035906020019082018035906020019080806020026020016040519081016040528093929190818152602001838360200280828437820191505050505050919080359060200190820180359060200190808060200260200160405190810160405280939291908181526020018383602002808284378201915050505050509190803590602001908201803590602001908080602002602001604051908101604052809392919081815260200183836020028082843782019150505050505091905050610301565b005b341561018657600080fd5b61018e6104e6565b6040518080602001828103825283818151815260200191508051906020019080838360005b838110156101ce5780820151818401526020810190506101b3565b50505050905090810190601f1680156101fb5780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b341561021457600080fd5b61021c61058e565b6040518080602001828103825283818151815260200191508051906020019060200280838360005b8381101561025f578082015181840152602081019050610244565b505050509050019250505060405180910390f35b341561027e57600080fd5b610286610622565b6040518080602001828103825283818151815260200191508051906020019080838360005b838110156102c65780820151818401526020810190506102ab565b50505050905090810190601f1680156102f35780820380516001836020036101000a031916815260200191505b509250505060405180910390f35b600080856040518082805190602001908083835b60208310151561033a5780518252602082019150602081019050602083039250610315565b6001836020036101000a03801982511681845116808217855250505050505090500191505060405180910390209150600090505b6000805490508110156104c75760008181548110151561038a57fe5b906000526020600020900160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1660018387848151811015156103dd57fe5b9060200190602002015187858151811015156103f557fe5b90602001906020020151878681518110151561040d57fe5b90602001906020020151604051600081526020016040526000604051602001526040518085600019166000191681526020018460ff1660ff16815260200183600019166000191681526020018260001916600019168152602001945050505050602060405160208103908084039060008661646e5a03f1151561048f57600080fd5b50506020604051035173ffffffffffffffffffffffffffffffffffffffff161415156104ba57600080fd5b808060010191505061036e565b85600190805190602001906104dd9291906106ca565b50505050505050565b6104ee61074a565b60018054600181600116156101000203166002900480601f0160208091040260200160405190810160405280929190818152602001828054600181600116156101000203166002900480156105845780601f1061055957610100808354040283529160200191610584565b820191906000526020600020905b81548152906001019060200180831161056757829003601f168201915b5050505050905090565b61059661075e565b600080548060200260200160405190810160405280929190818152602001828054801561061857602002820191906000526020600020905b8160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190600101908083116105ce575b5050505050905090565b61062a61074a565b60028054600181600116156101000203166002900480601f0160208091040260200160405190810160405280929190818152602001828054600181600116156101000203166002900480156106c05780601f10610695576101008083540402835291602001916106c0565b820191906000526020600020905b8154815290600101906020018083116106a357829003601f168201915b5050505050905090565b828054600181600116156101000203166002900490600052602060002090601f016020900481019282601f1061070b57805160ff1916838001178555610739565b82800160010185558215610739579182015b8281111561073857825182559160200191906001019061071d565b5b5090506107469190610772565b5090565b602060405190810160405280600081525090565b602060405190810160405280600081525090565b61079491905b80821115610790576000816000905550600101610778565b5090565b905600a165627a7a7230582012a0ab4be8ba61a3fc7601b05ab9c31c619ceccc4ff930f31cd28e140bcb4d340029";

/// Private transaction message call to the contract
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct PrivateTransaction {
	/// Encrypted data
	encrypted: Bytes,
	/// Address of the contract
	contract: Address,
}

impl Decodable for PrivateTransaction {
	fn decode(d: &UntrustedRlp) -> Result<Self, DecoderError> {
		if d.item_count()? != 2 {
			return Err(DecoderError::RlpIncorrectListLen);
		}
		Ok(PrivateTransaction {
			encrypted: d.val_at(0)?,
			contract: d.val_at(1)?,
		})
	}
}

impl Encodable for PrivateTransaction {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(2);
		s.append(&self.encrypted);
		s.append(&self.contract);
	}
}

fn initialization_vector() -> [u8; INIT_VEC_LEN] {
	let mut result = [0u8; INIT_VEC_LEN];
	let mut rng = OsRng::new().unwrap();
	rng.fill_bytes(&mut result);
	result
}

impl PrivateTransaction {
	/// Create private transaction from the signed transaction
	pub fn create_from_signed(transaction: UnverifiedTransaction, contract: Address) -> Result<Self, EthcoreError> {
		//TODO: retrieve key from secret store using contract
		let init_key: Bytes = "cac6c205eb06c8308d65156ff6c862c62b000b8ead121a4455a8ddeff7248128d895692136f240d5d1614dc7cc4147b1bd584bd617e30560bb872064d09ea325".from_hex().unwrap();
		let key: Bytes = init_key[..INIT_VEC_LEN].into();

		let transaction_document = transaction.rlp_bytes();
		let iv = initialization_vector();
		let mut encrypted_transaction = Vec::with_capacity(transaction_document.len() + iv.len());
		encrypted_transaction.extend(repeat(0).take(transaction_document.len()));
		encrypt(&key, &iv, &transaction_document, &mut encrypted_transaction);
		encrypted_transaction.extend_from_slice(&iv);

		let private = PrivateTransaction {
			encrypted: encrypted_transaction,
			contract: contract,
		};
		Ok(private)
	}

	/// Extract signed transaction from private transaction
	pub fn extract_signed_transaction(&self, _contract: Address) -> Result<UnverifiedTransaction, EthcoreError> {
		let mut encrypted_transaction = self.encrypted.clone();
		let encrypted_transaction_len = encrypted_transaction.len();
		if encrypted_transaction_len < INIT_VEC_LEN {
			return Err(EthkeyError::InvalidMessage.into());
		}

		//TODO: retrieve key from secret store using contract
		let init_key: Bytes = "cac6c205eb06c8308d65156ff6c862c62b000b8ead121a4455a8ddeff7248128d895692136f240d5d1614dc7cc4147b1bd584bd617e30560bb872064d09ea325".from_hex().unwrap();
		let key: Bytes = init_key[..INIT_VEC_LEN].into();

		// use symmetric decryption to decrypt transaction
		let iv = encrypted_transaction.split_off(encrypted_transaction_len - INIT_VEC_LEN);
		let mut document = Vec::with_capacity(encrypted_transaction_len - INIT_VEC_LEN);
		document.extend(repeat(0).take(encrypted_transaction_len - INIT_VEC_LEN));
		decrypt(&key, &iv, &encrypted_transaction, &mut document);
		let signed_transaction: UnverifiedTransaction = UntrustedRlp::new(&document).as_val()?;
		Ok(signed_transaction)
	}

	/// Compute hash on private transaction
	pub fn hash(&self) -> H256 {
		keccak(&*self.rlp_bytes())
	}
}

/// Message about private transaction's signing
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SignedPrivateTransaction {
	/// Hash of the corresponding private transaction
	private_transaction_hash: H256,
	/// Signature of the validator
	/// The V field of the signature
	v: u64,
	/// The R field of the signature
	r: U256,
	/// The S field of the signature
	s: U256,
}

impl Decodable for SignedPrivateTransaction {
	fn decode(d: &UntrustedRlp) -> Result<Self, DecoderError> {
		if d.item_count()? != 4 {
			return Err(DecoderError::RlpIncorrectListLen);
		}
		Ok(SignedPrivateTransaction {
			private_transaction_hash: d.val_at(0)?,
			v: d.val_at(1)?,
			r: d.val_at(1)?,
			s: d.val_at(1)?,
		})
	}
}

impl Encodable for SignedPrivateTransaction {
	fn rlp_append(&self, s: &mut RlpStream) {
		s.begin_list(4);
		s.append(&self.private_transaction_hash);
		s.append(&self.v);
		s.append(&self.r);
		s.append(&self.s);
	}
}

impl SignedPrivateTransaction {
	/// Construct a signed private transaction message
	pub fn new(private_transaction: PrivateTransaction, sig: Signature, chain_id: Option<u64>) -> Self {
		SignedPrivateTransaction {
			private_transaction_hash: private_transaction.hash(),
			r: sig.r().into(),
			s: sig.s().into(),
			v: sig.v() as u64 + if let Some(n) = chain_id { 35 + n * 2 } else { 27 },
		}
	}

	/// 0 if `v` would have been 27 under "Electrum" notation, 1 if 28 or 4 if invalid.
	pub fn standard_v(&self) -> u8 { match self.v { v if v == 27 || v == 28 || v > 36 => ((v - 1) % 2) as u8, _ => 4 } }

	/// Construct a signature object from the sig.
	pub fn signature(&self) -> Signature {
		Signature::from_rsv(&self.r.into(), &self.s.into(), self.standard_v())
	}

	/// Get the hash of of the original transaction.
	pub fn private_transaction_hash(&self) -> H256 {
		self.private_transaction_hash
	}
}

/// Manager of private transactions
pub struct Provider {
	notify: RwLock<Vec<Weak<ChainNotify>>>,
	private_transactions: Mutex<PrivateTransactions>,
}

#[derive(Debug)]
struct PrivateExecutionResult {
	code: Option<Bytes>,
	state: Bytes,
	result: Executed,
}

impl Provider {
	/// Create a new provider.
	pub fn new() -> Self {
		Provider {
			notify: RwLock::new(Vec::new()),
			private_transactions: Mutex::new(PrivateTransactions::new()),
		}
	}

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

	/// Add private transaction into the store
	pub fn import_private_transaction(&self, rlp: &[u8], peer_id: usize) -> Result<(), EthcoreError> {
		let tx: PrivateTransaction = UntrustedRlp::new(rlp).as_val()?;
		self.private_transactions.lock().import_transaction(tx, peer_id)
	}

	/// Add signed private transaction into the store
	pub fn import_signed_private_transaction(&self, rlp: &[u8], peer_id: usize) -> Result<(), EthcoreError> {
		let tx: SignedPrivateTransaction = UntrustedRlp::new(rlp).as_val()?;
		self.private_transactions.lock().import_signed_transaction(tx, peer_id)
	}

	/// Broadcast the private transaction message to chain
	pub fn broadcast_private_transaction(&self, message: Bytes) {
		self.notify(|notify| notify.broadcast(ChainMessageType::PrivateTransaction, message.clone()));
	}

	/// Broadcast signed private transaction message to chain
	pub fn broadcast_signed_private_transaction(&self, message: Bytes) {
		self.notify(|notify| notify.broadcast(ChainMessageType::SignedPrivateTransaction, message.clone()));
	}

	/// Returns the list of private transactions
	pub fn private_transactions(&self) -> Vec<PrivateTransaction> {
		self.private_transactions.lock().transactions_list()
	}

	/// Returns the list of signed private transactions
	pub fn signed_private_transactions(&self) -> Vec<SignedPrivateTransaction> {
		self.private_transactions.lock().signed_transactions_list()
	}

	fn encrypt(&self, _contract_address: &Address, data: &[u8]) -> Result<Bytes, EthcoreError> {
		Ok(data.to_vec())
	}

	fn decrypt(&self, _contract_address: &Address, data: &[u8]) -> Result<Bytes, EthcoreError> {
		Ok(data.to_vec())
	}

	fn get_decrypted_state(&self, address: &Address, block: BlockId, client: &Client) -> Result<Bytes, EthcoreError> {
		let contract = Contract::new(*address);
		let state = contract.get_state(|addr, data| futures::done(client.call_contract(block, addr, data))).wait()
			.map_err(|e| PrivateTransactionError::Call(e))?;
		self.decrypt(address, &state)
	}

	fn get_decrypted_code(&self, address: &Address, block: BlockId, client: &Client) -> Result<Bytes, EthcoreError> {
		let contract = Contract::new(*address);
		let code = contract.get_code(|addr, data| futures::done(client.call_contract(block, addr, data))).wait()
			.map_err(|e| PrivateTransactionError::Call(e))?;
		self.decrypt(address, &code)
	}

	fn to_storage(raw: Bytes) -> HashMap<H256, H256> {
		let items = raw.len() / 64;
		(0..items).map(|i| {
			let offset = i * 64;
			let key = H256::from_slice(&raw[offset..(offset + 32)]);
			let value = H256::from_slice(&raw[(offset + 32)..(offset + 64)]);
			(key, value)
		}).collect()
	}

	fn from_storage(storage: &HashMap<H256, H256>) -> Bytes {
		let mut raw = Vec::with_capacity(storage.len() * 64);
		for (key, value) in storage {
			raw.extend_from_slice(key);
			raw.extend_from_slice(value);
		};
		raw
	}

	fn execute_private<T, V>(&self, transaction: &SignedTransaction, options: TransactOptions<T, V>, block: BlockId, client: &Client) -> Result<PrivateExecutionResult, EthcoreError>
		where
			T: trace::Tracer,
			V: trace::VMTracer,
	{
		let mut env_info = client.env_info(block).ok_or(PrivateTransactionError::StatePruned)?;
		env_info.gas_limit = U256::max_value();

		let mut state = client.state_at(block).ok_or(PrivateTransactionError::StatePruned)?;
		// TODO: in case of BlockId::Latest these need to operate on the same state
		let contract_address = match &transaction.action {
			&Action::Call(ref contract_address) => {
				let contract_code = Arc::new(self.get_decrypted_code(contract_address, block, client)?);
				let contract_state = self.get_decrypted_state(contract_address, block, client)?;
				trace!("Patching contract at {:?}, code: {:?}, state: {:?}", contract_address, contract_code, contract_state);
				state.patch_account(contract_address, contract_code.clone(), Self::to_storage(contract_state))?;
				Some(contract_address.clone())
			},
			&Action::Create => None,
		};

		let engine = client.engine();
		let result = Executive::new(&mut state, &env_info, engine).transact_virtual(transaction, options)?;
		let contract_address = contract_address.or(result.contracts_created.first().cloned());
		let (encrypted_code, encrypted_storage) = match contract_address {
			Some(address) => {
				let (code, storage) = state.into_account(&address)?;
				let enc_code = match code {
					Some(c) => Some(self.encrypt(&address, &c)?),
					None => None,
				};
				(enc_code, self.encrypt(&address, &Self::from_storage(&storage))?)
			},
			None => return Err(PrivateTransactionError::ContractDoesNotExist.into())
		};
		trace!("Private contract executed. code: {:?}, state: {:?}, result: {:?}", encrypted_code, encrypted_storage, result);
		Ok(PrivateExecutionResult {
			code: encrypted_code,
			state: encrypted_storage,
			result: result,
		})
	}

	fn generate_constructor(validators: &[Address], code: Bytes, storage: Bytes) -> Bytes {
		let constructor_code = DEFAULT_STUB_CONTRACT.from_hex().expect("Default contract code is valid");
		let constructor = ethabi::Constructor { inputs: vec![
			ethabi::Param { name: "v".into(), kind: ethabi::ParamType::Array(Box::new(ethabi::ParamType::Address)) },
			ethabi::Param { name: "c".into(), kind: ethabi::ParamType::Bytes },
			ethabi::Param { name: "s".into(), kind: ethabi::ParamType::Bytes },
		]};

		let tokens = [
			ethabi::Token::Array(validators.iter().map(|a| ethabi::Token::Address(a.clone().0)).collect()),
			ethabi::Token::Bytes(code),
			ethabi::Token::Bytes(storage),
		];

		constructor.encode_input(constructor_code, &tokens).expect("Input is always valid")
	}

	fn generate_set_state_call(signatures: &[Signature], storage: Bytes) -> Bytes {
		let function = ethabi::Function { name: "setState".into(), constant:false, outputs: vec![], inputs: vec![
			ethabi::Param { name: "ns".into(), kind: ethabi::ParamType::Bytes },
			ethabi::Param { name: "v".into(), kind: ethabi::ParamType::Array(Box::new(ethabi::ParamType::Uint(8))) },
			ethabi::Param { name: "r".into(), kind: ethabi::ParamType::Array(Box::new(ethabi::ParamType::FixedBytes(32))) },
			ethabi::Param { name: "s".into(), kind: ethabi::ParamType::Array(Box::new(ethabi::ParamType::FixedBytes(32))) },
		]};

		let tokens = [
			ethabi::Token::Bytes(storage),
			ethabi::Token::Array(signatures.iter().map(|s| {
				let mut v: [u8; 32] = [0; 32];
				v[31] = s.v();
				ethabi::Token::Uint(v)
			}).collect()),
			ethabi::Token::Array(signatures.iter().map(|s| ethabi::Token::FixedBytes(s.r().to_vec())).collect()),
			ethabi::Token::Array(signatures.iter().map(|s| ethabi::Token::FixedBytes(s.s().to_vec())).collect()),
		];

		function.encode_input(&tokens).expect("Input is always valid")
	}

	/// Create encrypted public contract deployment transaction.
	pub fn public_creation_transaction(&self, block: BlockId, source: &SignedTransaction, validators: &[Address], nonce: U256, gas_price: U256, client: &Client) -> Result<Transaction, EthcoreError> {
		if let &Action::Call(_) = &source.action {
			return Err(PrivateTransactionError::BadTransactonType.into());
		}
		let executed = self.execute_private(source, TransactOptions::with_no_tracing(), block, client)?;
		let gas: u64 = 650000 +
			validators.len() as u64 * 30000 +
			executed.code.as_ref().map_or(0, |c| c.len() as u64) * 10000 +
			executed.state.len() as u64 * 10000;
		Ok(Transaction {
			nonce: nonce,
			action: Action::Create,
			gas: gas.into(),
			gas_price: gas_price,
			value: source.value,
			data: Self::generate_constructor(validators, executed.code.unwrap_or_default(), executed.state)
		})
	}

	/// Create encrypted public contract deployment transaction. Returns updated encrypted state.
	pub fn execute_private_transaction(&self, block: BlockId, source: &SignedTransaction, client: &Client) -> Result<Bytes, EthcoreError> {
		if let &Action::Create = &source.action {
			return Err(PrivateTransactionError::BadTransactonType.into());
		}
		let result = self.execute_private(source, TransactOptions::with_no_tracing(), block, client)?;
		Ok(result.state)
	}

	/// Create encrypted public contract deployment transaction.
	pub fn public_call_transaction(&self, state: Bytes, source: &SignedTransaction, signatures: &[Signature], nonce: U256, gas_price: U256) -> Result<Transaction, EthcoreError> {
		let gas: u64 = 650000 + state.len() as u64 * 20000 + signatures.len() as u64 * 50000;
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
	pub fn private_call(&self, block: BlockId, transaction: &SignedTransaction, client: &Client) -> Result<Executed, EthcoreError> {
		let result = self.execute_private(transaction, TransactOptions::with_no_tracing(), block, client)?;
		Ok(result.result)
	}

	/// Returns private valaidators for a contract.
	pub fn get_validators(&self, block: BlockId, contract: &Address, client: &Client) -> Result<Vec<Address>, EthcoreError> {
		let contract = Contract::new(*contract);
		Ok(contract.get_validators(|addr, data| futures::done(client.call_contract(block, addr, data))).wait()
			.map_err(|e| PrivateTransactionError::Call(e))?)
	}
}


#[cfg(test)]
mod test {
	use rustc_hex::FromHex;
	use client::{BlockChainClient, BlockId};
	use hash::keccak;
	use ethkey::{Secret, KeyPair, Signature};
	use transaction::{Transaction, Action};
	use executive::{contract_address};
	use evm::CreateContractAddress;
	use super::Provider;
	use tests::helpers::{generate_dummy_client, push_block_with_transactions};

	/// Contract code:
	#[test]
	fn private_contract() {
		::ethcore_logger::init_log();
		let client = generate_dummy_client(0);
		let chain_id = client.signing_chain_id();
		let key1 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000011")).unwrap();
		let _key2 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000012")).unwrap();
		let key3 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000013")).unwrap();
		let key4 = KeyPair::from_secret(Secret::from("0000000000000000000000000000000000000000000000000000000000000014")).unwrap();

		let pm = Provider::new();
		let (address, _) = contract_address(CreateContractAddress::FromSenderAndNonce, &key1.address(), &0.into(), &[]);

		trace!("Creating private contrct");
		let private_contract_test = "6060604052341561000f57600080fd5b60d88061001d6000396000f30060606040526000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff1680630c55699c146046578063bc64b76d14607457600080fd5b3415605057600080fd5b60566098565b60405180826000191660001916815260200191505060405180910390f35b3415607e57600080fd5b6096600480803560001916906020019091905050609e565b005b60005481565b8060008160001916905550505600a165627a7a723058206acbdf4b15ca4c2d43e1b1879b830451a34f1e9d02ff1f2f394d8d857e79d2080029".from_hex().unwrap();
		let mut private_create_tx = Transaction::default();
		private_create_tx.action = Action::Create;
		private_create_tx.data = private_contract_test;
		private_create_tx.gas = 200000.into();
		let private_create_tx_signed = private_create_tx.sign(&key1.secret(), None);
		let validators = vec![key3.address(), key4.address()];
		let public_tx = pm.public_creation_transaction(BlockId::Pending, &private_create_tx_signed, &validators, 0.into(), 0.into(), &client).unwrap();
		let public_tx = public_tx.sign(&key1.secret(), chain_id);
		push_block_with_transactions(&client, &[public_tx]);

		trace!("Modifying private state");
		let mut private_tx = Transaction::default();
		private_tx.action = Action::Call(address.clone());
		private_tx.data = "bc64b76d2a00000000000000000000000000000000000000000000000000000000000000".from_hex().unwrap();
		private_tx.gas = 120000.into();
		private_tx.nonce = 1.into();
		let private_tx = private_tx.sign(&key1.secret(), None);
		let private_state = pm.execute_private_transaction(BlockId::Latest, &private_tx, &client).unwrap();
		let private_state_hash = keccak(&private_state);
		let signatures: Vec<_> = [&key3, &key4].iter().map(|k|
			Signature::from(::ethkey::sign(&k.secret(), &private_state_hash).unwrap().into_electrum())).collect();
		let public_tx = pm.public_call_transaction(private_state, &private_tx, &signatures, 1.into(), 0.into()).unwrap();
		let public_tx = public_tx.sign(&key1.secret(), chain_id);
		push_block_with_transactions(&client, &[public_tx]);

		trace!("Querying private state");
		let mut private_tx = Transaction::default();
		private_tx.action = Action::Call(address.clone());
		private_tx.data = "0c55699c".from_hex().unwrap();
		private_tx.gas = 50000.into();
		private_tx.nonce = 2.into();
		let private_tx = private_tx.sign(&key1.secret(), chain_id);
		let result = pm.private_call(BlockId::Latest, &private_tx, &client).unwrap();
		assert_eq!(&result.output[..], &("2a00000000000000000000000000000000000000000000000000000000000000".from_hex().unwrap()[..]));
		assert_eq!(pm.get_validators(BlockId::Latest, &address, &client).unwrap(), validators);
	}
}

