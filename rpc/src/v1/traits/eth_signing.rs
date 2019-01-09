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

//! Eth rpc interface.

use jsonrpc_core::BoxFuture;

use v1::types::{Bytes, H160, H256, H520, TransactionRequest, RichRawTransaction};

build_rpc_trait! {
	/// Signing methods implementation relying on unlocked accounts.
	pub trait EthSigning {
		type Metadata;

		/// Signs the hash of data with given address signature.
		#[rpc(meta, name = "eth_sign")]
		fn sign(&self, Self::Metadata, H160, Bytes) -> BoxFuture<H520>;

		/// Sends transaction; will block waiting for signer to return the
		/// transaction hash.
		/// If Signer is disable it will require the account to be unlocked.
		#[rpc(meta, name = "eth_sendTransaction")]
		fn send_transaction(&self, Self::Metadata, TransactionRequest) -> BoxFuture<H256>;

		/// Signs transactions without dispatching it to the network.
		/// Returns signed transaction RLP representation and the transaction itself.
		/// It can be later submitted using `eth_sendRawTransaction/eth_submitTransaction`.
		#[rpc(meta, name = "eth_signTransaction")]
		fn sign_transaction(&self, Self::Metadata, TransactionRequest) -> BoxFuture<RichRawTransaction>;
	}
}
