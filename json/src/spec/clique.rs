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

//! Clique params deserialization.

use uint::Uint;

/// Tendermint params deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct CliqueParams {
	pub period: Option<Uint>,
	pub epoch: Option<Uint>
}

/// Clique engine deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Clique {
	pub params: CliqueParams,
}

#[cfg(test)]
mod tests {
    use serde_json;
    use ethereum_types::H160;
    use hash::Address;
    use spec::clique::Clique;

    #[test]
    fn clique_deserialization() {
        let s = r#"{
            "params": {
            	"period": 5,
            	"epoch": 30000
            }
        }"#;

        let deserialized: Clique = serde_json::from_str(s).unwrap();
		assert_eq!(deserialized.params.period, 5);
		assert_eq!(deserialized.params.epoch, 30000);
    }
}
