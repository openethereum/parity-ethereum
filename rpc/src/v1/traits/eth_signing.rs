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

//! Eth rpc interface.

use v1::helpers::auto_args::{WrapAsync, Ready};
use v1::types::{H160, H256, H520, TransactionRequest, RichRawTransaction};

build_rpc_trait! {
	/// Signing methods implementation relying on unlocked accounts.
	pub trait EthSigning {
		/// Signs the data with given address signature.
		#[rpc(async, name = "eth_sign")]
		fn sign(&self, Ready<H520>, H160, H256);

		/// Sends transaction; will block waiting for signer to return the
		/// transaction hash.
		/// If Signer is disable it will require the account to be unlocked.
		#[rpc(async, name = "eth_sendTransaction")]
		fn send_transaction(&self, Ready<H256>, TransactionRequest);

		/// Signs transactions without dispatching it to the network.
		/// Returns signed transaction RLP representation and the transaction itself.
		/// It can be later submitted using `eth_sendRawTransaction/eth_submitTransaction`.
		#[rpc(async, name = "eth_signTransaction")]
		fn sign_transaction(&self, Ready<RichRawTransaction>, TransactionRequest);
	}
}
