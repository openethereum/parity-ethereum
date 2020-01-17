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

//! Actions on ancestry blocks when working on a new block.

use ethereum_types::H256;

#[derive(Debug, PartialEq, Eq, Clone)]
/// Actions on a live block's parent block. Only committed when the live block is committed. Those actions here must
/// respect the normal blockchain reorganization rules.
pub enum AncestryAction {
	/// Mark an ancestry block as finalized.
	MarkFinalized(H256),
}
