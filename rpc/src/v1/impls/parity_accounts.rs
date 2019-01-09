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
use v1::types::{H160 as RpcH160, H256 as RpcH256, H520 as RpcH520, Derive, DeriveHierarchical, DeriveHash, ExtAccountInfo};
use ethkey::Password;

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
}

impl ParityAccounts for ParityAccountsClient {
	fn all_accounts_info(&self) -> Result<BTreeMap<RpcH160, ExtAccountInfo>> {
		let info = self.accounts.accounts_info().map_err(|e| errors::account("Could not fetch account info.", e))?;
		let other = self.accounts.addresses_info();

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

	fn new_account_from_phrase(&self, phrase: String, pass: Password) -> Result<RpcH160> {
		let brain = Brain::new(phrase).generate().unwrap();
		self.accounts.insert_account(brain.secret().clone(), &pass)
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn new_account_from_wallet(&self, json: String, pass: Password) -> Result<RpcH160> {
		self.accounts.import_presale(json.as_bytes(), &pass)
			.or_else(|_| self.accounts.import_wallet(json.as_bytes(), &pass, true))
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn new_account_from_secret(&self, secret: RpcH256, pass: Password) -> Result<RpcH160> {
		let secret = Secret::from_unsafe_slice(&secret.0)
			.map_err(|e| errors::account("Could not create account.", e))?;
		self.accounts.insert_account(secret, &pass)
			.map(Into::into)
			.map_err(|e| errors::account("Could not create account.", e))
	}

	fn test_password(&self, account: RpcH160, password: Password) -> Result<bool> {
		let account: Address = account.into();

		self.accounts
			.test_password(&account, &password)
			.map_err(|e| errors::account("Could not fetch account info.", e))
	}

	fn change_password(&self, account: RpcH160, password: Password, new_password: Password) -> Result<bool> {
		let account: Address = account.into();
		self.accounts
			.change_password(&account, password, new_password)
			.map(|_| true)
			.map_err(|e| errors::account("Could not fetch account info.", e))
	}

	fn kill_account(&self, account: RpcH160, password: Password) -> Result<bool> {
		let account: Address = account.into();
		self.accounts
			.kill_account(&account, &password)
			.map(|_| true)
			.map_err(|e| errors::account("Could not delete account.", e))
	}

	fn remove_address(&self, addr: RpcH160) -> Result<bool> {
		let addr: Address = addr.into();

		self.accounts.remove_address(addr);
		Ok(true)
	}

	fn set_account_name(&self, addr: RpcH160, name: String) -> Result<bool> {
		let addr: Address = addr.into();

		self.accounts.set_account_name(addr.clone(), name.clone())
			.unwrap_or_else(|_| self.accounts.set_address_name(addr, name));
		Ok(true)
	}

	fn set_account_meta(&self, addr: RpcH160, meta: String) -> Result<bool> {
		let addr: Address = addr.into();

		self.accounts.set_account_meta(addr.clone(), meta.clone())
			.unwrap_or_else(|_| self.accounts.set_address_meta(addr, meta));
		Ok(true)
	}

	fn import_geth_accounts(&self, addresses: Vec<RpcH160>) -> Result<Vec<RpcH160>> {
		self.accounts
			.import_geth_accounts(into_vec(addresses), false)
			.map(into_vec)
			.map_err(|e| errors::account("Couldn't import Geth accounts", e))
	}

	fn geth_accounts(&self) -> Result<Vec<RpcH160>> {
		Ok(into_vec(self.accounts.list_geth_accounts(false)))
	}

	fn create_vault(&self, name: String, password: Password) -> Result<bool> {
		self.accounts
			.create_vault(&name, &password)
			.map_err(|e| errors::account("Could not create vault.", e))
			.map(|_| true)
	}

	fn open_vault(&self, name: String, password: Password) -> Result<bool> {
		self.accounts
			.open_vault(&name, &password)
			.map_err(|e| errors::account("Could not open vault.", e))
			.map(|_| true)
	}

	fn close_vault(&self, name: String) -> Result<bool> {
		self.accounts
			.close_vault(&name)
			.map_err(|e| errors::account("Could not close vault.", e))
			.map(|_| true)
	}

	fn list_vaults(&self) -> Result<Vec<String>> {
		self.accounts
			.list_vaults()
			.map_err(|e| errors::account("Could not list vaults.", e))
	}

	fn list_opened_vaults(&self) -> Result<Vec<String>> {
		self.accounts
			.list_opened_vaults()
			.map_err(|e| errors::account("Could not list vaults.", e))
	}

	fn change_vault_password(&self, name: String, new_password: Password) -> Result<bool> {
		self.accounts
			.change_vault_password(&name, &new_password)
			.map_err(|e| errors::account("Could not change vault password.", e))
			.map(|_| true)
	}

	fn change_vault(&self, address: RpcH160, new_vault: String) -> Result<bool> {
		self.accounts
			.change_vault(address.into(), &new_vault)
			.map_err(|e| errors::account("Could not change vault.", e))
			.map(|_| true)
	}

	fn get_vault_meta(&self, name: String) -> Result<String> {
		self.accounts
			.get_vault_meta(&name)
			.map_err(|e| errors::account("Could not get vault metadata.", e))
	}

	fn set_vault_meta(&self, name: String, meta: String) -> Result<bool> {
		self.accounts
			.set_vault_meta(&name, &meta)
			.map_err(|e| errors::account("Could not update vault metadata.", e))
			.map(|_| true)
	}

	fn derive_key_index(&self, addr: RpcH160, password: Password, derivation: DeriveHierarchical, save_as_account: bool) -> Result<RpcH160> {
		let addr: Address = addr.into();
		self.accounts
			.derive_account(
				&addr,
				Some(password),
				Derive::from(derivation).to_derivation()
					.map_err(|c| errors::account("Could not parse derivation request: {:?}", c))?,
				save_as_account)
			.map(Into::into)
			.map_err(|e| errors::account("Could not derive account.", e))
	}

	fn derive_key_hash(&self, addr: RpcH160, password: Password, derivation: DeriveHash, save_as_account: bool) -> Result<RpcH160> {
		let addr: Address = addr.into();
		self.accounts
			.derive_account(
				&addr,
				Some(password),
				Derive::from(derivation).to_derivation()
					.map_err(|c| errors::account("Could not parse derivation request: {:?}", c))?,
				save_as_account)
			.map(Into::into)
			.map_err(|e| errors::account("Could not derive account.", e))
	}

	fn export_account(&self, addr: RpcH160, password: Password) -> Result<KeyFile> {
		let addr = addr.into();
		self.accounts
			.export_account(
				&addr,
				password,
			)
			.map(Into::into)
			.map_err(|e| errors::account("Could not export account.", e))
	}

	fn sign_message(&self, addr: RpcH160, password: Password, message: RpcH256) -> Result<RpcH520> {
		self.accounts
			.sign(
				addr.into(),
				Some(password),
				message.into()
			)
			.map(Into::into)
			.map_err(|e| errors::account("Could not sign message.", e))
	}

	fn hardware_pin_matrix_ack(&self, path: String, pin: String) -> Result<bool> {
		self.accounts.hardware_pin_matrix_ack(&path, &pin).map_err(|e| errors::account("Error communicating with hardware wallet.", e))
	}
}

fn into_vec<A, B>(a: Vec<A>) -> Vec<B> where
	A: Into<B>
{
	a.into_iter().map(Into::into).collect()
}
