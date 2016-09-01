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

//! General bytes-related utilities.
//!
//! Includes a pretty-printer for bytes, in the form of `ToPretty` and `PrettySlice`
//! as

use std::fmt;
use std::ops::{Deref, DerefMut};

/// Slice pretty print helper
pub struct PrettySlice<'a> (&'a [u8]);

impl<'a> fmt::Debug for PrettySlice<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for i in 0..self.0.len() {
			match i > 0 {
				true => { try!(write!(f, "·{:02x}", self.0[i])); },
				false => { try!(write!(f, "{:02x}", self.0[i])); },
			}
		}
		Ok(())
	}
}

impl<'a> fmt::Display for PrettySlice<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for i in 0..self.0.len() {
			try!(write!(f, "{:02x}", self.0[i]));
		}
		Ok(())
	}
}

/// Trait to allow a type to be pretty-printed in `format!`, where unoverridable
/// defaults cannot otherwise be avoided.
pub trait ToPretty {
	/// Convert a type into a derivative form in order to make `format!` print it prettily.
	fn pretty(&self) -> PrettySlice;
	/// Express the object as a hex string.
	fn to_hex(&self) -> String {
		format!("{}", self.pretty())
	}
}

impl<T: AsRef<[u8]>> ToPretty for T {
	fn pretty(&self) -> PrettySlice {
		PrettySlice(self.as_ref())
	}
}

/// A byte collection reference that can either be a slice or a vector
pub enum BytesRef<'a> {
	/// This is a reference to a vector
	Flexible(&'a mut Bytes),
	/// This is a reference to a slice
	Fixed(&'a mut [u8])
}

impl<'a> Deref for BytesRef<'a> {
	type Target = [u8];

	fn deref(&self) -> &[u8] {
		match *self {
			BytesRef::Flexible(ref bytes) => bytes,
			BytesRef::Fixed(ref bytes) => bytes,
		}
	}
}

impl <'a> DerefMut for BytesRef<'a> {
	fn deref_mut(&mut self) -> &mut [u8] {
		match *self {
			BytesRef::Flexible(ref mut bytes) => bytes,
			BytesRef::Fixed(ref mut bytes) => bytes,
		}
	}
}

/// Vector of bytes.
pub type Bytes = Vec<u8>;