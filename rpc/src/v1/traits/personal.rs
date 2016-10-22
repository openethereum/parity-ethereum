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
	fn accounts(&self, _: Params) -> Result<Value, Error>;

	/// Creates new account (it becomes new current unlocked account)
	/// Param is the password for the account.
	fn new_account(&self, _: Params) -> Result<Value, Error>;

	/// Creates new account from the given phrase using standard brainwallet mechanism.
	/// Second parameter is password for the new account.
	fn new_account_from_phrase(&self, _: Params) -> Result<Value, Error>;

	/// Creates new account from the given JSON wallet.
	/// Second parameter is password for the wallet and the new account.
	fn new_account_from_wallet(&self, params: Params) -> Result<Value, Error>;

	/// Unlocks specified account for use (can only be one unlocked account at one moment)
	fn unlock_account(&self, _: Params) -> Result<Value, Error>;

	/// Returns true if given `password` would unlock given `account`.
	/// Arguments: `account`, `password`.
	fn test_password(&self, _: Params) -> Result<Value, Error>;

	/// Changes an account's password.
	/// Arguments: `account`, `password`, `new_password`.
	fn change_password(&self, _: Params) -> Result<Value, Error>;

	/// Sends transaction and signs it in single call. The account is not unlocked in such case.
	fn sign_and_send_transaction(&self, _: Params) -> Result<Value, Error>;

	/// Set an account's name.
	fn set_account_name(&self, _: Params) -> Result<Value, Error>;

	/// Set an account's metadata string.
	fn set_account_meta(&self, _: Params) -> Result<Value, Error>;

	/// Returns accounts information.
	fn accounts_info(&self, _: Params) -> Result<Value, Error>;

	/// Returns the accounts available for importing from Geth.
	fn geth_accounts(&self, _: Params) -> Result<Value, Error>;

	/// Imports a number of Geth accounts, with the list provided as the argument.
	fn import_geth_accounts(&self, _: Params) -> Result<Value, Error>;

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("personal_listAccounts", Personal::accounts);
		delegate.add_method("personal_newAccount", Personal::new_account);
		delegate.add_method("personal_newAccountFromPhrase", Personal::new_account_from_phrase);
		delegate.add_method("personal_newAccountFromWallet", Personal::new_account_from_wallet);
		delegate.add_method("personal_unlockAccount", Personal::unlock_account);
		delegate.add_method("personal_testPassword", Personal::test_password);
		delegate.add_method("personal_changePassword", Personal::change_password);
		delegate.add_method("personal_signAndSendTransaction", Personal::sign_and_send_transaction);
		delegate.add_method("personal_setAccountName", Personal::set_account_name);
		delegate.add_method("personal_setAccountMeta", Personal::set_account_meta);
		delegate.add_method("personal_accountsInfo", Personal::accounts_info);
		delegate.add_method("personal_listGethAccounts", Personal::geth_accounts);
		delegate.add_method("personal_importGethAccounts", Personal::import_geth_accounts);

		delegate
	}
}

/// Personal extension for confirmations rpc interface.
pub trait PersonalSigner: Sized + Send + Sync + 'static {

	/// Returns a list of items to confirm.
	fn requests_to_confirm(&self, _: Params) -> Result<Value, Error>;

	/// Confirm specific request.
	fn confirm_request(&self, _: Params) -> Result<Value, Error>;

	/// Reject the confirmation request.
	fn reject_request(&self, _: Params) -> Result<Value, Error>;

	/// Generates new authorization token.
	fn generate_token(&self, _: Params) -> Result<Value, Error>;

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("personal_requestsToConfirm", PersonalSigner::requests_to_confirm);
		delegate.add_method("personal_confirmRequest", PersonalSigner::confirm_request);
		delegate.add_method("personal_rejectRequest", PersonalSigner::reject_request);
		delegate.add_method("personal_generateAuthorizationToken", PersonalSigner::generate_token);
		delegate
	}
}

