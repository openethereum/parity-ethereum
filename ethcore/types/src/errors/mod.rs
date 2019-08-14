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

//! General error types for use in parity-ethereum.

mod block_error;
mod engine_error;
mod ethcore_error;
mod snapshot_error;

pub use self::{
	block_error::{BlockError, ImportError},
	engine_error::EngineError,
	ethcore_error::{EthcoreError, ExecutionError, EthcoreResult},
	snapshot_error::SnapshotError,
};
