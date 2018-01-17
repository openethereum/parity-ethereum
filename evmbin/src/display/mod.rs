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

//! VM Output display utils.

use std::time::Duration;
use ethereum_types::U256;

pub mod json;
pub mod std_json;
pub mod simple;

/// Formats duration into human readable format.
pub fn format_time(time: &Duration) -> String {
	format!("{}.{:.9}s", time.as_secs(), time.subsec_nanos())
}

/// Formats the time as microseconds.
pub fn as_micros(time: &Duration) -> u64 {
	time.as_secs() * 1_000_000 + time.subsec_nanos() as u64 / 1_000
}

/// Converts U256 into string.
/// TODO Overcomes: https://github.com/paritytech/bigint/issues/13
pub fn u256_as_str(v: &U256) -> String {
	if v.is_zero() {
		"\"0x0\"".into()
	} else {
		format!("\"{:x}\"", v)
	}
}
