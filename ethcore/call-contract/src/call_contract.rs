// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

//! Provides CallContract trait

use bytes::Bytes;
use ethereum_types::Address;
use types::ids::BlockId;

/// Provides `call_contract` method
pub trait CallContract {
	/// Like `call`, but with various defaults. Designed to be used for calling contracts.
	fn call_contract(
		&self,
		block_id: BlockId,
		address: Address,
		data: Bytes
	) -> Result<Bytes, String>;
}
