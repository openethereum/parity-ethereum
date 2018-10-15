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

//! Small utility for signed 256-bit integer.

use std::ops::{Add, AddAssign, Sub, SubAssign};
use ethereum_types::U256;

/// Sign of a 256-bit integer.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Sign {
	Positive,
	Zero,
	Negative
}

/// Representation of a signed 256-bit integer.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct I256(Sign, U256);

impl I256 {
	/// Get the absolute value of the signed integer.
	pub fn abs(&self) -> U256 {
		self.1
	}

	/// Get the sign of the integer.
	pub fn sign(&self) -> Sign {
		self.0
	}

	/// Check whether the signed integer is non-negative.
	pub fn is_nonnegative(&self) -> bool {
		self.0 != Sign::Negative
	}
}

impl Default for I256 {
	fn default() -> Self {
		I256(Sign::Zero, U256::zero())
	}
}

impl From<u64> for I256 {
	fn from(value: u64) -> Self {
		I256::from(U256::from(value))
	}
}

impl From<U256> for I256 {
	fn from(value: U256) -> Self {
		if value.is_zero() {
			I256(Sign::Zero, value)
		} else {
			I256(Sign::Positive, value)
		}
	}
}

impl Add<U256> for I256 {
	type Output = Self;

	fn add(mut self, other: U256) -> Self {
		self.add_assign(other);
		self
	}
}

impl AddAssign<U256> for I256 {
	fn add_assign(&mut self, other: U256) {
		match self.0 {
			Sign::Positive => {
				self.1 += other;
			},
			Sign::Zero => {
				self.0 = Sign::Positive;
				self.1 = other;
			},
			Sign::Negative => {
				if other > self.1 {
					self.0 = Sign::Positive;
					self.1 = other - self.1;
				} else if other < self.1 {
					self.1 -= other;
				} else {
					self.0 = Sign::Zero;
					self.1 = U256::zero();
				}
			},
		}
	}
}

impl Sub<U256> for I256 {
	type Output = Self;

	fn sub(mut self, other: U256) -> Self {
		self.sub_assign(other);
		self
	}
}

impl SubAssign<U256> for I256 {
	fn sub_assign(&mut self, other: U256) {
		match self.0 {
			Sign::Positive => {
				if other > self.1 {
					self.0 = Sign::Negative;
					self.1 = other - self.1;
				} else if other < self.1 {
					self.1 -= other;
				} else {
					self.0 = Sign::Zero;
					self.1 = U256::zero();
				}
			},
			Sign::Zero => {
				self.0 = Sign::Negative;
				self.1 = other;
			},
			Sign::Negative => {
				self.1 += other;
			},
		}
	}
}

impl Add<I256> for I256 {
	type Output = Self;

	fn add(mut self, other: I256) -> Self {
		self.add_assign(other);
		self
	}
}

impl AddAssign<I256> for I256 {
	fn add_assign(&mut self, other: I256) {
		match other.0 {
			Sign::Positive => *self += other.1,
			Sign::Zero => (),
			Sign::Negative => *self -= other.1,
		}
	}
}
