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


//! An owning, nibble-oriented byte vector.

use ::NibbleSlice;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// Owning, nibble-oriented byte vector. Counterpart to `NibbleSlice`.
pub struct NibbleVec {
	inner: Vec<u8>,
	len: usize,
}

impl NibbleVec {
	/// Make a new `NibbleVec`
	pub fn new() -> Self {
		NibbleVec {
			inner: Vec::new(),
			len: 0
		}
	}

	/// Make a `NibbleVec` with capacity for `n` nibbles.
	pub fn with_capacity(n: usize) -> Self {
		NibbleVec {
			inner: Vec::with_capacity((n / 2) + (n % 2)),
			len: 0
		}
	}

	/// Length of the `NibbleVec`
	pub fn len(&self) -> usize { self.len }

	/// Retrurns true if `NibbleVec` has zero length
	pub fn is_empty(&self) -> bool { self.len == 0 }

	/// Capacity of the `NibbleVec`.
	pub fn capacity(&self) -> usize { self.inner.capacity() * 2 }

	/// Try to get the nibble at the given offset.
	pub fn at(&self, idx: usize) -> u8 {
		if idx % 2 == 0 {
			self.inner[idx / 2] >> 4
		} else {
			self.inner[idx / 2] & 0x0F
		}
	}

	/// Push a nibble onto the `NibbleVec`. Ignores the high 4 bits.
	pub fn push(&mut self, nibble: u8) {
		let nibble = nibble & 0x0F;

		if self.len % 2 == 0 {
			self.inner.push(nibble << 4);
		} else {
			*self.inner.last_mut().expect("len != 0 since len % 2 != 0; inner has a last element; qed") |= nibble;
		}

		self.len += 1;
	}

	/// Try to pop a nibble off the `NibbleVec`. Fails if len == 0.
	pub fn pop(&mut self) -> Option<u8> {
		if self.is_empty() {
			return None;
		}

		let byte = self.inner.pop().expect("len != 0; inner has last elem; qed");
		let nibble = if self.len % 2 == 0 {
			self.inner.push(byte & 0xF0);
			byte & 0x0F
		} else {
			byte >> 4
		};

		self.len -= 1;
		Some(nibble)
	}

	/// Try to treat this `NibbleVec` as a `NibbleSlice`. Works only if len is even.
	pub fn as_nibbleslice(&self) -> Option<NibbleSlice> {
		if self.len % 2 == 0 {
			Some(NibbleSlice::new(self.inner()))
		} else {
			None
		}
	}

	/// Get the underlying byte slice.
	pub fn inner(&self) -> &[u8] {
		&self.inner[..]
	}
}

impl<'a> From<NibbleSlice<'a>> for NibbleVec {
	fn from(s: NibbleSlice<'a>) -> Self {
		let mut v = NibbleVec::with_capacity(s.len());
		for i in 0..s.len() {
			v.push(s.at(i));
		}
		v
	}
}

#[cfg(test)]
mod tests {
	use super::NibbleVec;

	#[test]
	fn push_pop() {
		let mut v = NibbleVec::new();

		for i in 0..16 {
			v.push(i);
			assert_eq!(v.len() - 1, i as usize);
			assert_eq!(v.at(i as usize), i);
		}

		for i in (0..16).rev() {
			assert_eq!(v.pop(), Some(i));
			assert_eq!(v.len(), i as usize);
		}
	}

	#[test]
	fn nibbleslice_conv() {
		let mut v = NibbleVec::new();
		for i in 0..10 {
			v.push(i);
		}

		let v2: NibbleVec = v.as_nibbleslice().unwrap().into();
		assert_eq!(v, v2);
	}
}
