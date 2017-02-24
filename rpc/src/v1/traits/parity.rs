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

//! Parity-specific rpc interface.

use std::collections::BTreeMap;

use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;
use futures::BoxFuture;

use v1::types::{
	H160, H256, H512, U256, Bytes,
	Peers, Transaction, RpcSettings, Histogram,
	TransactionStats, LocalTransactionStatus,
	BlockNumber, ConsensusCapability, VersionInfo,
	OperationsInfo, DappId, ChainStatus,
	AccountInfo, HwAccountInfo,
};

build_rpc_trait! {
	/// Parity-specific rpc interface.
	pub trait Parity {
		type Metadata;

		/// Returns accounts information.
		#[rpc(name = "parity_accountsInfo")]
		fn accounts_info(&self, Trailing<DappId>) -> Result<BTreeMap<H160, AccountInfo>, Error>;

		/// Returns hardware accounts information.
		#[rpc(name = "parity_hardwareAccountsInfo")]
		fn hardware_accounts_info(&self) -> Result<BTreeMap<H160, HwAccountInfo>, Error>;

		/// Returns default account for dapp.
		#[rpc(meta, name = "parity_defaultAccount")]
		fn default_account(&self, Self::Metadata) -> BoxFuture<H160, Error>;

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
		#[rpc(async, name = "parity_gasPriceHistogram")]
		fn gas_price_histogram(&self) -> BoxFuture<Histogram, Error>;

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
		fn list_accounts(&self, u64, Option<H160>, Trailing<BlockNumber>) -> Result<Option<Vec<H160>>, Error>;

		/// Returns all storage keys of the given address (first parameter) if Fat DB is enabled (`--fat-db`),
		/// or null if not.
		#[rpc(name = "parity_listStorageKeys")]
		fn list_storage_keys(&self, H160, u64, Option<H256>, Trailing<BlockNumber>) -> Result<Option<Vec<H256>>, Error>;

		/// Encrypt some data with a public key under ECIES.
		/// First parameter is the 512-byte destination public key, second is the message.
		#[rpc(name = "parity_encryptMessage")]
		fn encrypt_message(&self, H512, Bytes) -> Result<Bytes, Error>;

		/// Returns all pending transactions from transaction queue.
		#[rpc(name = "parity_pendingTransactions")]
		fn pending_transactions(&self) -> Result<Vec<Transaction>, Error>;

		/// Returns all future transactions from transaction queue.
		#[rpc(name = "parity_futureTransactions")]
		fn future_transactions(&self) -> Result<Vec<Transaction>, Error>;

		/// Returns propagation statistics on transactions pending in the queue.
		#[rpc(name = "parity_pendingTransactionsStats")]
		fn pending_transactions_stats(&self) -> Result<BTreeMap<H256, TransactionStats>, Error>;

		/// Returns a list of current and past local transactions with status details.
		#[rpc(name = "parity_localTransactions")]
		fn local_transactions(&self) -> Result<BTreeMap<H256, LocalTransactionStatus>, Error>;

		/// Returns current Trusted Signer port or an error if signer is disabled.
		#[rpc(name = "parity_signerPort")]
		fn signer_port(&self) -> Result<u16, Error>;

		/// Returns current Dapps Server port or an error if dapps server is disabled.
		#[rpc(name = "parity_dappsPort")]
		fn dapps_port(&self) -> Result<u16, Error>;

		/// Returns current Dapps Server interface address or an error if dapps server is disabled.
		#[rpc(name = "parity_dappsInterface")]
		fn dapps_interface(&self) -> Result<String, Error>;

		/// Returns next nonce for particular sender. Should include all transactions in the queue.
		#[rpc(async, name = "parity_nextNonce")]
		fn next_nonce(&self, H160) -> BoxFuture<U256, Error>;

		/// Get the mode. Results one of: "active", "passive", "dark", "offline".
		#[rpc(name = "parity_mode")]
		fn mode(&self) -> Result<String, Error>;

		/// Get the enode of this node.
		#[rpc(name = "parity_enode")]
		fn enode(&self) -> Result<String, Error>;

		/// Returns information on current consensus capability.
		#[rpc(name = "parity_consensusCapability")]
		fn consensus_capability(&self) -> Result<ConsensusCapability, Error>;

		/// Get our version information in a nice object.
		#[rpc(name = "parity_versionInfo")]
		fn version_info(&self) -> Result<VersionInfo, Error>;

		/// Get information concerning the latest releases if available.
		#[rpc(name = "parity_releasesInfo")]
		fn releases_info(&self) -> Result<Option<OperationsInfo>, Error>;

		/// Get the current chain status.
		#[rpc(name = "parity_chainStatus")]
		fn chain_status(&self) -> Result<ChainStatus, Error>;
	}
}
