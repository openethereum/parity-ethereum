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

//! Privte transaction signing RPC implementation.

use std::sync::Arc;

use rlp::UntrustedRlp;

use ethcore::private_transactions::Provider as PrivateTransactionManager;
use ethcore::transaction::SignedTransaction;

use jsonrpc_core::{Error};
use v1::types::{Bytes, PrivateTransactionReceipt};
use v1::traits::Private;
use v1::helpers::errors;

/// Private transaction manager API endpoint implementation.
pub struct PrivateClient {
	private: Option<Arc<PrivateTransactionManager>>,
}

impl PrivateClient {
	/// Creates a new instance.
	pub fn new(private: &Option<Arc<PrivateTransactionManager>>) -> Self {
		PrivateClient {
			private: private.clone(),
		}
	}

	fn unwrap_manager(&self) -> Result<Arc<PrivateTransactionManager>, Error> {
		match self.private {
			Some(ref arc) => Ok(arc.clone()),
			None => Err(errors::public_unsupported(None)),
		}
	}
}

impl Private for PrivateClient {
	fn send_transaction(&self, request: Bytes) -> Result<PrivateTransactionReceipt, Error> {
		let signed_transaction = UntrustedRlp::new(&request.into_vec()).as_val()
			.map_err(errors::rlp)
			.and_then(|tx| SignedTransaction::new(tx).map_err(errors::transaction))?;
		let client = self.unwrap_manager()?;
		let receipt = client.create_private_transaction(signed_transaction).map_err(errors::transaction)?;
		Ok(receipt.into())
	}
}

