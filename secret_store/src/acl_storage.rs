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

use std::collections::{HashMap, HashSet};
use parking_lot::RwLock;
use types::all::{Error, DocumentAddress, Public};

/// ACL storage of Secret Store
pub trait AclStorage: Send + Sync {
	/// Check if requestor with `public` key can access document with hash `document`
	fn check(&self, public: &Public, document: &DocumentAddress) -> Result<bool, Error>;
}

/// Dummy ACL storage implementation
#[derive(Default, Debug)]
pub struct DummyAclStorage {
	prohibited: RwLock<HashMap<Public, HashSet<DocumentAddress>>>,
}

impl DummyAclStorage {
	#[cfg(test)]
	/// Prohibit given requestor access to given document
	pub fn prohibit(&self, public: Public, document: DocumentAddress) {
		self.prohibited.write()
			.entry(public)
			.or_insert_with(Default::default)
			.insert(document);
	}
}

impl AclStorage for DummyAclStorage {
	fn check(&self, public: &Public, document: &DocumentAddress) -> Result<bool, Error> {
		Ok(self.prohibited.read()
			.get(public)
			.map(|docs| !docs.contains(document))
			.unwrap_or(true))
	}
}
