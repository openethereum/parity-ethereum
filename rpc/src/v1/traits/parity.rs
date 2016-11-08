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

//! Parity-specific rpc interface.
use jsonrpc_core::Error;

use std::collections::BTreeMap;
use v1::helpers::auto_args::Wrap;
use v1::types::{H160, H256, H512, U256, Bytes, Peers, Transaction, RpcSettings, Histogram};

build_rpc_trait! {
	/// Parity-specific rpc interface.
	pub trait Parity {
		/// Returns current transactions limit.
		#[rpc(name = "parity_transactionsLimit")]
		fn transactions_limit(&self) -> Result<usize, Error>;

		/// Returns mining extra data.
		#[rpc(name = "parity_extraData")]
		fn extra_data(&self) -> Result<Bytes, Error>;

		/// Returns mining gas floor target.
		#[rpc(name = "parity_gasFloorTarget")]
		fn gas_floor_target(&self) -> Result<U256, Error>;

		/// Returns mining gas floor cap.
		#[rpc(name = "parity_gasCeilTarget")]
		fn gas_ceil_target(&self) -> Result<U256, Error>;

		/// Returns minimal gas price for transaction to be included in queue.
		#[rpc(name = "parity_minGasPrice")]
		fn min_gas_price(&self) -> Result<U256, Error>;

		/// Returns latest logs
		#[rpc(name = "parity_devLogs")]
		fn dev_logs(&self) -> Result<Vec<String>, Error>;

		/// Returns logs levels
		#[rpc(name = "parity_devLogsLevels")]
		fn dev_logs_levels(&self) -> Result<String, Error>;

		/// Returns chain name
		#[rpc(name = "parity_netChain")]
		fn net_chain(&self) -> Result<String, Error>;

		/// Returns peers details
		#[rpc(name = "parity_netPeers")]
		fn net_peers(&self) -> Result<Peers, Error>;

		/// Returns network port
		#[rpc(name = "parity_netPort")]
		fn net_port(&self) -> Result<u16, Error>;

		/// Returns rpc settings
		#[rpc(name = "parity_rpcSettings")]
		fn rpc_settings(&self) -> Result<RpcSettings, Error>;

		/// Returns node name
		#[rpc(name = "parity_nodeName")]
		fn node_name(&self) -> Result<String, Error>;

		/// Returns default extra data
		#[rpc(name = "parity_defaultExtraData")]
		fn default_extra_data(&self) -> Result<Bytes, Error>;

		/// Returns distribution of gas price in latest blocks.
		#[rpc(name = "parity_gasPriceHistogram")]
		fn gas_price_histogram(&self) -> Result<Histogram, Error>;

		/// Returns number of unsigned transactions waiting in the signer queue (if signer enabled)
		/// Returns error when signer is disabled
		#[rpc(name = "parity_unsignedTransactionsCount")]
		fn unsigned_transactions_count(&self) -> Result<usize, Error>;

		/// Returns a cryptographically random phrase sufficient for securely seeding a secret key.
		#[rpc(name = "parity_generateSecretPhrase")]
		fn generate_secret_phrase(&self) -> Result<String, Error>;

		/// Returns whatever address would be derived from the given phrase if it were to seed a brainwallet.
		#[rpc(name = "parity_phraseToAddress")]
		fn phrase_to_address(&self, String) -> Result<H160, Error>;

		/// Returns the value of the registrar for this network.
		#[rpc(name = "parity_registryAddress")]
		fn registry_address(&self) -> Result<Option<H160>, Error>;

		/// Returns all addresses if Fat DB is enabled (`--fat-db`), or null if not.
		#[rpc(name = "parity_listAccounts")]
		fn list_accounts(&self) -> Result<Option<Vec<H160>>, Error>;

		/// Returns all storage keys of the given address (first parameter) if Fat DB is enabled (`--fat-db`),
		/// or null if not.
		#[rpc(name = "parity_listStorageKeys")]
		fn list_storage_keys(&self, H160) -> Result<Option<Vec<H256>>, Error>;

		/// Encrypt some data with a public key under ECIES.
		/// First parameter is the 512-byte destination public key, second is the message.
		#[rpc(name = "parity_encryptMessage")]
		fn encrypt_message(&self, H512, Bytes) -> Result<Bytes, Error>;

		/// Returns all pending transactions from transaction queue.
		#[rpc(name = "parity_pendingTransactions")]
		fn pending_transactions(&self) -> Result<Vec<Transaction>, Error>;

		/// Returns current Trusted Signer port or an error if signer is disabled.
		#[rpc(name = "parity_signerPort")]
		fn signer_port(&self) -> Result<u16, Error>;

		/// Returns current Dapps Server port or an error if dapps server is disabled.
		#[rpc(name = "parity_dappsPort")]
		fn dapps_port(&self) -> Result<u16, Error>;

		/// Returns next nonce for particular sender. Should include all transactions in the queue.
		#[rpc(name = "parity_nextNonce")]
		fn next_nonce(&self, H160) -> Result<U256, Error>;

		/// Get the mode. Results one of: "active", "passive", "dark", "offline".
		#[rpc(name = "parity_mode")]
		fn mode(&self) -> Result<String, Error>;

		/// Get the enode of this node.
		#[rpc(name = "parity_enode")]
		fn enode(&self) -> Result<String, Error>;

		/// Returns accounts information.
		#[rpc(name = "parity_accounts")]
		fn accounts(&self) -> Result<BTreeMap<String, BTreeMap<String, String>>, Error>;
	}
}
