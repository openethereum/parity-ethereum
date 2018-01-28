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

/// Light pool status.
/// This status is cheap to compute and can be called frequently.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct LightStatus {
	/// Memory usage in bytes.
	pub mem_usage: usize,
	/// Total number of transactions in the pool.
	pub transaction_count: usize,
	/// Number of unique senders in the pool.
	pub senders: usize,
}

/// A full queue status.
/// To compute this status it is required to provide `Ready`.
/// NOTE: To compute the status we need to visit each transaction in the pool.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Status {
	/// Number of stalled transactions.
	pub stalled: usize,
	/// Number of pending (ready) transactions.
	pub pending: usize,
	/// Number of future (not ready) transactions.
	pub future: usize,
}
