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

//! Types used in the public api

pub mod error;
pub mod filter;
pub mod flat;
pub mod trace;
pub mod localized;

use self::flat::FlatTransactionTraces;

/// Container for block traces.
#[derive(Clone)]
pub enum Tracing {
	/// This variant should be used when tracing is enabled.
	Enabled(Vec<FlatTransactionTraces>),
	/// Tracing is disabled.
	Disabled,
}

impl Tracing {
	/// Creates new instance of enabled tracing object.
	pub fn enabled() -> Self {
		Tracing::Enabled(Default::default())
	}

	/// Returns true if tracing is enabled.
	pub fn is_enabled(&self) -> bool {
		match *self {
			Tracing::Enabled(_) => true,
			Tracing::Disabled => false,
		}
	}

	/// Drain all traces.
	pub fn drain(self) -> Vec<FlatTransactionTraces> {
		match self {
			Tracing::Enabled(traces) => traces,
			Tracing::Disabled => vec![],
		}
	}
}
