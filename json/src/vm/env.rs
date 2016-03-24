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

//! Vm environment.
use hash::Address;
use uint::Uint;

/// Vm environment.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Env {
	/// Address.
	#[serde(rename="currentCoinbase")]
	pub author: Address,
	/// Difficulty
	#[serde(rename="currentDifficulty")]
	pub difficulty: Uint,
	/// Gas limit.
	#[serde(rename="currentGasLimit")]
	pub gas_limit: Uint,
	/// Number.
	#[serde(rename="currentNumber")]
	pub number: Uint,
	/// Timestamp.
	#[serde(rename="currentTimestamp")]
	pub timestamp: Uint,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use vm::Env;

	#[test]
	fn env_deserialization() {
		let s = r#"{
			"currentCoinbase" : "2adc25665018aa1fe0e6bc666dac8fc2697ff9ba",
			"currentDifficulty" : "0x0100",
			"currentGasLimit" : "0x0f4240",
			"currentNumber" : "0x00",
			"currentTimestamp" : "0x01"
		}"#;
		let _deserialized: Env = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
