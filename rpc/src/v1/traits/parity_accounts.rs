// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Parity Accounts-related rpc interface.
use std::collections::BTreeMap;

use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use ethereum_types::{H160, H256, H520};
use ethkey::Password;
use ethstore::KeyFile;
use v1::types::{DeriveHash, DeriveHierarchical, ExtAccountInfo};
use v1::types::AccountInfo;

/// Parity-specific read-only accounts rpc interface.
#[rpc(server)]
pub trait ParityAccountsInfo {
	/// Returns accounts information.
	#[rpc(name = "parity_accountsInfo")]
	fn accounts_info(&self) -> Result<BTreeMap<H160, AccountInfo>>;

	/// Returns default account for dapp.
	#[rpc(name = "parity_defaultAccount")]
	fn default_account(&self) -> Result<H160>;
}

/// Personal Parity rpc interface.
#[rpc(server)]
pub trait ParityAccounts {
	/// Returns accounts information.
	#[rpc(name = "parity_allAccountsInfo")]
	fn all_accounts_info(&self) -> Result<BTreeMap<H160, ExtAccountInfo>>;

	/// Creates new account from the given phrase using standard brainwallet mechanism.
	/// Second parameter is password for the new account.
	#[rpc(name = "parity_newAccountFromPhrase")]
	fn new_account_from_phrase(&self, _: String, _: Password) -> Result<H160>;

	/// Creates new account from the given JSON wallet.
	/// Second parameter is password for the wallet and the new account.
	#[rpc(name = "parity_newAccountFromWallet")]
	fn new_account_from_wallet(&self, _: String, _: Password) -> Result<H160>;

	/// Creates new account from the given raw secret.
	/// Second parameter is password for the new account.
	#[rpc(name = "parity_newAccountFromSecret")]
	fn new_account_from_secret(&self, _: H256, _: Password) -> Result<H160>;

	/// Returns true if given `password` would unlock given `account`.
	/// Arguments: `account`, `password`.
	#[rpc(name = "parity_testPassword")]
	fn test_password(&self, _: H160, _: Password) -> Result<bool>;

	/// Changes an account's password.
	/// Arguments: `account`, `password`, `new_password`.
	#[rpc(name = "parity_changePassword")]
	fn change_password(&self, _: H160, _: Password, _: Password) -> Result<bool>;

	/// Permanently deletes an account.
	/// Arguments: `account`, `password`.
	#[rpc(name = "parity_killAccount")]
	fn kill_account(&self, _: H160, _: Password) -> Result<bool>;

	/// Permanently deletes an address from the addressbook
	/// Arguments: `address`
	#[rpc(name = "parity_removeAddress")]
	fn remove_address(&self, _: H160) -> Result<bool>;

	/// Set an account's name.
	#[rpc(name = "parity_setAccountName")]
	fn set_account_name(&self, _: H160, _: String) -> Result<bool>;

	/// Set an account's metadata string.
	#[rpc(name = "parity_setAccountMeta")]
	fn set_account_meta(&self, _: H160, _: String) -> Result<bool>;

	/// Imports a number of Geth accounts, with the list provided as the argument.
	#[rpc(name = "parity_importGethAccounts")]
	fn import_geth_accounts(&self, _: Vec<H160>) -> Result<Vec<H160>>;

	/// Returns the accounts available for importing from Geth.
	#[rpc(name = "parity_listGethAccounts")]
	fn geth_accounts(&self) -> Result<Vec<H160>>;

	/// Create new vault.
	#[rpc(name = "parity_newVault")]
	fn create_vault(&self, _: String, _: Password) -> Result<bool>;

	/// Open existing vault.
	#[rpc(name = "parity_openVault")]
	fn open_vault(&self, _: String, _: Password) -> Result<bool>;

	/// Close previously opened vault.
	#[rpc(name = "parity_closeVault")]
	fn close_vault(&self, _: String) -> Result<bool>;

	/// List all vaults.
	#[rpc(name = "parity_listVaults")]
	fn list_vaults(&self) -> Result<Vec<String>>;

	/// List all currently opened vaults.
	#[rpc(name = "parity_listOpenedVaults")]
	fn list_opened_vaults(&self) -> Result<Vec<String>>;

	/// Change vault password.
	#[rpc(name = "parity_changeVaultPassword")]
	fn change_vault_password(&self, _: String, _: Password) -> Result<bool>;

	/// Change vault of the given address.
	#[rpc(name = "parity_changeVault")]
	fn change_vault(&self, _: H160, _: String) -> Result<bool>;

	/// Get vault metadata string.
	#[rpc(name = "parity_getVaultMeta")]
	fn get_vault_meta(&self, _: String) -> Result<String>;

	/// Set vault metadata string.
	#[rpc(name = "parity_setVaultMeta")]
	fn set_vault_meta(&self, _: String, _: String) -> Result<bool>;

	/// Derive new address from given account address using specific hash.
	/// Resulting address can be either saved as a new account (with the same password).
	#[rpc(name = "parity_deriveAddressHash")]
	fn derive_key_hash(&self, _: H160, _: Password, _: DeriveHash, _: bool) -> Result<H160>;

	/// Derive new address from given account address using
	/// hierarchical derivation (sequence of 32-bit integer indices).
	/// Resulting address can be either saved as a new account (with the same password).
	#[rpc(name = "parity_deriveAddressIndex")]
	fn derive_key_index(&self, _: H160, _: Password, _: DeriveHierarchical, _: bool) -> Result<H160>;

	/// Exports an account with given address if provided password matches.
	#[rpc(name = "parity_exportAccount")]
	fn export_account(&self, _: H160, _: Password) -> Result<KeyFile>;

	/// Sign raw hash with the key corresponding to address and password.
	#[rpc(name = "parity_signMessage")]
	fn sign_message(&self, _: H160, _: Password, _: H256) -> Result<H520>;
}
