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

//! Account management (personal) rpc implementation
use std::sync::Arc;
use std::collections::btree_map::{BTreeMap, Entry};
use ethereum_types::Address;

use ethkey::{Brain, Generator, Secret};
use ethstore::KeyFile;
use ethcore::account_provider::AccountProvider;

use jsonrpc_core::Result;
use v1::helpers::errors;
use v1::traits::ParityAccounts;
use v1::types::{H160 as RpcH160, H256 as RpcH256, H520 as RpcH520, DappId, Derive, DeriveHierarchical, DeriveHash, ExtAccountInfo};

/// Account management (personal) rpc implementation.
pub struct ParityAccountsClient {
	accounts: Arc<AccountProvider>,
}

impl ParityAccountsClient {
	/// Creates new PersonalClient
	pub fn new(store: &Arc<AccountProvider>) -> Self {
		ParityAccountsClient {
			accounts: store.clone(),
		}
	}

	/// Attempt to get the `Arc<AccountProvider>`, errors if provider was not
	/// set.
	fn account_provider(&self) -> Result<Arc<AccountProvider>> {
		Ok(self.accounts.clone())
	}
}

impl ParityAccounts for ParityAccountsClient {
	fn all_accounts_info(&self) -> Result<BTreeMap<RpcH160, ExtAccountInfo>> {
		let store = self.account_provider()?;
		let info = store.accounts_info().map_err(|e| errors::account("Could not fetch account info.", e))?;
		let other = store.addresses_info();

		let account_iter = info
			.into_iter()
			.chain(other.into_iter())
			.map(|(address, v)| (address.into(), ExtAccountInfo {
				name: v.name,
				meta: v.meta,
				uuid: v.uuid.map(|uuid| uuid.to_string())
			}));

		let mut accounts: BTreeMap<RpcH160, ExtAccountInfo> = BTreeMap::new();

		for (address, account) in account_iter {
			match accounts.entry(address) {
				// Insert only if occupied entry isn't already an account with UUID
				Entry::Occupied(ref mut occupied) if occupied.get().uuid.is_none() => {
					occupied.insert(account);
				},
				Entry::Vacant(vacant) => {
					vacant.insert(account);
				},
				_ => {}
			};
		}

		Ok(accounts)
	}

	fn new_account_from_phrase(&self, phrase: String, pass: String) -> Result<RpcH160> {
		let store = self.account_provider()?;

		let brain = Brain::new(phrase).generate().unwrap();
		store.insert_account(brain.secret().clone(), &pass)
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn new_account_from_wallet(&self, json: String, pass: String) -> Result<RpcH160> {
		let store = self.account_provider()?;

		store.import_presale(json.as_bytes(), &pass)
			.or_else(|_| store.import_wallet(json.as_bytes(), &pass, true))
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn new_account_from_secret(&self, secret: RpcH256, pass: String) -> Result<RpcH160> {
		let store = self.account_provider()?;

		let secret = Secret::from_unsafe_slice(&secret.0)
			.map_err(|e| errors::account("Could not create account.", e))?;
		store.insert_account(secret, &pass)
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn test_password(&self, account: RpcH160, password: String) -> Result<bool> {
		let account: Address = account.into();

		self.account_provider()?
			.test_password(&account, &password)
			.map_err(|e| errors::account("Could not fetch account info.", e))
	}

	fn change_password(&self, account: RpcH160, password: String, new_password: String) -> Result<bool> {
		let account: Address = account.into();
		self.account_provider()?
			.change_password(&account, password, new_password)
			.map(|_| true)
			.map_err(|e| errors::account("Could not fetch account info.", e))
	}

	fn kill_account(&self, account: RpcH160, password: String) -> Result<bool> {
		let account: Address = account.into();
		self.account_provider()?
			.kill_account(&account, &password)
			.map(|_| true)
			.map_err(|e| errors::account("Could not delete account.", e))
	}

	fn remove_address(&self, addr: RpcH160) -> Result<bool> {
		let store = self.account_provider()?;
		let addr: Address = addr.into();

		store.remove_address(addr);
		Ok(true)
	}

	fn set_account_name(&self, addr: RpcH160, name: String) -> Result<bool> {
		let store = self.account_provider()?;
		let addr: Address = addr.into();

		store.set_account_name(addr.clone(), name.clone())
			.unwrap_or_else(|_| store.set_address_name(addr, name));
		Ok(true)
	}

	fn set_account_meta(&self, addr: RpcH160, meta: String) -> Result<bool> {
		let store = self.account_provider()?;
		let addr: Address = addr.into();

		store.set_account_meta(addr.clone(), meta.clone())
			.unwrap_or_else(|_| store.set_address_meta(addr, meta));
		Ok(true)
	}

	fn set_dapp_addresses(&self, dapp: DappId, addresses: Option<Vec<RpcH160>>) -> Result<bool> {
		let store = self.account_provider()?;

		store.set_dapp_addresses(dapp.into(), addresses.map(into_vec))
			.map_err(|e| errors::account("Couldn't set dapp addresses.", e))
			.map(|_| true)
	}

	fn dapp_addresses(&self, dapp: DappId) -> Result<Vec<RpcH160>> {
		let store = self.account_provider()?;

		store.dapp_addresses(dapp.into())
			.map_err(|e| errors::account("Couldn't get dapp addresses.", e))
			.map(into_vec)
	}

	fn set_dapp_default_address(&self, dapp: DappId, address: RpcH160) -> Result<bool> {
		let store = self.account_provider()?;

		store.set_dapp_default_address(dapp.into(), address.into())
			.map_err(|e| errors::account("Couldn't set dapp default address.", e))
			.map(|_| true)
	}

	fn dapp_default_address(&self, dapp: DappId) -> Result<RpcH160> {
		let store = self.account_provider()?;

		store.dapp_default_address(dapp.into())
			.map_err(|e| errors::account("Couldn't get dapp default address.", e))
			.map(Into::into)
	}

	fn set_new_dapps_addresses(&self, addresses: Option<Vec<RpcH160>>) -> Result<bool> {
		let store = self.account_provider()?;

		store
			.set_new_dapps_addresses(addresses.map(into_vec))
			.map_err(|e| errors::account("Couldn't set dapps addresses.", e))
			.map(|_| true)
	}

	fn new_dapps_addresses(&self) -> Result<Option<Vec<RpcH160>>> {
		let store = self.account_provider()?;

		store.new_dapps_addresses()
			.map_err(|e| errors::account("Couldn't get dapps addresses.", e))
			.map(|accounts| accounts.map(into_vec))
	}

	fn set_new_dapps_default_address(&self, address: RpcH160) -> Result<bool> {
		let store = self.account_provider()?;

		store.set_new_dapps_default_address(address.into())
			.map_err(|e| errors::account("Couldn't set new dapps default address.", e))
			.map(|_| true)
	}

	fn new_dapps_default_address(&self) -> Result<RpcH160> {
		let store = self.account_provider()?;

		store.new_dapps_default_address()
			.map_err(|e| errors::account("Couldn't get new dapps default address.", e))
			.map(Into::into)
	}

	fn recent_dapps(&self) -> Result<BTreeMap<DappId, u64>> {
		let store = self.account_provider()?;

		store.recent_dapps()
			.map_err(|e| errors::account("Couldn't get recent dapps.", e))
			.map(|map| map.into_iter().map(|(k, v)| (k.into(), v)).collect())
	}

	fn import_geth_accounts(&self, addresses: Vec<RpcH160>) -> Result<Vec<RpcH160>> {
		let store = self.account_provider()?;

		store
			.import_geth_accounts(into_vec(addresses), false)
			.map(into_vec)
			.map_err(|e| errors::account("Couldn't import Geth accounts", e))
	}

	fn geth_accounts(&self) -> Result<Vec<RpcH160>> {
		let store = self.account_provider()?;

		Ok(into_vec(store.list_geth_accounts(false)))
	}

	fn create_vault(&self, name: String, password: String) -> Result<bool> {
		self.account_provider()?
			.create_vault(&name, &password)
			.map_err(|e| errors::account("Could not create vault.", e))
			.map(|_| true)
	}

	fn open_vault(&self, name: String, password: String) -> Result<bool> {
		self.account_provider()?
			.open_vault(&name, &password)
			.map_err(|e| errors::account("Could not open vault.", e))
			.map(|_| true)
	}

	fn close_vault(&self, name: String) -> Result<bool> {
		self.account_provider()?
			.close_vault(&name)
			.map_err(|e| errors::account("Could not close vault.", e))
			.map(|_| true)
	}

	fn list_vaults(&self) -> Result<Vec<String>> {
		self.account_provider()?
			.list_vaults()
			.map_err(|e| errors::account("Could not list vaults.", e))
	}

	fn list_opened_vaults(&self) -> Result<Vec<String>> {
		self.account_provider()?
			.list_opened_vaults()
			.map_err(|e| errors::account("Could not list vaults.", e))
	}

	fn change_vault_password(&self, name: String, new_password: String) -> Result<bool> {
		self.account_provider()?
			.change_vault_password(&name, &new_password)
			.map_err(|e| errors::account("Could not change vault password.", e))
			.map(|_| true)
	}

	fn change_vault(&self, address: RpcH160, new_vault: String) -> Result<bool> {
		self.account_provider()?
			.change_vault(address.into(), &new_vault)
			.map_err(|e| errors::account("Could not change vault.", e))
			.map(|_| true)
	}

	fn get_vault_meta(&self, name: String) -> Result<String> {
		self.account_provider()?
			.get_vault_meta(&name)
			.map_err(|e| errors::account("Could not get vault metadata.", e))
	}

	fn set_vault_meta(&self, name: String, meta: String) -> Result<bool> {
		self.account_provider()?
			.set_vault_meta(&name, &meta)
			.map_err(|e| errors::account("Could not update vault metadata.", e))
			.map(|_| true)
	}

	fn derive_key_index(&self, addr: RpcH160, password: String, derivation: DeriveHierarchical, save_as_account: bool) -> Result<RpcH160> {
		let addr: Address = addr.into();
		self.account_provider()?
			.derive_account(
				&addr,
				Some(password),
				Derive::from(derivation).to_derivation()
					.map_err(|c| errors::account("Could not parse derivation request: {:?}", c))?,
				save_as_account)
			.map(Into::into)
			.map_err(|e| errors::account("Could not derive account.", e))
	}

	fn derive_key_hash(&self, addr: RpcH160, password: String, derivation: DeriveHash, save_as_account: bool) -> Result<RpcH160> {
		let addr: Address = addr.into();
		self.account_provider()?
			.derive_account(
				&addr,
				Some(password),
				Derive::from(derivation).to_derivation()
					.map_err(|c| errors::account("Could not parse derivation request: {:?}", c))?,
				save_as_account)
			.map(Into::into)
			.map_err(|e| errors::account("Could not derive account.", e))
	}

	fn export_account(&self, addr: RpcH160, password: String) -> Result<KeyFile> {
		let addr = addr.into();
		self.account_provider()?
			.export_account(
				&addr,
				password,
			)
			.map(Into::into)
			.map_err(|e| errors::account("Could not export account.", e))
	}

	fn sign_message(&self, addr: RpcH160, password: String, message: RpcH256) -> Result<RpcH520> {
		self.account_provider()?
			.sign(
				addr.into(),
				Some(password),
				message.into()
			)
			.map(Into::into)
			.map_err(|e| errors::account("Could not sign message.", e))
	}

	fn hardware_pin_matrix_ack(&self, path: String, pin: String) -> Result<bool> {
		let store = self.account_provider()?;
		Ok(store.hardware_pin_matrix_ack(&path, &pin).map_err(|e| errors::account("Error communicating with hardware wallet.", e))?)
	}
}

fn into_vec<A, B>(a: Vec<A>) -> Vec<B> where
	A: Into<B>
{
	a.into_iter().map(Into::into).collect()
}
