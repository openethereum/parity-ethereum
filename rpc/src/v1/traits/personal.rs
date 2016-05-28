// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
use std::sync::Arc;
use jsonrpc_core::*;

/// Personal rpc interface.
pub trait Personal: Sized + Send + Sync + 'static {

	/// Lists all stored accounts
	fn accounts(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Creates new account (it becomes new current unlocked account)
	fn new_account(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Unlocks specified account for use (can only be one unlocked account at one moment)
	fn unlock_account(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Sends transaction and signs it in single call. The account is not unlocked in such case.
	fn sign_and_send_transaction(&self, _: Params) -> Result<Value, Error> { rpc_unimplemented!() }

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("personal_listAccounts", Personal::accounts);
		delegate.add_method("personal_newAccount", Personal::new_account);
		delegate.add_method("personal_unlockAccount", Personal::unlock_account);
		delegate.add_method("personal_signAndSendTransaction", Personal::sign_and_send_transaction);
		delegate
	}
}

/// Personal extension for transactions confirmations rpc interface.
pub trait PersonalSigner: Sized + Send + Sync + 'static {

	/// Returns a list of transactions to confirm.
	fn transactions_to_confirm(&self, _: Params) -> Result<Value, Error>;

	/// Confirm and send a specific transaction.
	fn confirm_transaction(&self, _: Params) -> Result<Value, Error>;

	/// Reject the transaction request.
	fn reject_transaction(&self, _: Params) -> Result<Value, Error>;

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("personal_transactionsToConfirm", PersonalSigner::transactions_to_confirm);
		delegate.add_method("personal_confirmTransaction", PersonalSigner::confirm_transaction);
		delegate.add_method("personal_rejectTransaction", PersonalSigner::reject_transaction);
		delegate
	}
}

