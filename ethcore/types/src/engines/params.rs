use ethereum_types::{Address, U256, H256};
use bytes::Bytes;

use BlockNumber;

/// Parameters common to ethereum-like blockchains.
/// NOTE: when adding bugfix hard-fork parameters,
/// add to `nonzero_bugfix_hard_fork`
///
/// we define a "bugfix" hard fork as any hard fork which
/// you would put on-by-default in a new chain.
#[derive(Debug, PartialEq, Default)]
#[cfg_attr(test, derive(Clone))] // todo: this will not work across crate boundaries I think
pub struct CommonParams {
	/// Account start nonce.
	pub account_start_nonce: U256,
	/// Maximum size of extra data.
	pub maximum_extra_data_size: usize,
	/// Network id.
	pub network_id: u64,
	/// Chain id.
	pub chain_id: u64,
	/// Main subprotocol name.
	pub subprotocol_name: String,
	/// Minimum gas limit.
	pub min_gas_limit: U256,
	/// Fork block to check.
	pub fork_block: Option<(BlockNumber, H256)>,
	/// EIP150 transition block number.
	pub eip150_transition: BlockNumber,
	/// Number of first block where EIP-160 rules begin.
	pub eip160_transition: BlockNumber,
	/// Number of first block where EIP-161.abc begin.
	pub eip161abc_transition: BlockNumber,
	/// Number of first block where EIP-161.d begins.
	pub eip161d_transition: BlockNumber,
	/// Number of first block where EIP-98 rules begin.
	pub eip98_transition: BlockNumber,
	/// Number of first block where EIP-658 rules begin.
	pub eip658_transition: BlockNumber,
	/// Number of first block where EIP-155 rules begin.
	pub eip155_transition: BlockNumber,
	/// Validate block receipts root.
	pub validate_receipts_transition: BlockNumber,
	/// Validate transaction chain id.
	pub validate_chain_id_transition: BlockNumber,
	/// Number of first block where EIP-140 rules begin.
	pub eip140_transition: BlockNumber,
	/// Number of first block where EIP-210 rules begin.
	pub eip210_transition: BlockNumber,
	/// EIP-210 Blockhash contract address.
	pub eip210_contract_address: Address,
	/// EIP-210 Blockhash contract code.
	pub eip210_contract_code: Bytes,
	/// Gas allocated for EIP-210 blockhash update.
	pub eip210_contract_gas: U256,
	/// Number of first block where EIP-211 rules begin.
	pub eip211_transition: BlockNumber,
	/// Number of first block where EIP-214 rules begin.
	pub eip214_transition: BlockNumber,
	/// Number of first block where EIP-145 rules begin.
	pub eip145_transition: BlockNumber,
	/// Number of first block where EIP-1052 rules begin.
	pub eip1052_transition: BlockNumber,
	/// Number of first block where EIP-1283 rules begin.
	pub eip1283_transition: BlockNumber,
	/// Number of first block where EIP-1283 rules end.
	pub eip1283_disable_transition: BlockNumber,
	/// Number of first block where EIP-1014 rules begin.
	pub eip1014_transition: BlockNumber,
	/// Number of first block where dust cleanup rules (EIP-168 and EIP169) begin.
	pub dust_protection_transition: BlockNumber,
	/// Nonce cap increase per block. Nonce cap is only checked if dust protection is enabled.
	pub nonce_cap_increment: u64,
	/// Enable dust cleanup for contracts.
	pub remove_dust_contracts: bool,
	/// Wasm activation blocknumber, if any disabled initially.
	pub wasm_activation_transition: BlockNumber,
	/// Wasm account version, activated after `wasm_activation_transition`. If this field is defined, do not use code
	/// prefix to determine VM to execute.
	pub wasm_version: Option<U256>,
	/// Number of first block where KIP-4 rules begin. Only has effect if Wasm is activated.
	pub kip4_transition: BlockNumber,
	/// Number of first block where KIP-6 rules begin. Only has effect if Wasm is activated.
	pub kip6_transition: BlockNumber,
	/// Gas limit bound divisor (how much gas limit can change per block)
	pub gas_limit_bound_divisor: U256,
	/// Registrar contract address.
	pub registrar: Option<Address>,
	/// Node permission managing contract address.
	pub node_permission_contract: Option<Address>,
	/// Maximum contract code size that can be deployed.
	pub max_code_size: u64,
	/// Number of first block where max code size limit is active.
	pub max_code_size_transition: BlockNumber,
	/// Transaction permission managing contract address.
	pub transaction_permission_contract: Option<Address>,
	/// Block at which the transaction permission contract should start being used.
	pub transaction_permission_contract_transition: BlockNumber,
	/// Maximum size of transaction's RLP payload
	pub max_transaction_size: usize,
}
