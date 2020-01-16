// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! A client interface for interacting with the block gas limit contract.

use client_traits::BlockChainClient;
use common_types::{header::Header, ids::BlockId};
use ethabi::FunctionOutputDecoder;
use ethabi_contract::use_contract;
use ethereum_types::{Address, U256};
use log::{debug, error};

use_contract!(contract, "res/block_gas_limit.json");

pub fn block_gas_limit(full_client: &dyn BlockChainClient, header: &Header, address: Address) -> Option<U256> {
	let (data, decoder) = contract::functions::block_gas_limit::call();
	let value = full_client.call_contract(BlockId::Hash(*header.parent_hash()), address, data).map_err(|err| {
		error!(target: "block_gas_limit", "Contract call failed. Not changing the block gas limit. {:?}", err);
	}).ok()?;
	if value.is_empty() {
		debug!(target: "block_gas_limit", "Contract call returned nothing. Not changing the block gas limit.");
		None
	} else {
		decoder.decode(&value).ok()
	}
}
