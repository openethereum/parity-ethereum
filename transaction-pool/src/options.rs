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

/// Transaction Pool options.
#[derive(Clone, Debug, PartialEq)]
pub struct Options {
	/// Maximal number of transactions in the pool.
	pub max_count: usize,
	/// Maximal number of transactions from single sender.
	pub max_per_sender: usize,
	/// Maximal memory usage.
	pub max_mem_usage: usize,
}

impl Default for Options {
	fn default() -> Self {
		Options {
			max_count: 1024,
			max_per_sender: 16,
			max_mem_usage: 8 * 1024 * 1024,
		}
	}
}
