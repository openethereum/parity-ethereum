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

//! Block import error related types

use error::{ImportError, BlockError, Error};
use std::convert::From;

/// Error dedicated to import block function
#[derive(Debug)]
#[cfg_attr(feature = "ipc", binary)]
pub enum BlockImportError {
	/// Import error
	Import(ImportError),
	/// Block error
	Block(BlockError),
	/// Other error
	Other(String),
}

impl From<Error> for BlockImportError {
	fn from(e: Error) -> Self {
		match e {
			Error::Block(block_error) => BlockImportError::Block(block_error),
			Error::Import(import_error) => BlockImportError::Import(import_error),
			_ => BlockImportError::Other(format!("other block import error: {:?}", e)),
		}
	}
}
