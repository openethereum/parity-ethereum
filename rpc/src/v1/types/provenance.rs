// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of Open Ethereum.

// Open Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Open Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Open Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Request Provenance

use std::fmt;
use ethereum_types::H256;

/// RPC request origin
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub enum Origin {
	/// RPC server (includes request origin)
	Rpc(String),
	/// IPC server (includes session hash)
	Ipc(H256),
	/// WS server
	Ws {
		/// Session id
		session: H256,
	},
	/// Signer (authorized WS server)
	Signer {
		/// Session id
		session: H256
	},
	/// From the C API
	CApi,
	/// Unknown
	Unknown,
}

impl Default for Origin {
	fn default() -> Self {
		Origin::Unknown
	}
}

impl fmt::Display for Origin {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Origin::Rpc(ref origin) => write!(f, "{} via RPC", origin),
			Origin::Ipc(ref session) => write!(f, "IPC (session: {})", session),
			Origin::Ws { ref session } => write!(f, "WebSocket (session: {})", session),
			Origin::Signer { ref session } => write!(f, "Secure Session (session: {})", session),
			Origin::CApi => write!(f, "C API"),
			Origin::Unknown => write!(f, "unknown origin"),
		}
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::Origin;
	use ethereum_types::H256;

	#[test]
	fn should_serialize_origin() {
		// given
		let o1 = Origin::Rpc("test service".into());
		let o3 = Origin::Ipc(H256::from_low_u64_be(5));
		let o4 = Origin::Signer {
			session: H256::from_low_u64_be(10),
		};
		let o5 = Origin::Unknown;
		let o6 = Origin::Ws {
			session: H256::from_low_u64_be(5),
		};

		// when
		let res1 = serde_json::to_string(&o1).unwrap();
		let res3 = serde_json::to_string(&o3).unwrap();
		let res4 = serde_json::to_string(&o4).unwrap();
		let res5 = serde_json::to_string(&o5).unwrap();
		let res6 = serde_json::to_string(&o6).unwrap();

		// then
		assert_eq!(res1, r#"{"rpc":"test service"}"#);
		assert_eq!(res3, r#"{"ipc":"0x0000000000000000000000000000000000000000000000000000000000000005"}"#);
		assert_eq!(res4, r#"{"signer":{"session":"0x000000000000000000000000000000000000000000000000000000000000000a"}}"#);
		assert_eq!(res5, r#""unknown""#);
		assert_eq!(res6, r#"{"ws":{"session":"0x0000000000000000000000000000000000000000000000000000000000000005"}}"#);
	}
}
