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
use std::collections::BTreeMap;
use jsonrpc_core::{Value, Error};

use v1::helpers::auto_args::Wrap;
use v1::types::{H160, H256, U256, TransactionRequest, TransactionModification, ConfirmationRequest};

build_rpc_trait! {
	/// Personal rpc interface. Safe (read-only) functions.
	pub trait Personal {
		/// Lists all stored accounts
		#[rpc(name = "personal_listAccounts")]
		fn accounts(&self) -> Result<Vec<H160>, Error>;

		/// Returns accounts information.
		#[rpc(name = "personal_accountsInfo")]
		fn accounts_info(&self) -> Result<BTreeMap<String, Value>, Error>;
	}
}

build_rpc_trait! {
	/// Personal rpc methods altering stored accounts or their settings.
	pub trait PersonalAccounts {

		/// Creates new account (it becomes new current unlocked account)
		/// Param is the password for the account.
		#[rpc(name = "personal_newAccount")]
		fn new_account(&self, String) -> Result<H160, Error>;

		/// Creates new account from the given phrase using standard brainwallet mechanism.
		/// Second parameter is password for the new account.
		#[rpc(name = "personal_newAccountFromPhrase")]
		fn new_account_from_phrase(&self, String, String) -> Result<H160, Error>;

		/// Creates new account from the given JSON wallet.
		/// Second parameter is password for the wallet and the new account.
		#[rpc(name = "personal_newAccountFromWallet")]
		fn new_account_from_wallet(&self, String, String) -> Result<H160, Error>;

		/// Creates new account from the given raw secret.
		/// Second parameter is password for the new account.
		#[rpc(name = "personal_newAccountFromSecret")]
		fn new_account_from_secret(&self, H256, String) -> Result<H160, Error>;

		/// Unlocks specified account for use (can only be one unlocked account at one moment)
		#[rpc(name = "personal_unlockAccount")]
		fn unlock_account(&self, H160, String, Option<u64>) -> Result<bool, Error>;

		/// Returns true if given `password` would unlock given `account`.
		/// Arguments: `account`, `password`.
		#[rpc(name = "personal_testPassword")]
		fn test_password(&self, H160, String) -> Result<bool, Error>;

		/// Changes an account's password.
		/// Arguments: `account`, `password`, `new_password`.
		#[rpc(name = "personal_changePassword")]
		fn change_password(&self, H160, String, String) -> Result<bool, Error>;

		/// Sends transaction and signs it in single call. The account is not unlocked in such case.
		#[rpc(name = "personal_signAndSendTransaction")]
		fn sign_and_send_transaction(&self, TransactionRequest, String) -> Result<H256, Error>;

		/// Set an account's name.
		#[rpc(name = "personal_setAccountName")]
		fn set_account_name(&self, H160, String) -> Result<bool, Error>;

		/// Set an account's metadata string.
		#[rpc(name = "personal_setAccountMeta")]
		fn set_account_meta(&self, H160, String) -> Result<bool, Error>;

		/// Imports a number of Geth accounts, with the list provided as the argument.
		#[rpc(name = "personal_importGethAccounts")]
		fn import_geth_accounts(&self, Vec<H160>) -> Result<Vec<H160>, Error>;

		/// Returns the accounts available for importing from Geth.
		#[rpc(name = "personal_listGethAccounts")]
		fn geth_accounts(&self) -> Result<Vec<H160>, Error>;
	}
}

build_rpc_trait! {
	/// Personal extension for confirmations rpc interface.
	pub trait PersonalSigner {

		/// Returns a list of items to confirm.
		#[rpc(name = "personal_requestsToConfirm")]
		fn requests_to_confirm(&self) -> Result<Vec<ConfirmationRequest>, Error>;

		/// Confirm specific request.
		#[rpc(name = "personal_confirmRequest")]
		fn confirm_request(&self, U256, TransactionModification, String) -> Result<Value, Error>;

		/// Reject the confirmation request.
		#[rpc(name = "personal_rejectRequest")]
		fn reject_request(&self, U256) -> Result<bool, Error>;

		/// Generates new authorization token.
		#[rpc(name = "personal_generateAuthorizationToken")]
		fn generate_token(&self) -> Result<String, Error>;
	}
}

