// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! Blockchain test account deserializer.

use bytes::Bytes;
use std::collections::BTreeMap;
use uint::Uint;

/// Blockchain test account deserializer.
#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Account {
    /// Balance.
    pub balance: Uint,
    /// Code.
    pub code: Bytes,
    /// Nonce.
    pub nonce: Uint,
    /// Storage.
    pub storage: BTreeMap<Uint, Uint>,
}

#[cfg(test)]
mod tests {
    use blockchain::account::Account;
    use serde_json;

    #[test]
    fn account_deserialization() {
        let s = r#"{
			"balance" : "0x09184e72a078",
			"code" : "0x600140600155",
			"nonce" : "0x00",
			"storage" : {
				"0x01" : "0x9a10c2b5bb8f3c602e674006d9b21f09167df57c87a78a5ce96d4159ecb76520"
			}
		}"#;
        let _deserialized: Account = serde_json::from_str(s).unwrap();
        // TODO: validate all fields
    }
}
