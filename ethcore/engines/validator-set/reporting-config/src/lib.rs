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

/// Reporting config
#[derive(PartialEq, Debug, Clone)]
pub enum ReportingConfig {
	Force,
	Call(u64),
	Disable
}

impl ReportingConfig {
	pub fn from_str_and_call_every(config_str: Option<String>, call_every: u64) -> Result<Self, String> {
		if let Some(s) = config_str {
			let s = s.as_str();
			match s {
				"force" => Ok(ReportingConfig::Force),
				"call" => Ok(ReportingConfig::Call(call_every)),
				"disable" => Ok(ReportingConfig::Disable),
				other => Err(format!("Invalid reporting config value: {}", other))
			}
		} else {
			Ok(Default::default())
		}
	}
}

impl Default for ReportingConfig {
	fn default() -> Self {
		ReportingConfig::Force
	}
}
