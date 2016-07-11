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

use std::str::FromStr;
use std::time::Duration;
use util::journaldb::Algorithm;
use ethcore::client::Mode;

pub fn to_duration(s: &str) -> Result<Duration, String> {
	to_seconds(s).map(Duration::from_secs)
}

fn to_seconds(s: &str) -> Result<u64, String> {
	let bad = |_| {
		format!("{}: Invalid duration given. See parity --help for more information.", s)
	};

	match s {
		"twice-daily" => Ok(12 * 60 * 60),
		"half-hourly" => Ok(30 * 60),
		"1second" | "1 second" | "second" => Ok(1),
		"1minute" | "1 minute" | "minute" => Ok(60),
		"hourly" | "1hour" | "1 hour" | "hour" => Ok(60 * 60),
		"daily" | "1day" | "1 day" | "day" => Ok(24 * 60 * 60),
		x if x.ends_with("seconds") => x[0..x.len() - 7].parse::<u64>().map_err(bad),
		x if x.ends_with("minutes") => x[0..x.len() -7].parse::<u64>().map_err(bad).map(|x| x * 60),
		x if x.ends_with("hours") => x[0..x.len() - 5].parse::<u64>().map_err(bad).map(|x| x * 60 * 60),
		x if x.ends_with("days") => x[0..x.len() - 4].parse::<u64>().map_err(bad).map(|x| x * 24 * 60 * 60),
		x => x.parse::<u64>().map_err(bad),
	}
}

pub fn to_mode(s: &str, timeout: u64, alarm: u64) -> Result<Mode, String> {
	match s {
		"active" => Ok(Mode::Active),
		"passive" => Ok(Mode::Passive(Duration::from_secs(timeout), Duration::from_secs(alarm))),
		"dark" => Ok(Mode::Dark(Duration::from_secs(timeout))),
		_ => Err(format!("{}: Invalid address for --mode. Must be one of active, passive or dark.", s)),
	}
}

pub fn to_pruning(pruning: &str) -> Result<Option<Algorithm>, String> {
	match pruning {
		"auto" => Ok(None),
		specific => Algorithm::from_str(specific).map(Some),
	}
}

#[cfg(test)]
mod tests {
	use std::time::Duration;
	use util::journaldb::Algorithm;
	use ethcore::client::Mode;
	use super::{to_duration, to_mode, to_pruning};

	#[test]
	fn test_to_duration() {
		assert_eq!(to_duration("twice-daily").unwrap(), Duration::from_secs(12 * 60 * 60));
		assert_eq!(to_duration("half-hourly").unwrap(), Duration::from_secs(30 * 60));
		assert_eq!(to_duration("1second").unwrap(), Duration::from_secs(1));
		assert_eq!(to_duration("2seconds").unwrap(), Duration::from_secs(2));
		assert_eq!(to_duration("15seconds").unwrap(), Duration::from_secs(15));
		assert_eq!(to_duration("1minute").unwrap(), Duration::from_secs(1 * 60));
		assert_eq!(to_duration("2minutes").unwrap(), Duration::from_secs(2 * 60));
		assert_eq!(to_duration("15minutes").unwrap(), Duration::from_secs(15 * 60));
		assert_eq!(to_duration("hourly").unwrap(), Duration::from_secs(60 * 60));
		assert_eq!(to_duration("daily").unwrap(), Duration::from_secs(24 * 60 * 60));
		assert_eq!(to_duration("1hour").unwrap(), Duration::from_secs(1 * 60 * 60));
		assert_eq!(to_duration("2hours").unwrap(), Duration::from_secs(2 * 60 * 60));
		assert_eq!(to_duration("15hours").unwrap(), Duration::from_secs(15 * 60 * 60));
		assert_eq!(to_duration("1day").unwrap(), Duration::from_secs(1 * 24 * 60 * 60));
		assert_eq!(to_duration("2days").unwrap(), Duration::from_secs(2 * 24 *60 * 60));
		assert_eq!(to_duration("15days").unwrap(), Duration::from_secs(15 * 24 * 60 * 60));
	}

	#[test]
	fn test_to_mode() {
		assert_eq!(to_mode("active", 0, 0).unwrap(), Mode::Active);
		assert_eq!(to_mode("passive", 10, 20).unwrap(), Mode::Passive(Duration::from_secs(10), Duration::from_secs(20)));
		assert_eq!(to_mode("dark", 20, 30).unwrap(), Mode::Dark(Duration::from_secs(20)));
		assert!(to_mode("other", 20, 30).is_err());
	}

	#[test]
	fn test_to_pruning() {
		assert_eq!(to_pruning("auto").unwrap(), None);
		assert_eq!(to_pruning("archive").unwrap(), Some(Algorithm::Archive));
		assert_eq!(to_pruning("earlymerge").unwrap(), Some(Algorithm::EarlyMerge));
		assert_eq!(to_pruning("overlayrecent").unwrap(), Some(Algorithm::OverlayRecent));
		assert_eq!(to_pruning("refcounted").unwrap(), Some(Algorithm::RefCounted));
		assert!(to_pruning("utop").is_err());
	}
}
