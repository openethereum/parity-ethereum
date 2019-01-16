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

//! ParitySigning rpc interface.
use jsonrpc_core::{BoxFuture, Result};

use v1::types::{U256, H160, Bytes, ConfirmationResponse, TransactionRequest, Either};

build_rpc_trait! {
	/// Signing methods implementation.
	pub trait ParitySigning {
		type Metadata;

		/// Given partial transaction request produces transaction with all fields filled in.
		/// Such transaction can be then signed externally.
		#[rpc(meta, name = "parity_composeTransaction")]
		fn compose_transaction(&self, Self::Metadata, TransactionRequest) -> BoxFuture<TransactionRequest>;

		/// Posts sign request asynchronously.
		/// Will return a confirmation ID for later use with check_transaction.
		#[rpc(meta, name = "parity_postSign")]
		fn post_sign(&self, Self::Metadata, H160, Bytes) -> BoxFuture<Either<U256, ConfirmationResponse>>;

		/// Posts transaction asynchronously.
		/// Will return a transaction ID for later use with check_transaction.
		#[rpc(meta, name = "parity_postTransaction")]
		fn post_transaction(&self, Self::Metadata, TransactionRequest) -> BoxFuture<Either<U256, ConfirmationResponse>>;

		/// Checks the progress of a previously posted request (transaction/sign).
		/// Should be given a valid send_transaction ID.
		#[rpc(name = "parity_checkRequest")]
		fn check_request(&self, U256) -> Result<Option<ConfirmationResponse>>;

		/// Decrypt some ECIES-encrypted message.
		/// First parameter is the address with which it is encrypted, second is the ciphertext.
		#[rpc(meta, name = "parity_decryptMessage")]
		fn decrypt_message(&self, Self::Metadata, H160, Bytes) -> BoxFuture<Bytes>;
	}
}
