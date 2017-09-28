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
use std::sync::{Arc, Weak};
use client::{ChainNotify, ChainMessageType};
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
//TODO: to remove this use
use rustc_hex::FromHex;

/// Initialization vector length.
const INIT_VEC_LEN: usize = 16;

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
}