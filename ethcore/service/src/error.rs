// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use ethcore;
use io;
use ethcore_private_tx;

#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum Error {
	/// Ethcore Error
	Ethcore(ethcore::error::Error),
	/// Io Error
	IoError(io::IoError),
	/// Private Transactions Error
	PrivateTransactions(ethcore_private_tx::Error),
}

impl std::error::Error for Error {
	fn source(&self) -> Option<&(std::error::Error + 'static)> {
		match self {
			Error::Ethcore(err) => Some(err),
			Error::IoError(err) => Some(err),
			Error::PrivateTransactions(err) => Some(err),
		}
	}
}
