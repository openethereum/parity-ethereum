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

//! Trace filter deserialization.

use util::Address;
use ethcore::client::BlockId;
use ethcore::client;
use super::BlockNumber;

#[derive(Debug, Deserialize)]
pub struct TraceFilter {
	#[serde(rename="fromBlock")]
	pub from_block: Option<BlockNumber>,
	#[serde(rename="toBlock")]
	pub to_block: Option<BlockNumber>,
	#[serde(rename="fromAddress")]
	pub from_address: Option<Vec<Address>>,
	#[serde(rename="toAddress")]
	pub to_address: Option<Vec<Address>>,
}

impl Into<client::TraceFilter> for TraceFilter {
	fn into(self) -> client::TraceFilter {
		let start = self.from_block.map_or(BlockId::Latest, Into::into);
		let end = self.to_block.map_or(BlockId::Latest, Into::into);
		client::TraceFilter {
			range: start..end,
			from_address: self.from_address.unwrap_or_else(Vec::new),
			to_address: self.to_address.unwrap_or_else(Vec::new),
		}
	}
}
