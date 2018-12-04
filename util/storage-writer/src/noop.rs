// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

use std::collections::HashMap;
use std::io;

use super::StorageWriter;
use ethereum_types::{Address, H256};

#[derive(Clone)]
pub struct NoopStorageWriter;

impl NoopStorageWriter {
    pub fn new() -> NoopStorageWriter {
        NoopStorageWriter
    }
}

impl StorageWriter for NoopStorageWriter {
    fn boxed_clone(&self) -> Box<StorageWriter> {
        Box::new(NoopStorageWriter)
    }

    fn enabled(&self) -> bool {
        false
    }

    fn write_storage_diffs(&mut self, _header_hash: H256, _header_number: u64,  _dirty_accounts: HashMap<Address, HashMap<H256, H256>>) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{NoopStorageWriter, StorageWriter};

    #[test]
    fn test_not_enabled() {
        let storage_writer = NoopStorageWriter::new();

        assert!(!storage_writer.enabled())
    }
}