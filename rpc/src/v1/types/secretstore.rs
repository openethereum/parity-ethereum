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

use v1::types::H512;

/// Sync info
#[derive(Default, Debug, Serialize, PartialEq)]
pub struct EncryptedDocumentKey {
	/// Common encryption point.
	pub common_point: H512,
	/// Ecnrypted point.
	pub encrypted_point: H512,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::EncryptedDocumentKey;

	#[test]
	fn test_serialize_encrypted_document_key() {
		let initial = EncryptedDocumentKey {
			common_point: 1.into(),
			encrypted_point: 2.into(),
		};

		let serialized = serde_json::to_string(&initial).unwrap();
		assert_eq!(serialized, r#"{"common_point":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001","encrypted_point":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002"}"#);
	}
}
