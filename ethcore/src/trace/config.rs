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
use bloomchain::Config as BloomConfig;

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

impl Switch {
	/// Tries to turn old switch to new value.
	pub fn turn_to(&self, to: Switch) -> Result<Switch, &'static str> {
		match (*self, to) {
			(Switch::On, Switch::On) => Ok(Switch::On),
			(Switch::On, Switch::Auto) => Ok(Switch::On),
			(Switch::On, Switch::Off) => Ok(Switch::Off),
			(Switch::Off, Switch::On) => Err("Tracing can't be enabled"),
			(Switch::Off, Switch::Auto) => Ok(Switch::Off),
			(Switch::Off, Switch::Off) => Ok(Switch::Off),
			(Switch::Auto, Switch::On) => Ok(Switch::On),
			_ => Ok(Switch::Off),
		}
	}

	/// Returns switch boolean switch value.
	pub fn as_bool(&self) -> bool {
		match *self {
			Switch::On => true,
			Switch::Off | Switch::Auto => false,
		}
	}
}

/// Traces config.
#[derive(Debug, Clone)]
pub struct Config {
	/// Indicates if tracing should be enabled or not.
	/// If it's None, it will be automatically configured.
	pub enabled: Switch,
	/// Traces blooms configuration.
	pub blooms: BloomConfig,
}

impl Default for Config {
	fn default() -> Self {
		Config {
			enabled: Switch::Auto,
			blooms: BloomConfig {
				levels: 3,
				elements_per_index: 16,
			}
		}
	}
}
