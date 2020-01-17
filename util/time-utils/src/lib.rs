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

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Temporary trait for `checked operations` on SystemTime until these are available in the standard library
pub trait CheckedSystemTime {
	/// Returns `Some<SystemTime>` when the result less or equal to `i32::max_value` to prevent `SystemTime` to panic because
	/// it is platform specific, possible representations are i32, i64, u64 or Duration. `None` otherwise
	fn checked_add(self, _d: Duration) -> Option<SystemTime>;
	/// Returns `Some<SystemTime>` when the result is successful and `None` when it is not
	fn checked_sub(self, _d: Duration) -> Option<SystemTime>;
}

impl CheckedSystemTime for SystemTime {
	fn checked_add(self, dur: Duration) -> Option<SystemTime> {
		let this_dur = self.duration_since(UNIX_EPOCH).ok()?;
		let total_time = this_dur.checked_add(dur)?;

		if total_time.as_secs() <= i32::max_value() as u64 {
			Some(self + dur)
		} else {
			None
		}
	}

	fn checked_sub(self, dur: Duration) -> Option<SystemTime> {
		let this_dur = self.duration_since(UNIX_EPOCH).ok()?;
		let total_time = this_dur.checked_sub(dur)?;

		if total_time.as_secs() <= i32::max_value() as u64 {
			Some(self - dur)
		} else {
			None
		}
	}
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
		use super::CheckedSystemTime;
		use std::time::{Duration, SystemTime, UNIX_EPOCH};

		assert!(CheckedSystemTime::checked_add(UNIX_EPOCH, Duration::new(i32::max_value() as u64 + 1, 0)).is_none());
		assert!(CheckedSystemTime::checked_add(UNIX_EPOCH, Duration::new(i32::max_value() as u64, 0)).is_some());
		assert!(CheckedSystemTime::checked_add(UNIX_EPOCH, Duration::new(i32::max_value() as u64 - 1, 1_000_000_000)).is_some());

		assert!(CheckedSystemTime::checked_sub(UNIX_EPOCH, Duration::from_secs(120)).is_none());
		assert!(CheckedSystemTime::checked_sub(SystemTime::now(), Duration::from_secs(1000)).is_some());
	}
}
