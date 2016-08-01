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

//! Traces config.
use std::str::FromStr;
use bloomchain::Config as BloomConfig;
use trace::Error;

/// 3-value enum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Switch {
	/// True.
	On,
	/// False.
	Off,
	/// Auto.
	Auto,
}

impl Default for Switch {
	fn default() -> Self {
		Switch::Auto
	}
}

impl FromStr for Switch {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"on" => Ok(Switch::On),
			"off" => Ok(Switch::Off),
			"auto" => Ok(Switch::Auto),
			other => Err(format!("Invalid switch value: {}", other))
		}
	}
}

impl Switch {
	/// Tries to turn old switch to new value.
	pub fn turn_to(&self, to: Switch) -> Result<bool, Error> {
		match (*self, to) {
			(Switch::On, Switch::On) | (Switch::On, Switch::Auto) | (Switch::Auto, Switch::On) => Ok(true),
			(Switch::Off, Switch::On) => Err(Error::ResyncRequired),
			_ => Ok(false),
		}
	}
}

/// Traces config.
#[derive(Debug, PartialEq, Clone)]
pub struct Config {
	/// Indicates if tracing should be enabled or not.
	/// If it's None, it will be automatically configured.
	pub enabled: Switch,
	/// Traces blooms configuration.
	pub blooms: BloomConfig,
	/// Preferef cache-size.
	pub pref_cache_size: usize,
	/// Max cache-size.
	pub max_cache_size: usize,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			enabled: Switch::default(),
			blooms: BloomConfig {
				levels: 3,
				elements_per_index: 16,
			},
			pref_cache_size: 15 * 1024 * 1024,
			max_cache_size: 20 * 1024 * 1024,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::Switch;

	#[test]
	fn test_switch_parsing() {
		assert_eq!(Switch::On, "on".parse().unwrap());
		assert_eq!(Switch::Off, "off".parse().unwrap());
		assert_eq!(Switch::Auto, "auto".parse().unwrap());
	}

	#[test]
	fn test_switch_default() {
		assert_eq!(Switch::default(), Switch::Auto);
	}
}
