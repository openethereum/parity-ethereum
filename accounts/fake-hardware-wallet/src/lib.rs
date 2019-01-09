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

//! Dummy module for platforms that does not provide support for hardware wallets (libusb)

extern crate ethereum_types;
extern crate ethkey;

use std::fmt;
use ethereum_types::U256;
use ethkey::{Address, Signature};

pub struct WalletInfo {
	pub address: Address,
	pub name: String,
	pub manufacturer: String,
}

#[derive(Debug)]
/// `ErrorType` for devices with no `hardware wallet`
pub enum Error {
	NoWallet,
	KeyNotFound,
}

pub struct TransactionInfo {
	/// Nonce
	pub nonce: U256,
	/// Gas price
	pub gas_price: U256,
	/// Gas limit
	pub gas_limit: U256,
	/// Receiver
	pub to: Option<Address>,
	/// Value
	pub value: U256,
	/// Data
	pub data: Vec<u8>,
	/// Chain ID
	pub chain_id: Option<u64>,
}

pub enum KeyPath {
	/// Ethereum.
	Ethereum,
	/// Ethereum classic.
	EthereumClassic,
}

/// `HardwareWalletManager` for devices with no `hardware wallet`
pub struct HardwareWalletManager;

impl HardwareWalletManager {
	pub fn new() -> Result<Self, Error> {
		Err(Error::NoWallet)
	}

	pub fn set_key_path(&self, _key_path: KeyPath) {}

	pub fn wallet_info(&self, _: &Address) -> Option<WalletInfo> { 
		None 
	}

	pub fn list_wallets(&self) -> Vec<WalletInfo> {
		Vec::with_capacity(0)
	}

	pub fn list_locked_wallets(&self) -> Result<Vec<String>, Error> {
		Err(Error::NoWallet)
	}

	pub fn pin_matrix_ack(&self, _: &str, _: &str) -> Result<bool, Error> { 
		Err(Error::NoWallet)
	}
	
	pub fn sign_transaction(&self, _address: &Address, _transaction: &TransactionInfo, _rlp_transaction: &[u8]) -> Result<Signature, Error> { 
		Err(Error::NoWallet) }
	
	pub fn sign_message(&self, _address: &Address, _msg: &[u8]) -> Result<Signature, Error> {
		Err(Error::NoWallet)
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
		write!(f, "No hardware wallet!!") 
	}
}
