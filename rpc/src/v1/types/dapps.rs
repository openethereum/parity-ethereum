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

/// Local Dapp
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalDapp {
	/// ID of local dapp
	pub id: String,
	/// Dapp name
	pub name: String,
	/// Dapp description
	pub description: String,
	/// Dapp version string
	pub version: String,
	/// Dapp author
	pub author: String,
	/// Dapp icon
	#[serde(rename="iconUrl")]
	pub icon_url: String,
	/// Local development Url
	#[serde(rename="localUrl")]
	pub local_url: Option<String>,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::LocalDapp;

	#[test]
	fn dapp_serialization() {
		let s = r#"{"id":"skeleton","name":"Skeleton","description":"A skeleton dapp","version":"0.1","author":"Parity Technologies Ltd","iconUrl":"title.png","localUrl":"http://localhost:5000"}"#;

		let dapp = LocalDapp {
			id: "skeleton".into(),
			name: "Skeleton".into(),
			description: "A skeleton dapp".into(),
			version: "0.1".into(),
			author: "Parity Technologies Ltd".into(),
			icon_url: "title.png".into(),
			local_url: "http://localhost:5000".into(),
		};

		let serialized = serde_json::to_string(&dapp).unwrap();
		assert_eq!(serialized, s);
	}
}
