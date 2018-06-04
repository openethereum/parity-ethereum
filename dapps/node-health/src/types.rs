// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! Base health types.

/// Health API endpoint status.
#[derive(Debug, PartialEq, Serialize)]
pub enum HealthStatus {
	/// Everything's OK.
	#[serde(rename = "ok")]
	Ok,
	/// Node health need attention
	/// (the issue is not critical, but may need investigation)
	#[serde(rename = "needsAttention")]
	NeedsAttention,
	/// There is something bad detected with the node.
	#[serde(rename = "bad")]
	Bad,
}

/// Represents a single check in node health.
/// Cointains the status of that check and apropriate message and details.
#[derive(Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HealthInfo<T> {
	/// Check status.
	pub status: HealthStatus,
	/// Human-readable message.
	pub message: String,
	/// Technical details of the check.
	pub details: T,
}

/// Node Health status.
#[derive(Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Health {
	/// Status of peers.
	pub peers: HealthInfo<(usize, usize)>,
	/// Sync status.
	pub sync: HealthInfo<bool>,
	/// Time diff info.
	pub time: HealthInfo<i64>,
}
