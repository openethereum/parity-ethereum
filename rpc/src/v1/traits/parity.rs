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

use jsonrpc_core::{BoxFuture, Result};
use jsonrpc_macros::Trailing;

use node_health::Health;
use v1::types::{
	H160, H256, H512, U256, U64, Bytes, CallRequest,
	Peers, Transaction, RpcSettings, Histogram,
	TransactionStats, LocalTransactionStatus,
	BlockNumber, ConsensusCapability, VersionInfo,
	OperationsInfo, DappId, ChainStatus,
	AccountInfo, HwAccountInfo, RichHeader,
};

build_rpc_trait! {
	/// Parity-specific rpc interface.
	pub trait Parity {
		type Metadata;

		/// Returns accounts information.
		#[rpc(name = "parity_accountsInfo")]
		fn accounts_info(&self, Trailing<DappId>) -> Result<BTreeMap<H160, AccountInfo>>;

		/// Returns hardware accounts information.
		#[rpc(name = "parity_hardwareAccountsInfo")]
		fn hardware_accounts_info(&self) -> Result<BTreeMap<H160, HwAccountInfo>>;

		/// Get a list of paths to locked hardware wallets
		#[rpc(name = "parity_lockedHardwareAccountsInfo")]
		fn locked_hardware_accounts_info(&self) -> Result<Vec<String>>;

		/// Returns default account for dapp.
		#[rpc(meta, name = "parity_defaultAccount")]
		fn default_account(&self, Self::Metadata) -> Result<H160>;

		/// Returns current transactions limit.
		#[rpc(name = "parity_transactionsLimit")]
		fn transactions_limit(&self) -> Result<usize>;

		/// Returns mining extra data.
		#[rpc(name = "parity_extraData")]
		fn extra_data(&self) -> Result<Bytes>;

		/// Returns mining gas floor target.
		#[rpc(name = "parity_gasFloorTarget")]
		fn gas_floor_target(&self) -> Result<U256>;

		/// Returns mining gas floor cap.
		#[rpc(name = "parity_gasCeilTarget")]
		fn gas_ceil_target(&self) -> Result<U256>;

		/// Returns minimal gas price for transaction to be included in queue.
		#[rpc(name = "parity_minGasPrice")]
		fn min_gas_price(&self) -> Result<U256>;

		/// Returns latest logs
		#[rpc(name = "parity_devLogs")]
		fn dev_logs(&self) -> Result<Vec<String>>;

		/// Returns logs levels
		#[rpc(name = "parity_devLogsLevels")]
		fn dev_logs_levels(&self) -> Result<String>;

		/// Returns chain name - DEPRECATED. Use `parity_chainName` instead.
		#[rpc(name = "parity_netChain")]
		fn net_chain(&self) -> Result<String>;

		/// Returns peers details
		#[rpc(name = "parity_netPeers")]
		fn net_peers(&self) -> Result<Peers>;

		/// Returns network port
		#[rpc(name = "parity_netPort")]
		fn net_port(&self) -> Result<u16>;

		/// Returns rpc settings
		#[rpc(name = "parity_rpcSettings")]
		fn rpc_settings(&self) -> Result<RpcSettings>;

		/// Returns node name
		#[rpc(name = "parity_nodeName")]
		fn node_name(&self) -> Result<String>;

		/// Returns default extra data
		#[rpc(name = "parity_defaultExtraData")]
		fn default_extra_data(&self) -> Result<Bytes>;

		/// Returns distribution of gas price in latest blocks.
		#[rpc(name = "parity_gasPriceHistogram")]
		fn gas_price_histogram(&self) -> BoxFuture<Histogram>;

		/// Returns number of unsigned transactions waiting in the signer queue (if signer enabled)
		/// Returns error when signer is disabled
		#[rpc(name = "parity_unsignedTransactionsCount")]
		fn unsigned_transactions_count(&self) -> Result<usize>;

		/// Returns a cryptographically random phrase sufficient for securely seeding a secret key.
		#[rpc(name = "parity_generateSecretPhrase")]
		fn generate_secret_phrase(&self) -> Result<String>;

		/// Returns whatever address would be derived from the given phrase if it were to seed a brainwallet.
		#[rpc(name = "parity_phraseToAddress")]
		fn phrase_to_address(&self, String) -> Result<H160>;

		/// Returns the value of the registrar for this network.
		#[rpc(name = "parity_registryAddress")]
		fn registry_address(&self) -> Result<Option<H160>>;

		/// Returns all addresses if Fat DB is enabled (`--fat-db`), or null if not.
		#[rpc(name = "parity_listAccounts")]
		fn list_accounts(&self, u64, Option<H160>, Trailing<BlockNumber>) -> Result<Option<Vec<H160>>>;

		/// Returns all storage keys of the given address (first parameter) if Fat DB is enabled (`--fat-db`),
		/// or null if not.
		#[rpc(name = "parity_listStorageKeys")]
		fn list_storage_keys(&self, H160, u64, Option<H256>, Trailing<BlockNumber>) -> Result<Option<Vec<H256>>>;

		/// Encrypt some data with a public key under ECIES.
		/// First parameter is the 512-byte destination public key, second is the message.
		#[rpc(name = "parity_encryptMessage")]
		fn encrypt_message(&self, H512, Bytes) -> Result<Bytes>;

		/// Returns all pending transactions from transaction queue.
		#[rpc(name = "parity_pendingTransactions")]
		fn pending_transactions(&self) -> Result<Vec<Transaction>>;

		/// Returns all transactions from transaction queue.
		///
		/// Some of them might not be ready to be included in a block yet.
		#[rpc(name = "parity_allTransactions")]
		fn all_transactions(&self) -> Result<Vec<Transaction>>;

		/// Returns all future transactions from transaction queue (deprecated)
		#[rpc(name = "parity_futureTransactions")]
		fn future_transactions(&self) -> Result<Vec<Transaction>>;

		/// Returns propagation statistics on transactions pending in the queue.
		#[rpc(name = "parity_pendingTransactionsStats")]
		fn pending_transactions_stats(&self) -> Result<BTreeMap<H256, TransactionStats>>;

		/// Returns a list of current and past local transactions with status details.
		#[rpc(name = "parity_localTransactions")]
		fn local_transactions(&self) -> Result<BTreeMap<H256, LocalTransactionStatus>>;

		/// Returns current Dapps Server interface and port or an error if dapps server is disabled.
		#[rpc(name = "parity_dappsUrl")]
		fn dapps_url(&self) -> Result<String>;

		/// Returns current WS Server interface and port or an error if ws server is disabled.
		#[rpc(name = "parity_wsUrl")]
		fn ws_url(&self) -> Result<String>;

		/// Returns next nonce for particular sender. Should include all transactions in the queue.
		#[rpc(name = "parity_nextNonce")]
		fn next_nonce(&self, H160) -> BoxFuture<U256>;

		/// Get the mode. Returns one of: "active", "passive", "dark", "offline".
		#[rpc(name = "parity_mode")]
		fn mode(&self) -> Result<String>;

		/// Returns the chain ID used for transaction signing at the
		/// current best block. An empty string is returned if not
		/// available.
		#[rpc(name = "parity_chainId")]
		fn chain_id(&self) -> Result<Option<U64>>;

		/// Get the chain name. Returns one of: "foundation", "kovan", &c. of a filename.
		#[rpc(name = "parity_chain")]
		fn chain(&self) -> Result<String>;

		/// Get the enode of this node.
		#[rpc(name = "parity_enode")]
		fn enode(&self) -> Result<String>;

		/// Returns information on current consensus capability.
		#[rpc(name = "parity_consensusCapability")]
		fn consensus_capability(&self) -> Result<ConsensusCapability>;

		/// Get our version information in a nice object.
		#[rpc(name = "parity_versionInfo")]
		fn version_info(&self) -> Result<VersionInfo>;

		/// Get information concerning the latest releases if available.
		#[rpc(name = "parity_releasesInfo")]
		fn releases_info(&self) -> Result<Option<OperationsInfo>>;

		/// Get the current chain status.
		#[rpc(name = "parity_chainStatus")]
		fn chain_status(&self) -> Result<ChainStatus>;

		/// Get node kind info.
		#[rpc(name = "parity_nodeKind")]
		fn node_kind(&self) -> Result<::v1::types::NodeKind>;

		/// Get block header.
		/// Same as `eth_getBlockByNumber` but without uncles and transactions.
		#[rpc(name = "parity_getBlockHeaderByNumber")]
		fn block_header(&self, Trailing<BlockNumber>) -> BoxFuture<RichHeader>;

		/// Get IPFS CIDv0 given protobuf encoded bytes.
		#[rpc(name = "parity_cidV0")]
		fn ipfs_cid(&self, Bytes) -> Result<String>;

		/// Call contract, returning the output data.
		#[rpc(meta, name = "parity_call")]
		fn call(&self, Self::Metadata, Vec<CallRequest>, Trailing<BlockNumber>) -> Result<Vec<Bytes>>;

		/// Returns node's health report.
		#[rpc(name = "parity_nodeHealth")]
		fn node_health(&self) -> BoxFuture<Health>;
	}
}
