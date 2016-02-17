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

use util::hash::*;
use util::uint::*;
use ethcore::log_entry::LocalizedLogEntry;
use v1::types::Bytes;

#[derive(Debug, Serialize)]
pub struct Log {
	address: Address,
	topics: Vec<H256>,
	data: Bytes,
	#[serde(rename="blockHash")]
	block_hash: H256,
	#[serde(rename="blockNumber")]
	block_number: U256,
	#[serde(rename="transactionHash")]
	transaction_hash: H256,
	#[serde(rename="transactionIndex")]
	transaction_index: U256,
	#[serde(rename="logIndex")]
	log_index: U256
}

impl From<LocalizedLogEntry> for Log {
	fn from(e: LocalizedLogEntry) -> Log {
		Log {
			address: e.entry.address,
			topics: e.entry.topics,
			data: Bytes::new(e.entry.data),
			block_hash: e.block_hash,
			block_number: From::from(e.block_number),
			transaction_hash: e.transaction_hash,
			transaction_index: From::from(e.transaction_index),
			log_index: From::from(e.log_index)
		}
	}
}
