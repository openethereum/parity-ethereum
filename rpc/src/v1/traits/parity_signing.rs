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

//! ParitySigning rpc interface.
use std::sync::Arc;
use jsonrpc_core::*;

/// Signing methods implementation relying on unlocked accounts.
pub trait ParitySigning: Sized + Send + Sync + 'static {
	/// Posts sign request asynchronously.
	/// Will return a confirmation ID for later use with check_transaction.
	fn post_sign(&self, _: Params) -> Result<Value, Error>;

	/// Posts transaction asynchronously.
	/// Will return a transaction ID for later use with check_transaction.
	fn post_transaction(&self, _: Params) -> Result<Value, Error>;

	/// Checks the progress of a previously posted request (transaction/sign).
	/// Should be given a valid send_transaction ID.
	/// Returns the transaction hash, the zero hash (not yet available),
	/// or the signature,
	/// or an error.
	fn check_request(&self, _: Params) -> Result<Value, Error>;

	/// Decrypt some ECIES-encrypted message.
	/// First parameter is the address with which it is encrypted, second is the ciphertext.
	fn decrypt_message(&self, _: Params, _: Ready);

	/// Should be used to convert object to io delegate.
	fn to_delegate(self) -> IoDelegate<Self> {
		let mut delegate = IoDelegate::new(Arc::new(self));
		delegate.add_method("parity_postSign", ParitySigning::post_sign);
		delegate.add_method("parity_postTransaction", ParitySigning::post_transaction);
		delegate.add_method("parity_checkRequest", ParitySigning::check_request);
		delegate.add_async_method("parity_decryptMessage", ParitySigning::decrypt_message);

		delegate
	}
}
