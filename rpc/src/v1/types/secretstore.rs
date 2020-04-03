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

use ethereum_types::H512;
use v1::types::Bytes;

/// Encrypted document key.
#[derive(Default, Debug, Serialize, PartialEq)]
#[cfg_attr(test, derive(Deserialize))]
pub struct EncryptedDocumentKey {
	/// Common encryption point. Pass this to Secret Store 'Document key storing session'
	pub common_point: H512,
	/// Encrypted point. Pass this to Secret Store 'Document key storing session'.
	pub encrypted_point: H512,
	/// Document key itself, encrypted with passed account public. Pass this to 'secretstore_encrypt'.
	pub encrypted_key: Bytes,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::{EncryptedDocumentKey, H512};

	#[test]
	fn test_serialize_encrypted_document_key() {
		let initial = EncryptedDocumentKey {
			common_point: H512::from_low_u64_be(1),
			encrypted_point: H512::from_low_u64_be(2),
			encrypted_key: vec![3].into(),
		};

		let serialized = serde_json::to_string(&initial).unwrap();
		assert_eq!(serialized, r#"{"common_point":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001","encrypted_point":"0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002","encrypted_key":"0x03"}"#);

		let deserialized: EncryptedDocumentKey = serde_json::from_str(&serialized).unwrap();
		assert_eq!(deserialized.common_point, H512::from_low_u64_be(1));
		assert_eq!(deserialized.encrypted_point, H512::from_low_u64_be(2));
		assert_eq!(deserialized.encrypted_key, vec![3].into());
	}
}
