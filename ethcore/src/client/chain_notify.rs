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

/// Represents what has to be handled by actor listening to chain events
pub trait ChainNotify {
	fn chain_new_blocks(&self,
		imported: &[H256],
		invalid: &[H256],
		enacted: &[H256],
		retracted: &[H256],
		sealed: &[H256]) {
		// does nothing by default
	}

	fn start(&self) {
		// does nothing by default
	}

	fn stop(&self) {
		// does nothing by default
	}
}

struct EmptyNotifier;

impl ChainNotify for EmptyNotifier { }
