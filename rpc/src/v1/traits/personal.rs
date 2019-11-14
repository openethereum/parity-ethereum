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

//! Personal rpc interface.
use eip_712::EIP712;
use ethereum_types::{H160, H256, H520, U128};
use jsonrpc_core::types::Value;
use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_derive::rpc;
use v1::types::{Bytes, TransactionRequest, RichRawTransaction as RpcRichRawTransaction, EIP191Version};

/// Personal rpc interface. Safe (read-only) functions.
#[rpc(server)]
pub trait Personal {
	/// RPC Metadata
	type Metadata;

	/// Lists all stored accounts
	#[rpc(name = "personal_listAccounts")]
	fn accounts(&self) -> Result<Vec<H160>>;

	/// Creates new account (it becomes new current unlocked account)
	/// Param is the password for the account.
	#[rpc(name = "personal_newAccount")]
	fn new_account(&self, _: String) -> Result<H160>;

	/// Unlocks specified account for use (can only be one unlocked account at one moment)
	#[rpc(name = "personal_unlockAccount")]
	fn unlock_account(&self, _: H160, _: String, _: Option<U128>) -> Result<bool>;

	/// Signs the hash of data with given account signature using the given password to unlock the account during
	/// the request.
	#[rpc(name = "personal_sign")]
	fn sign(&self, _: Bytes, _:  H160, _: String) -> BoxFuture<H520>;

	/// Produces an EIP-712 compliant signature with given account using the given password to unlock the
	/// account during the request.
	#[rpc(name = "personal_signTypedData")]
	fn sign_typed_data(&self, _: EIP712, _: H160, _: String) -> BoxFuture<H520>;

	/// Signs an arbitrary message based on the version specified
	#[rpc(name = "personal_sign191")]
	fn sign_191(&self, _: EIP191Version, _: Value, _: H160, _: String) -> BoxFuture<H520>;

	/// Returns the account associated with the private key that was used to calculate the signature in
	/// `personal_sign`.
	#[rpc(name = "personal_ecRecover")]
	fn ec_recover(&self, _: Bytes, _: H520) -> BoxFuture<H160>;

	/// Signs transaction. The account is not unlocked in such case.
	#[rpc(meta, name = "personal_signTransaction")]
	fn sign_transaction(&self, _: Self::Metadata, _: TransactionRequest, _: String) -> BoxFuture<RpcRichRawTransaction>;

	/// Sends transaction and signs it in single call. The account is not unlocked in such case.
	#[rpc(meta, name = "personal_sendTransaction")]
	fn send_transaction(&self, _: Self::Metadata, _: TransactionRequest, _: String) -> BoxFuture<H256>;

	/// @deprecated alias for `personal_sendTransaction`.
	#[rpc(meta, name = "personal_signAndSendTransaction")]
	fn sign_and_send_transaction(&self, _: Self::Metadata, _: TransactionRequest, _: String) -> BoxFuture<H256>;
}
