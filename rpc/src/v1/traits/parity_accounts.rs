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

//! Parity Accounts-related rpc interface.
use std::collections::BTreeMap;

use jsonrpc_core::Error;
use v1::types::{H160, H256, DappId};

build_rpc_trait! {
	/// Personal Parity rpc interface.
	pub trait ParityAccounts {
		/// Returns accounts information.
		#[rpc(name = "parity_allAccountsInfo")]
		fn all_accounts_info(&self) -> Result<BTreeMap<String, BTreeMap<String, String>>, Error>;

		/// Creates new account from the given phrase using standard brainwallet mechanism.
		/// Second parameter is password for the new account.
		#[rpc(name = "parity_newAccountFromPhrase")]
		fn new_account_from_phrase(&self, String, String) -> Result<H160, Error>;

		/// Creates new account from the given JSON wallet.
		/// Second parameter is password for the wallet and the new account.
		#[rpc(name = "parity_newAccountFromWallet")]
		fn new_account_from_wallet(&self, String, String) -> Result<H160, Error>;

		/// Creates new account from the given raw secret.
		/// Second parameter is password for the new account.
		#[rpc(name = "parity_newAccountFromSecret")]
		fn new_account_from_secret(&self, H256, String) -> Result<H160, Error>;

		/// Returns true if given `password` would unlock given `account`.
		/// Arguments: `account`, `password`.
		#[rpc(name = "parity_testPassword")]
		fn test_password(&self, H160, String) -> Result<bool, Error>;

		/// Changes an account's password.
		/// Arguments: `account`, `password`, `new_password`.
		#[rpc(name = "parity_changePassword")]
		fn change_password(&self, H160, String, String) -> Result<bool, Error>;

		/// Permanently deletes an account.
		/// Arguments: `account`, `password`.
		#[rpc(name = "parity_killAccount")]
		fn kill_account(&self, H160, String) -> Result<bool, Error>;

		/// Permanently deletes an address from the addressbook
		/// Arguments: `address`
		#[rpc(name = "parity_removeAddress")]
		fn remove_address(&self, H160) -> Result<bool, Error>;

		/// Set an account's name.
		#[rpc(name = "parity_setAccountName")]
		fn set_account_name(&self, H160, String) -> Result<bool, Error>;

		/// Set an account's metadata string.
		#[rpc(name = "parity_setAccountMeta")]
		fn set_account_meta(&self, H160, String) -> Result<bool, Error>;

		/// Sets account visibility
		#[rpc(name = "parity_setAccountVisiblity")]
		fn set_account_visibility(&self, H160, H256, bool) -> Result<bool, Error>;

		/// Sets accounts exposed for particular dapp.
		#[rpc(name = "parity_setDappsAddresses")]
		fn set_dapps_addresses(&self, DappId, Vec<H160>) -> Result<bool, Error>;

		/// Gets accounts exposed for particular dapp.
		#[rpc(name = "parity_getDappsAddresses")]
		fn dapps_addresses(&self, DappId) -> Result<Vec<H160>, Error>;

		/// Sets accounts exposed for new dapps.
		/// `None` means that all accounts will be exposed.
		#[rpc(name = "parity_setNewDappsWhitelist")]
		fn set_new_dapps_whitelist(&self, Option<Vec<H160>>) -> Result<bool, Error>;

		/// Gets accounts exposed for new dapps.
		/// `None` means that all accounts will be exposed.
		#[rpc(name = "parity_getNewDappsWhitelist")]
		fn new_dapps_whitelist(&self) -> Result<Option<Vec<H160>>, Error>;

		/// Returns identified dapps that recently used RPC
		/// Includes last usage timestamp.
		#[rpc(name = "parity_listRecentDapps")]
		fn recent_dapps(&self) -> Result<BTreeMap<DappId, u64>, Error>;

		/// Imports a number of Geth accounts, with the list provided as the argument.
		#[rpc(name = "parity_importGethAccounts")]
		fn import_geth_accounts(&self, Vec<H160>) -> Result<Vec<H160>, Error>;

		/// Returns the accounts available for importing from Geth.
		#[rpc(name = "parity_listGethAccounts")]
		fn geth_accounts(&self) -> Result<Vec<H160>, Error>;
	}
}
