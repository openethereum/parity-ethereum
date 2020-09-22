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

//! Blockchain test deserializer.

use blockchain::blockchain::BlockChain;
use serde_json::{self, Error};
use std::{collections::BTreeMap, io::Read};

/// Blockchain test deserializer.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Test(BTreeMap<String, BlockChain>);

impl IntoIterator for Test {
    type Item = <BTreeMap<String, BlockChain> as IntoIterator>::Item;
    type IntoIter = <BTreeMap<String, BlockChain> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Test {
    /// Loads test from json.
    pub fn load<R>(reader: R) -> Result<Self, Error>
    where
        R: Read,
    {
        serde_json::from_reader(reader)
    }
}
