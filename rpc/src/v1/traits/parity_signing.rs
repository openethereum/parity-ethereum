// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
use jsonrpc_core::Error;
use futures::BoxFuture;

use v1::types::{U256, H160, H256, Bytes, ConfirmationResponse, TransactionRequest, Either};

build_rpc_trait! {
	/// Signing methods implementation.
	pub trait ParitySigning {
		/// Posts sign request asynchronously.
		/// Will return a confirmation ID for later use with check_transaction.
		#[rpc(name = "parity_postSign")]
		fn post_sign(&self, H160, H256) -> Result<Either<U256, ConfirmationResponse>, Error>;

		/// Posts transaction asynchronously.
		/// Will return a transaction ID for later use with check_transaction.
		#[rpc(name = "parity_postTransaction")]
		fn post_transaction(&self, TransactionRequest) -> Result<Either<U256, ConfirmationResponse>, Error>;

		/// Checks the progress of a previously posted request (transaction/sign).
		/// Should be given a valid send_transaction ID.
		#[rpc(name = "parity_checkRequest")]
		fn check_request(&self, U256) -> Result<Option<ConfirmationResponse>, Error>;

		/// Decrypt some ECIES-encrypted message.
		/// First parameter is the address with which it is encrypted, second is the ciphertext.
		#[rpc(async, name = "parity_decryptMessage")]
		fn decrypt_message(&self, H160, Bytes) -> BoxFuture<Bytes, Error>;
	}
}
