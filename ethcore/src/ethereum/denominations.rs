// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use ethereum_types::U256;

#[inline]
/// 1 Ether in Wei
pub fn ether() -> U256 { U256::exp10(18) }

#[inline]
/// 1 Finney in Wei
pub fn finney() -> U256 { U256::exp10(15) }

#[inline]
/// 1 Szabo in Wei
pub fn szabo() -> U256 { U256::exp10(12) }

#[inline]
/// 1 Shannon in Wei
pub fn shannon() -> U256 { U256::exp10(9) }

#[inline]
/// 1 Wei in Wei
pub fn wei() -> U256 { U256::exp10(0) }
