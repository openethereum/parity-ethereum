//! Nibble-orientated view onto byte-slice, allowing nibble-precision offsets.
use std::cmp::*;
use bytes::*;

/// Nibble-orientated view onto byte-slice, allowing nibble-precision offsets.
///
/// This is an immutable struct. No operations actually change it.
///
/// # Example
/// ```rust
/// extern crate ethcore_util;
/// use ethcore_util::nibbleslice::*;
/// fn main() {
///   let d1 = &[0x01u8, 0x23, 0x45];
///   let d2 = &[0x34u8, 0x50, 0x12];
///   let d3 = &[0x00u8, 0x12];
///   let n1 = NibbleSlice::new(d1);			// 0,1,2,3,4,5
///   let n2 = NibbleSlice::new(d2);			// 3,4,5,0,1,2
///   let n3 = NibbleSlice::new_offset(d3, 1);	// 0,1,2
///   assert!(n1 > n3);							// 0,1,2,... > 0,1,2
///   assert!(n1 < n2);							// 0,... < 3,...
///   assert!(n2.mid(3) == n3);					// 0,1,2 == 0,1,2
///   assert!(n1.starts_with(&n3));
///   assert_eq!(n1.common_prefix(&n3), 3);
///   assert_eq!(n2.mid(3).common_prefix(&n1), 3);
/// }
/// ```
#[derive(Debug, Copy, Clone, Eq, Ord)]
pub struct NibbleSlice<'a> {
	data: &'a [u8],
	offset: usize,
}

impl<'a> NibbleSlice<'a> {
	/// Create a new nibble slice with the given byte-slice.
	pub fn new(data: &[u8]) -> NibbleSlice { NibbleSlice::new_offset(data, 0) }

	/// Create a new nibble slice with the given byte-slice with a nibble offset.
	pub fn new_offset(data: &'a [u8], offset: usize) -> NibbleSlice { NibbleSlice{data: data, offset: offset} }

	/// Create a new nibble slice from the given HPE encoded data (e.g. output of `encoded()`).
	pub fn from_encoded(data: &'a [u8]) -> (NibbleSlice, bool) {
		(Self::new_offset(data, if data[0] & 16 == 16 {1} else {2}), data[0] & 32 == 32)
	}

	/// Is this an empty slice?
	pub fn is_empty(&self) -> bool { self.len() == 0 }

	/// Get the length (in nibbles, naturally) of this slice.
	pub fn len(&self) -> usize { self.data.len() * 2 - self.offset }

	/// Get the nibble at position `i`.
	pub fn at(&self, i: usize) -> u8 {
		if (self.offset + i) & 1 == 1 {
			self.data[(self.offset + i) / 2] & 15u8
		}
		else {
			self.data[(self.offset + i) / 2] >> 4
		}
	}

	/// Return object which represents a view on to this slice (further) offset by `i` nibbles.
	pub fn mid(&self, i: usize) -> Self { NibbleSlice{ data: self.data, offset: self.offset + i} }

	/// Do we start with the same nibbles as the whole of `them`?
 	pub fn starts_with(&self, them: &Self) -> bool { self.common_prefix(them) == them.len() }

 	/// How many of the same nibbles at the beginning do we match with `them`?
	pub fn common_prefix(&self, them: &Self) -> usize {
		let s = min(self.len(), them.len());
		let mut i = 0usize;
		while i < s {
			if self.at(i) != them.at(i) { break; }
			i += 1;
		}
		i
	}

	pub fn encoded(&self, is_leaf: bool) -> Bytes {
		let l = self.len();
		let mut r = Bytes::with_capacity(l / 2 + 1);
		let mut i = l % 2;
		r.push(if i == 1 {0x10 + self.at(0)} else {0} + if is_leaf {0x20} else {0});
		while i < l {
			r.push(self.at(i) * 16 + self.at(i + 1));
			i += 2;
		}
		r
	}

	pub fn encoded_leftmost(&self, n: usize, is_leaf: bool) -> Bytes {
		let l = min(self.len(), n);
		let mut r = Bytes::with_capacity(l / 2 + 1);
		let mut i = l % 2;
		r.push(if i == 1 {0x10 + self.at(0)} else {0} + if is_leaf {0x20} else {0});
		while i < l {
			r.push(self.at(i) * 16 + self.at(i + 1));
			i += 2;
		}
		r
	}
}

impl<'a> PartialEq for NibbleSlice<'a> {
	fn eq(&self, them: &Self) -> bool {
		self.len() == them.len() && self.starts_with(them)
	}
}

impl<'a> PartialOrd for NibbleSlice<'a> {
	fn partial_cmp(&self, them: &Self) -> Option<Ordering> {
		let s = min(self.len(), them.len());
		let mut i = 0usize;
		while i < s {
			match self.at(i).partial_cmp(&them.at(i)).unwrap() {
				Ordering::Less => return Some(Ordering::Less),
				Ordering::Greater => return Some(Ordering::Greater),
				_ => i += 1,
			}
		}
		self.len().partial_cmp(&them.len())
	}
}

#[cfg(test)]
mod tests {
	use super::NibbleSlice;
	static D: &'static [u8;3] = &[0x01u8, 0x23, 0x45];

	#[test]
	fn basics() {
		let n = NibbleSlice::new(D);
		assert_eq!(n.len(), 6);
		assert!(!n.is_empty());

		let n = NibbleSlice::new_offset(D, 6);
		assert!(n.is_empty());

		let n = NibbleSlice::new_offset(D, 3);
		assert_eq!(n.len(), 3);
		for i in 0..3 {
			assert_eq!(n.at(i), i as u8 + 3);
		}
	}

	#[test]
	fn mid() {
		let n = NibbleSlice::new(D);
		let m = n.mid(2);
		for i in 0..4 {
			assert_eq!(m.at(i), i as u8 + 2);
		}
		let m = n.mid(3);
		for i in 0..3 {
			assert_eq!(m.at(i), i as u8 + 3);
		}
	}

	#[test]
	fn encoded() {
		let n = NibbleSlice::new(D);
		assert_eq!(n.encoded(false), &[0x00, 0x01, 0x23, 0x45]);
		assert_eq!(n.encoded(true), &[0x20, 0x01, 0x23, 0x45]);
		assert_eq!(n.mid(1).encoded(false), &[0x11, 0x23, 0x45]);
		assert_eq!(n.mid(1).encoded(true), &[0x31, 0x23, 0x45]);
	}

	#[test]
	fn from_encoded() {
		let n = NibbleSlice::new(D);
		assert_eq!((n, false), NibbleSlice::from_encoded(&[0x00, 0x01, 0x23, 0x45]));
		assert_eq!((n, true), NibbleSlice::from_encoded(&[0x20, 0x01, 0x23, 0x45]));
		assert_eq!((n.mid(1), false), NibbleSlice::from_encoded(&[0x11, 0x23, 0x45]));
		assert_eq!((n.mid(1), true), NibbleSlice::from_encoded(&[0x31, 0x23, 0x45]));
	}

	#[test]
	fn shared() {
		let n = NibbleSlice::new(D);

		let other = &[0x01u8, 0x23, 0x01, 0x23, 0x45, 0x67];
		let m = NibbleSlice::new(other);

		assert_eq!(n.common_prefix(&m), 4);
		assert_eq!(m.common_prefix(&n), 4);
		assert_eq!(n.mid(1).common_prefix(&m.mid(1)), 3);
		assert_eq!(n.mid(1).common_prefix(&m.mid(2)), 0);
		assert_eq!(n.common_prefix(&m.mid(4)), 6);
		assert!(!n.starts_with(&m.mid(4)));
		assert!(m.mid(4).starts_with(&n));
	}

	#[test]
	fn compare() {
		let other = &[0x01u8, 0x23, 0x01, 0x23, 0x45];
		let n = NibbleSlice::new(D);
		let m = NibbleSlice::new(other);

		assert!(n != m);
		assert!(n > m);
		assert!(m < n);

		assert!(n == m.mid(4));
		assert!(n >= m.mid(4));
		assert!(n <= m.mid(4));
	}
}