// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;

use accounts::AccountProvider;
use bytes::Bytes;
use crypto::DEFAULT_MAC;
use ethereum_types::{H256, U256, Address};
use crypto::publickey::Signature;
use types::transaction::{Transaction, Action, SignedTransaction};

use jsonrpc_core::Result;
use v1::helpers::{errors, FilledTransactionRequest};

use super::{eth_data_hash, WithToken, SignWith, SignMessage};

/// Account-aware signer
pub struct Signer {
	accounts: Arc<AccountProvider>,
}

impl Signer {
	/// Create new instance of signer
	pub fn new(accounts: Arc<AccountProvider>) -> Self {
		Signer { accounts }
	}
}

impl super::Accounts for Signer {
	fn sign_transaction(&self, filled: FilledTransactionRequest, chain_id: Option<u64>, nonce: U256, password: SignWith) -> Result<WithToken<SignedTransaction>> {
		let t = Transaction {
			nonce: nonce,
			action: filled.to.map_or(Action::Create, Action::Call),
			gas: filled.gas,
			gas_price: filled.gas_price,
			value: filled.value,
			data: filled.data,
		};

		let hash = t.hash(chain_id);
		let signature = signature(&*self.accounts, filled.from, hash, password)?;

		Ok(signature.map(|sig| {
			SignedTransaction::new(t.with_signature(sig, chain_id))
				.expect("Transaction was signed by AccountsProvider; it never produces invalid signatures; qed")
		}))
	}

	fn sign_message(&self, address: Address, password: SignWith, hash: SignMessage) -> Result<WithToken<Signature>> {
		match hash {
			SignMessage::Data(data) => {
				let hash = eth_data_hash(data);
				signature(&self.accounts, address, hash, password)
			},
			SignMessage::Hash(hash) => {
				signature(&self.accounts, address, hash, password)
			}
		}
	}

	fn decrypt(&self, address: Address, password: SignWith, data: Bytes) -> Result<WithToken<Bytes>> {
		match password.clone() {
			SignWith::Nothing => self.accounts.decrypt(address, None, &DEFAULT_MAC, &data).map(WithToken::No),
			SignWith::Password(pass) => self.accounts.decrypt(address, Some(pass), &DEFAULT_MAC, &data).map(WithToken::No),
			SignWith::Token(token) => self.accounts.decrypt_with_token(address, token, &DEFAULT_MAC, &data).map(Into::into),
		}.map_err(|e| match password {
			SignWith::Nothing => errors::signing(e),
			_ => errors::password(e),
		})
	}

	fn supports_prospective_signing(&self, address: &Address, password: &SignWith) -> bool {
		// If the account is permanently unlocked we can try to sign
		// using prospective nonce. This should speed up sending
		// multiple subsequent transactions in multi-threaded RPC environment.
		let is_unlocked_permanently = self.accounts.is_unlocked_permanently(address);
		let has_password = password.is_password();

		is_unlocked_permanently || has_password
	}

	fn default_account(&self) -> Address {
		self.accounts.default_account().ok().unwrap_or_default()
	}

	fn is_unlocked(&self, address: &Address) -> bool {
		self.accounts.is_unlocked(address)
	}
}

fn signature(accounts: &AccountProvider, address: Address, hash: H256, password: SignWith) -> Result<WithToken<Signature>> {
	match password.clone() {
		SignWith::Nothing => accounts.sign(address, None, hash).map(WithToken::No),
		SignWith::Password(pass) => accounts.sign(address, Some(pass), hash).map(WithToken::No),
		SignWith::Token(token) => accounts.sign_with_token(address, token, hash).map(Into::into),
	}.map_err(|e| match password {
		SignWith::Nothing => errors::signing(e),
		_ => errors::password(e),
	})
}

