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

//! Ethcore basic typenames.

use util::hash::H2048;

/// Type for a 2048-bit log-bloom, as used by our blocks.
pub type LogBloom = H2048;

/// Constant 2048-bit datum for 0. Often used as a default.
pub static ZERO_LOGBLOOM: LogBloom = H2048([0x00; 256]);

#[cfg_attr(feature="dev", allow(enum_variant_names))]
/// Semantic boolean for when a seal/signature is included.
pub enum Seal {
	/// The seal/signature is included.
	With,
	/// The seal/signature is not included.
	Without,
}
