// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use jsonrpc_core::Result;
use ethkey::Password;
use ethstore::KeyFile;
use v1::types::{H160, H256, H520, DappId, DeriveHash, DeriveHierarchical, ExtAccountInfo};

build_rpc_trait! {
	/// Personal Parity rpc interface.
	pub trait ParityAccounts {
		/// Returns accounts information.
		#[rpc(name = "parity_allAccountsInfo")]
		fn all_accounts_info(&self) -> Result<BTreeMap<H160, ExtAccountInfo>>;

		/// Creates new account from the given phrase using standard brainwallet mechanism.
		/// Second parameter is password for the new account.
		#[rpc(name = "parity_newAccountFromPhrase")]
		fn new_account_from_phrase(&self, String, Password) -> Result<H160>;

		/// Creates new account from the given JSON wallet.
		/// Second parameter is password for the wallet and the new account.
		#[rpc(name = "parity_newAccountFromWallet")]
		fn new_account_from_wallet(&self, String, Password) -> Result<H160>;

		/// Creates new account from the given raw secret.
		/// Second parameter is password for the new account.
		#[rpc(name = "parity_newAccountFromSecret")]
		fn new_account_from_secret(&self, H256, Password) -> Result<H160>;

		/// Returns true if given `password` would unlock given `account`.
		/// Arguments: `account`, `password`.
		#[rpc(name = "parity_testPassword")]
		fn test_password(&self, H160, Password) -> Result<bool>;

		/// Changes an account's password.
		/// Arguments: `account`, `password`, `new_password`.
		#[rpc(name = "parity_changePassword")]
		fn change_password(&self, H160, Password, Password) -> Result<bool>;

		/// Permanently deletes an account.
		/// Arguments: `account`, `password`.
		#[rpc(name = "parity_killAccount")]
		fn kill_account(&self, H160, Password) -> Result<bool>;

		/// Permanently deletes an address from the addressbook
		/// Arguments: `address`
		#[rpc(name = "parity_removeAddress")]
		fn remove_address(&self, H160) -> Result<bool>;

		/// Set an account's name.
		#[rpc(name = "parity_setAccountName")]
		fn set_account_name(&self, H160, String) -> Result<bool>;

		/// Set an account's metadata string.
		#[rpc(name = "parity_setAccountMeta")]
		fn set_account_meta(&self, H160, String) -> Result<bool>;

		/// Sets addresses exposed for particular dapp.
		/// Setting a non-empty list will also override default account.
		/// Setting `None` will resets visible account to what's visible for new dapps
		/// (does not affect default account though)
		#[rpc(name = "parity_setDappAddresses")]
		fn set_dapp_addresses(&self, DappId, Option<Vec<H160>>) -> Result<bool>;

		/// Gets accounts exposed for particular dapp.
		#[rpc(name = "parity_getDappAddresses")]
		fn dapp_addresses(&self, DappId) -> Result<Vec<H160>>;

		/// Changes dapp default address.
		/// Does not affect other accounts exposed for this dapp, but
		/// default account will always be retured as the first one.
		#[rpc(name = "parity_setDappDefaultAddress")]
		fn set_dapp_default_address(&self, DappId, H160) -> Result<bool>;

		/// Returns current dapp default address.
		/// If not set explicite for the dapp will return global default.
		#[rpc(name = "parity_getDappDefaultAddress")]
		fn dapp_default_address(&self, DappId) -> Result<H160>;

		/// Sets accounts exposed for new dapps.
		/// Setting a non-empty list will also override default account.
		/// Setting `None` exposes all internal-managed accounts.
		/// (does not affect default account though)
		#[rpc(name = "parity_setNewDappsAddresses")]
		fn set_new_dapps_addresses(&self, Option<Vec<H160>>) -> Result<bool>;

		/// Gets accounts exposed for new dapps.
		/// `None` means that all accounts are exposes.
		#[rpc(name = "parity_getNewDappsAddresses")]
		fn new_dapps_addresses(&self) -> Result<Option<Vec<H160>>>;

		/// Changes default address for new dapps (global default address)
		/// Does not affect other accounts exposed for new dapps, but
		/// default account will always be retured as the first one.
		#[rpc(name = "parity_setNewDappsDefaultAddress")]
		fn set_new_dapps_default_address(&self, H160) -> Result<bool>;

		/// Returns current default address for new dapps (global default address)
		/// In case it's not set explicite will return first available account.
		/// If no accounts are available will return `0x0`
		#[rpc(name = "parity_getNewDappsDefaultAddress")]
		fn new_dapps_default_address(&self) -> Result<H160>;

		/// Returns identified dapps that recently used RPC
		/// Includes last usage timestamp.
		#[rpc(name = "parity_listRecentDapps")]
		fn recent_dapps(&self) -> Result<BTreeMap<DappId, u64>>;

		/// Imports a number of Geth accounts, with the list provided as the argument.
		#[rpc(name = "parity_importGethAccounts")]
		fn import_geth_accounts(&self, Vec<H160>) -> Result<Vec<H160>>;

		/// Returns the accounts available for importing from Geth.
		#[rpc(name = "parity_listGethAccounts")]
		fn geth_accounts(&self) -> Result<Vec<H160>>;

		/// Create new vault.
		#[rpc(name = "parity_newVault")]
		fn create_vault(&self, String, Password) -> Result<bool>;

		/// Open existing vault.
		#[rpc(name = "parity_openVault")]
		fn open_vault(&self, String, Password) -> Result<bool>;

		/// Close previously opened vault.
		#[rpc(name = "parity_closeVault")]
		fn close_vault(&self, String) -> Result<bool>;

		/// List all vaults.
		#[rpc(name = "parity_listVaults")]
		fn list_vaults(&self) -> Result<Vec<String>>;

		/// List all currently opened vaults.
		#[rpc(name = "parity_listOpenedVaults")]
		fn list_opened_vaults(&self) -> Result<Vec<String>>;

		/// Change vault password.
		#[rpc(name = "parity_changeVaultPassword")]
		fn change_vault_password(&self, String, Password) -> Result<bool>;

		/// Change vault of the given address.
		#[rpc(name = "parity_changeVault")]
		fn change_vault(&self, H160, String) -> Result<bool>;

		/// Get vault metadata string.
		#[rpc(name = "parity_getVaultMeta")]
		fn get_vault_meta(&self, String) -> Result<String>;

		/// Set vault metadata string.
		#[rpc(name = "parity_setVaultMeta")]
		fn set_vault_meta(&self, String, String) -> Result<bool>;

		/// Derive new address from given account address using specific hash.
		/// Resulting address can be either saved as a new account (with the same password).
		#[rpc(name = "parity_deriveAddressHash")]
		fn derive_key_hash(&self, H160, Password, DeriveHash, bool) -> Result<H160>;

		/// Derive new address from given account address using
		/// hierarchical derivation (sequence of 32-bit integer indices).
		/// Resulting address can be either saved as a new account (with the same password).
		#[rpc(name = "parity_deriveAddressIndex")]
		fn derive_key_index(&self, H160, Password, DeriveHierarchical, bool) -> Result<H160>;

		/// Exports an account with given address if provided password matches.
		#[rpc(name = "parity_exportAccount")]
		fn export_account(&self, H160, Password) -> Result<KeyFile>;

		/// Sign raw hash with the key corresponding to address and password.
		#[rpc(name = "parity_signMessage")]
		fn sign_message(&self, H160, Password, H256) -> Result<H520>;

		/// Send a PinMatrixAck to a hardware wallet, unlocking it
		#[rpc(name = "parity_hardwarePinMatrixAck")]
		fn hardware_pin_matrix_ack(&self, String, String) -> Result<bool>;
	}
}
