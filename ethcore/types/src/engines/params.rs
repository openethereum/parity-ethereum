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

//! Engine-specific parameter types.

use ethereum_types::{Address, U256, H256};
use bytes::Bytes;
use ethjson;

use BlockNumber;
use engines::DEFAULT_BLOCKHASH_CONTRACT;

const MAX_TRANSACTION_SIZE: usize = 300 * 1024;

/// Parameters common to ethereum-like blockchains.
/// NOTE: when adding bugfix hard-fork parameters,
/// add to `nonzero_bugfix_hard_fork`
///
/// we define a "bugfix" hard fork as any hard fork which
/// you would put on-by-default in a new chain.
#[derive(Debug, PartialEq, Default)]
#[cfg_attr(any(test, feature = "test-helpers"), derive(Clone))]
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
	/// Number of first block where EIP-1283 rules re-enabled.
	pub eip1283_reenable_transition: BlockNumber,
	/// Number of first block where EIP-1014 rules begin.
	pub eip1014_transition: BlockNumber,
	/// Number of first block where EIP-1706 rules begin.
	pub eip1706_transition: BlockNumber,
	/// Number of first block where EIP-1344 rules begin: https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1344.md
	pub eip1344_transition: BlockNumber,
	/// Number of first block where EIP-1884 rules begin:https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1884.md
	pub eip1884_transition: BlockNumber,
	/// Number of first block where EIP-2028 rules begin.
	pub eip2028_transition: BlockNumber,
	/// Number of first block where EIP-2200 advance transition begin.
	pub eip2200_advance_transition: BlockNumber,
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

impl CommonParams {
	/// Schedule for an EVM in the post-EIP-150-era of the Ethereum main net.
	pub fn schedule(&self, block_number: u64) -> vm::Schedule {
		if block_number < self.eip150_transition {
			vm::Schedule::new_homestead()
		} else {
			let max_code_size = self.max_code_size(block_number);
			let mut schedule = vm::Schedule::new_post_eip150(
				max_code_size as _,
				block_number >= self.eip160_transition,
				block_number >= self.eip161abc_transition,
				block_number >= self.eip161d_transition
			);

			self.update_schedule(block_number, &mut schedule);
			schedule
		}
	}

	/// Returns max code size at given block.
	pub fn max_code_size(&self, block_number: u64) -> u64 {
		if block_number >= self.max_code_size_transition {
			self.max_code_size
		} else {
			u64::max_value()
		}
	}

	/// Apply common spec config parameters to the schedule.
	pub fn update_schedule(&self, block_number: u64, schedule: &mut vm::Schedule) {
		schedule.have_create2 = block_number >= self.eip1014_transition;
		schedule.have_revert = block_number >= self.eip140_transition;
		schedule.have_static_call = block_number >= self.eip214_transition;
		schedule.have_return_data = block_number >= self.eip211_transition;
		schedule.have_bitwise_shifting = block_number >= self.eip145_transition;
		schedule.have_extcodehash = block_number >= self.eip1052_transition;
		schedule.have_chain_id = block_number >= self.eip1344_transition;
		schedule.eip1283 =
			(block_number >= self.eip1283_transition &&
			 !(block_number >= self.eip1283_disable_transition)) ||
			block_number >= self.eip1283_reenable_transition;
		schedule.eip1706 = block_number >= self.eip1706_transition;

		if block_number >= self.eip1884_transition {
			schedule.have_selfbalance = true;
			schedule.sload_gas = 800;
			schedule.balance_gas = 700;
			schedule.extcodehash_gas = 700;
		}
		if block_number >= self.eip2028_transition {
			schedule.tx_data_non_zero_gas = 16;
		}
		if block_number >= self.eip2200_advance_transition {
			schedule.sstore_dirty_gas = Some(800);
		}
		if block_number >= self.eip210_transition {
			schedule.blockhash_gas = 800;
		}
		if block_number >= self.dust_protection_transition {
			schedule.kill_dust = match self.remove_dust_contracts {
				true => vm::CleanDustMode::WithCodeAndStorage,
				false => vm::CleanDustMode::BasicOnly,
			};
		}
		if block_number >= self.wasm_activation_transition {
			let mut wasm = vm::WasmCosts::default();
			if block_number >= self.kip4_transition {
				wasm.have_create2 = true;
			}
			if block_number >= self.kip6_transition {
				wasm.have_gasleft = true;
			}
			schedule.wasm = Some(wasm);
			if let Some(version) = self.wasm_version {
				schedule.versions.insert(version, vm::VersionedSchedule::PWasm);
			}
		}
	}

	/// Return Some if the current parameters contain a bugfix hard fork not on block 0.
	pub fn nonzero_bugfix_hard_fork(&self) -> Option<&str> {
		if self.eip155_transition != 0 {
			return Some("eip155Transition");
		}

		if self.validate_receipts_transition != 0 {
			return Some("validateReceiptsTransition");
		}

		if self.validate_chain_id_transition != 0 {
			return Some("validateChainIdTransition");
		}

		None
	}
}

impl From<ethjson::spec::Params> for CommonParams {
	fn from(p: ethjson::spec::Params) -> Self {
		CommonParams {
			account_start_nonce: p.account_start_nonce.map_or_else(U256::zero, Into::into),
			maximum_extra_data_size: p.maximum_extra_data_size.into(),
			network_id: p.network_id.into(),
			chain_id: if let Some(n) = p.chain_id {
				n.into()
			} else {
				p.network_id.into()
			},
			subprotocol_name: p.subprotocol_name.unwrap_or_else(|| "eth".to_owned()),
			min_gas_limit: p.min_gas_limit.into(),
			fork_block: if let (Some(n), Some(h)) = (p.fork_block, p.fork_hash) {
				Some((n.into(), h.into()))
			} else {
				None
			},
			eip150_transition: p.eip150_transition.map_or(0, Into::into),
			eip160_transition: p.eip160_transition.map_or(0, Into::into),
			eip161abc_transition: p.eip161abc_transition.map_or(0, Into::into),
			eip161d_transition: p.eip161d_transition.map_or(0, Into::into),
			eip98_transition: p.eip98_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip155_transition: p.eip155_transition.map_or(0, Into::into),
			validate_receipts_transition: p.validate_receipts_transition.map_or(0, Into::into),
			validate_chain_id_transition: p.validate_chain_id_transition.map_or(0, Into::into),
			eip140_transition: p.eip140_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip210_transition: p.eip210_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip210_contract_address: p.eip210_contract_address.map_or(Address::from_low_u64_be(0xf0), Into::into),
			eip210_contract_code: p.eip210_contract_code.map_or_else(
				|| DEFAULT_BLOCKHASH_CONTRACT.to_vec(),
				Into::into,
			),
			eip210_contract_gas: p.eip210_contract_gas.map_or(1000000.into(), Into::into),
			eip211_transition: p.eip211_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip145_transition: p.eip145_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip214_transition: p.eip214_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip658_transition: p.eip658_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip1052_transition: p.eip1052_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip1283_transition: p.eip1283_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip1283_disable_transition: p.eip1283_disable_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip1283_reenable_transition: p.eip1283_reenable_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip1706_transition: p.eip1706_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip1014_transition: p.eip1014_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip1344_transition: p.eip1344_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip1884_transition: p.eip1884_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip2028_transition: p.eip2028_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			eip2200_advance_transition: p.eip2200_advance_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			dust_protection_transition: p.dust_protection_transition.map_or_else(
				BlockNumber::max_value,
				Into::into,
			),
			nonce_cap_increment: p.nonce_cap_increment.map_or(64, Into::into),
			remove_dust_contracts: p.remove_dust_contracts.unwrap_or(false),
			gas_limit_bound_divisor: p.gas_limit_bound_divisor.into(),
			registrar: p.registrar.map(Into::into),
			node_permission_contract: p.node_permission_contract.map(Into::into),
			max_code_size: p.max_code_size.map_or(u64::max_value(), Into::into),
			max_transaction_size: p.max_transaction_size.map_or(MAX_TRANSACTION_SIZE, Into::into),
			max_code_size_transition: p.max_code_size_transition.map_or(0, Into::into),
			transaction_permission_contract: p.transaction_permission_contract.map(Into::into),
			transaction_permission_contract_transition:
			p.transaction_permission_contract_transition.map_or(0, Into::into),
			wasm_activation_transition: p.wasm_activation_transition.map_or_else(
				BlockNumber::max_value,
				Into::into
			),
			wasm_version: p.wasm_version.map(Into::into),
			kip4_transition: p.kip4_transition.map_or_else(
				BlockNumber::max_value,
				Into::into
			),
			kip6_transition: p.kip6_transition.map_or_else(
				BlockNumber::max_value,
				Into::into
			),
		}
	}
}
