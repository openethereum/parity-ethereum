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

//! Dapp Id type

/// Dapplication Internal Id
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct DappId(pub String);

impl Into<String> for DappId {
	fn into(self) -> String {
		self.0
	}
}

impl From<String> for DappId {
	fn from(s: String) -> Self {
		DappId(s)
	}
}

#[cfg(test)]
mod tests {

	use serde_json;
	use super::DappId;

	#[test]
	fn should_serialize_dapp_id() {
		// given
		let id = DappId("testapp".into());

		// when
		let res = serde_json::to_string(&id).unwrap();

		// then
		assert_eq!(res, r#""testapp""#);
	}

	#[test]
	fn should_deserialize_dapp_id() {
		// given
		let id = r#""testapp""#;

		// when
		let res: DappId = serde_json::from_str(id).unwrap();

		// then
		assert_eq!(res, DappId("testapp".into()));
	}


}
