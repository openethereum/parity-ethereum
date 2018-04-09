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

//! SecretStore-specific rpc interface.

use jsonrpc_core::Error;

use v1::types::{Bytes, PrivateTransactionReceipt, H160, H256, U256, BlockNumber,
	PrivateTransactionReceiptAndTransaction, CallRequest};

build_rpc_trait! {
	/// Private transaction management RPC interface.
	pub trait Private {
		type Metadata;

		/// Sends private transaction; Transaction will be added to the validation queue and sent out when ready.
		#[rpc(name = "private_sendTransaction")]
		fn send_transaction(&self, Bytes) -> Result<PrivateTransactionReceipt, Error>;

		/// Creates a transaction for contract's deployment from origin (signed transaction)
		#[rpc(name = "private_composeDeploymentTransaction")]
		fn compose_deployment_transaction(&self, BlockNumber, Bytes, Vec<H160>, U256) -> Result<PrivateTransactionReceiptAndTransaction, Error>;

		/// Make a call to the private contract
		#[rpc(meta, name = "private_call")]
		fn private_call(&self, Self::Metadata, BlockNumber, CallRequest) -> Result<Bytes, Error>;

		/// Retrieve the id of the key associated with the contract
		#[rpc(name = "private_contractKey")]
		fn private_contract_key(&self, H160) -> Result<H256, Error>;
	}
}
