// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

use ipc::IpcConfig;
use util::H256;

/// Represents what has to be handled by actor listening to chain events
#[ipc]
pub trait ChainNotify : Send + Sync {
	/// fires when chain has new blocks.
	fn new_blocks(&self,
		_imported: Vec<H256>,
		_invalid: Vec<H256>,
		_enacted: Vec<H256>,
		_retracted: Vec<H256>,
		_sealed: Vec<H256>,
		_duration: u64) {
		// does nothing by default
	}

	/// fires when chain achieves active mode
	fn start(&self) {
		// does nothing by default
	}

	/// fires when chain achieves passive mode
	fn stop(&self) {
		// does nothing by default
	}
}

impl IpcConfig for ChainNotify { }
