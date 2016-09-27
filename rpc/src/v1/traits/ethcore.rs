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

//! Ethcore-specific rpc interface.
use jsonrpc_core::Error;

use v1::helpers::auto_args::Wrap;
use v1::types::{H160, H512, U256, Bytes, Peers, Transaction, RpcSettings};

build_rpc_trait! {
	/// Ethcore-specific rpc interface.
	pub trait Ethcore {
		/// Returns current transactions limit.
		#[name("ethcore_transactionsLimit")]
		fn transactions_limit(&self) -> Result<usize, Error>;

		/// Returns mining extra data.
		#[name("ethcore_extraData")]
		fn extra_data(&self) -> Result<Bytes, Error>;

		/// Returns mining gas floor target.
		#[name("ethcore_gasFloorTarget")]
		fn gas_floor_target(&self) -> Result<U256, Error>;

		/// Returns mining gas floor cap.
		#[name("ethcore_gasCeilTarget")]
		fn gas_ceil_target(&self) -> Result<U256, Error>;

		/// Returns minimal gas price for transaction to be included in queue.
		#[name("ethcore_minGasPrice")]
		fn min_gas_price(&self) -> Result<U256, Error>;

		/// Returns latest logs
		#[name("ethcore_devLogs")]
		fn dev_logs(&self) -> Result<Vec<String>, Error>;

		/// Returns logs levels
		#[name("ethcore_devLogsLevels")]
		fn dev_logs_levels(&self) -> Result<String, Error>;

		/// Returns chain name
		#[name("ethcore_netChain")]
		fn net_chain(&self) -> Result<String, Error>;

		/// Returns peers details
		#[name("ethcore_netPeers")]
		fn net_peers(&self) -> Result<Peers, Error>;

		/// Returns network port
		#[name("ethcore_netPort")]
		fn net_port(&self) -> Result<u16, Error>;

		/// Returns rpc settings
		#[name("ethcore_rpcSettings")]
		fn rpc_settings(&self) -> Result<RpcSettings, Error>;

		/// Returns node name
		#[name("ethcore_nodeName")]
		fn node_name(&self) -> Result<String, Error>;

		/// Returns default extra data
		#[name("ethcore_defaultExtraData")]
		fn default_extra_data(&self) -> Result<Bytes, Error>;

		/// Returns distribution of gas price in latest blocks.
		#[name("ethcore_gasPriceStatistics")]
		fn gas_price_statistics(&self) -> Result<Vec<U256>, Error>;

		/// Returns number of unsigned transactions waiting in the signer queue (if signer enabled)
		/// Returns error when signer is disabled
		#[name("ethcore_unsignedTransactionsCount")]
		fn unsigned_transactions_count(&self) -> Result<usize, Error>;

		/// Returns a cryptographically random phrase sufficient for securely seeding a secret key.
		#[name("ethcore_generateSecretPhrase")]
		fn generate_secret_phrase(&self) -> Result<String, Error>;

		/// Returns whatever address would be derived from the given phrase if it were to seed a brainwallet.
		#[name("ethcore_phraseToAddress")]
		fn phrase_to_address(&self, String) -> Result<H160, Error>;

		/// Returns the value of the registrar for this network.
		#[name("ethcore_registryAddress")]
		fn registry_address(&self) -> Result<Option<H160>, Error>;

		/// Encrypt some data with a public key under ECIES.
		/// First parameter is the 512-byte destination public key, second is the message.
		#[name("ethcore_encryptMessage")]
		fn encrypt_message(&self, H512, Bytes) -> Result<Bytes, Error>;

		/// Returns all pending transactions from transaction queue.
		#[name("ethcore_pendingTransactions")]
		fn pending_transactions(&self) -> Result<Vec<Transaction>, Error>;
	}
}