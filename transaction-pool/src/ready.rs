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

use {VerifiedTransaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readiness {
	Stalled,
	Ready,
	Future,
}

impl From<bool> for Readiness {
	fn from(b: bool) -> Self {
		if b { Readiness::Ready } else { Readiness::Future }
	}
}

pub trait Ready {
	/// Returns true if transaction is ready to be included in pending block,
	/// given all previous transactions that were ready are included.
	fn is_ready(&mut self, tx: &VerifiedTransaction) -> Readiness;
}

impl<F> Ready for F where F: FnMut(&VerifiedTransaction) -> Readiness {
	fn is_ready(&mut self, tx: &VerifiedTransaction) -> Readiness {
		(*self)(tx)
	}
}
