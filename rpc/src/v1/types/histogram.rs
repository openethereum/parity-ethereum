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

//! Gas prices histogram.

use ethereum_types::U256;

/// Values of RPC settings.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Histogram {
	/// Gas prices for bucket edges.
	pub bucket_bounds: Vec<U256>,
	/// Transacion counts for each bucket.
	pub counts: Vec<usize>,
}

impl From<::stats::Histogram<::ethereum_types::U256>> for Histogram {
	fn from(h: ::stats::Histogram<::ethereum_types::U256>) -> Self {
		Histogram {
			bucket_bounds: h.bucket_bounds.into_iter().map(Into::into).collect(),
			counts: h.counts
		}
	}
}
