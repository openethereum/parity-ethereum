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

//! Personal rpc interface.
use jsonrpc_core::Error;

use futures::BoxFuture;

use v1::types::{U128, H160, H256, TransactionRequest};

build_rpc_trait! {
	/// Personal rpc interface. Safe (read-only) functions.
	pub trait Personal {
		type Metadata;

		/// Lists all stored accounts
		#[rpc(name = "personal_listAccounts")]
		fn accounts(&self) -> Result<Vec<H160>, Error>;

		/// Creates new account (it becomes new current unlocked account)
		/// Param is the password for the account.
		#[rpc(name = "personal_newAccount")]
		fn new_account(&self, String) -> Result<H160, Error>;

		/// Unlocks specified account for use (can only be one unlocked account at one moment)
		#[rpc(name = "personal_unlockAccount")]
		fn unlock_account(&self, H160, String, Option<U128>) -> Result<bool, Error>;

		/// Sends transaction and signs it in single call. The account is not unlocked in such case.
		#[rpc(meta, name = "personal_sendTransaction")]
		fn send_transaction(&self, Self::Metadata, TransactionRequest, String) -> BoxFuture<H256, Error>;

		/// Deprecated alias for `personal_sendTransaction`.
		#[rpc(meta, name = "personal_signAndSendTransaction")]
		fn sign_and_send_transaction(&self, Self::Metadata, TransactionRequest, String) -> BoxFuture<H256, Error>;

	}
}
