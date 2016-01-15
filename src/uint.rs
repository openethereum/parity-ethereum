// taken from Rust Bitcoin Library (https://github.com/apoelstra/rust-bitcoin)
// original author: Andrew Poelstra <apoelstra@wpsoftware.net>

// Rust Bitcoin Library
// Written in 2014 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! Big unsigned integer types
///!
///! Implementation of a various large-but-fixed sized unsigned integer types.
///! The functions here are designed to be fast.
///!

use standard::*;
use from_json::*;
use std::num::wrapping::OverflowingOps;

macro_rules! impl_map_from {
	($thing:ident, $from:ty, $to:ty) => {
		impl From<$from> for $thing {
			fn from(value: $from) -> $thing {
				From::from(value as $to)
			}
		}
	}
}

macro_rules! panic_on_overflow {
	($name:expr) => {
		if $name {
			panic!("arithmetic operation overflow")
		}
	}
}
pub trait Uint: Sized + Default + FromStr + From<u64> + FromJson + fmt::Debug + fmt::Display + PartialOrd + Ord + PartialEq + Eq + Hash {

	/// Size of this type.
	const SIZE: usize;

	fn zero() -> Self;
	fn one() -> Self;

	type FromDecStrErr;
	fn from_dec_str(value: &str) -> Result<Self, Self::FromDecStrErr>;

	/// Conversion to u32
	fn low_u32(&self) -> u32;

	/// Conversion to u64
	fn low_u64(&self) -> u64;

	/// Conversion to u32 with overflow checking
	fn as_u32(&self) -> u32;

	/// Conversion to u64 with overflow checking
	fn as_u64(&self) -> u64;
	
	/// Return the least number of bits needed to represent the number
	fn bits(&self) -> usize;
	fn bit(&self, index: usize) -> bool;
	fn byte(&self, index: usize) -> u8;
	fn to_bytes(&self, bytes: &mut[u8]);

	fn exp10(n: usize) -> Self;
}

macro_rules! construct_uint {
	($name:ident, $n_words:expr) => (
		/// Little-endian large integer type
		#[derive(Copy, Clone, Eq, PartialEq)]
		pub struct $name(pub [u64; $n_words]);

		impl Uint for $name {
			const SIZE: usize = $n_words * 8;

			type FromDecStrErr = FromHexError;

			/// TODO: optimize, throw appropriate err
			fn from_dec_str(value: &str) -> Result<Self, Self::FromDecStrErr> {
				Ok(value.bytes()
				   .map(|b| b - 48)
				   .fold($name::from(0u64), | acc, c |
						 // fast multiplication by 10
						 // (acc << 3) + (acc << 1) => acc * 10
						 (acc << 3) + (acc << 1) + $name::from(c)
					))
			}

			/// Conversion to u32
			#[inline]
			fn low_u32(&self) -> u32 {
				let &$name(ref arr) = self;
				arr[0] as u32
			}

			/// Conversion to u64
			#[inline]
			fn low_u64(&self) -> u64 {
				let &$name(ref arr) = self;
				arr[0]
			}

			/// Conversion to u32 with overflow checking
			#[inline]
			fn as_u32(&self) -> u32 {
				let &$name(ref arr) = self;
				if (arr[0] & (0xffffffffu64 << 32)) != 0 {
					panic!("Integer overflow when casting U256") 
				}
				self.as_u64() as u32
			}

			/// Conversion to u64 with overflow checking
			#[inline]
			fn as_u64(&self) -> u64 {
				let &$name(ref arr) = self;
				for i in 1..$n_words {
					if arr[i] != 0 {
						panic!("Integer overflow when casting U256") 
					}
				}
				arr[0]
			}
			/// Return the least number of bits needed to represent the number
			#[inline]
			fn bits(&self) -> usize {
				let &$name(ref arr) = self;
				for i in 1..$n_words {
					if arr[$n_words - i] > 0 { return (0x40 * ($n_words - i + 1)) - arr[$n_words - i].leading_zeros() as usize; }
				}
				0x40 - arr[0].leading_zeros() as usize
			}

			#[inline]
			fn bit(&self, index: usize) -> bool {
				let &$name(ref arr) = self;
				arr[index / 64] & (1 << (index % 64)) != 0
			}

			#[inline]
			fn byte(&self, index: usize) -> u8 {
				let &$name(ref arr) = self;
				(arr[index / 8] >> ((index % 8)) * 8) as u8
			}

			fn to_bytes(&self, bytes: &mut[u8]) {
				assert!($n_words * 8 == bytes.len());
				let &$name(ref arr) = self;
				for i in 0..bytes.len() {
					let rev = bytes.len() - 1 - i;
					let pos = rev / 8;
					bytes[i] = (arr[pos] >> ((rev % 8) * 8)) as u8;
				}
			}

			#[inline]
			fn exp10(n: usize) -> $name {
				match n {
					0 => $name::from(1u64),
					_ => $name::exp10(n - 1) * $name::from(10u64)
				}
			}

			#[inline]
			fn zero() -> $name {
				From::from(0u64)
			}

			#[inline]
			fn one() -> $name {
				From::from(1u64)
			}
		}

		impl $name {

			pub fn pow(self, expon: $name) -> $name {
				if expon == $name::zero() {
					return $name::one()
				}
				let is_even = |x : &$name| x.low_u64() & 1 == 0;

				let u_one = $name::one();
				let u_two = $name::from(2);
				let mut y = u_one;
				let mut n = expon;
				let mut x = self;
				while n > u_one {
					if is_even(&n) {
						x = x * x;
						n = n / u_two;
					} else {
						y = x * y;
						x = x * x;
						n = (n - u_one) / u_two;
					}
				}
				x * y
			}

			pub fn overflowing_pow(self, expon: $name) -> ($name, bool) {
				if expon == $name::zero() {
					return ($name::one(), false)
				}
				let is_even = |x : &$name| x.low_u64() & 1 == 0;

				let u_one = $name::one();
				let u_two = $name::from(2);
				let mut y = u_one;
				let mut n = expon;
				let mut x = self;
				let mut overflow = false;

				while n > u_one {
					if is_even(&n) {
						let (c, mul_overflow) = x.overflowing_mul(x);
						x = c;
						overflow |= mul_overflow;
						n = n / u_two;
					} else {
						let (new_y, y_overflow) = x.overflowing_mul(y);
						let (new_x, x_overflow) = x.overflowing_mul(x);
						x = new_x;
						y = new_y;
						overflow |= y_overflow | x_overflow;

						n = (n - u_one) / u_two;
					}
				}
				let (res, mul_overflow) = x.overflowing_mul(y);
				(res, mul_overflow | overflow)
			}

			/// Multiplication by u32
			fn mul_u32(self, other: u32) -> $name {
				let $name(ref arr) = self;
				let mut carry = [0u64; $n_words];
				let mut ret = [0u64; $n_words];
				for i in 0..$n_words {
					let upper = other as u64 * (arr[i] >> 32);
					let lower = other as u64 * (arr[i] & 0xFFFFFFFF);

					ret[i] = lower.wrapping_add(upper << 32);

					if i < $n_words - 1 {
						carry[i + 1] = upper >> 32;
						if ret[i] < lower {
							carry[i + 1] += 1;
						}
					}
				}
				$name(ret) + $name(carry)
			}

			/// Overflowing multiplication by u32
			fn overflowing_mul_u32(self, other: u32) -> ($name, bool) {
				let $name(ref arr) = self;
				let mut carry = [0u64; $n_words];
				let mut ret = [0u64; $n_words];
				let mut overflow = false;
				for i in 0..$n_words {
					let upper = other as u64 * (arr[i] >> 32);
					let lower = other as u64 * (arr[i] & 0xFFFFFFFF);

					ret[i] = lower.wrapping_add(upper << 32);

					if i < $n_words - 1 {
						carry[i + 1] = upper >> 32;
						if ret[i] < lower {
							carry[i + 1] += 1;
						}
					} else if (upper >> 32) > 0 || ret[i] < lower {
						overflow = true
					}
				}
				let (result, add_overflow) = $name(ret).overflowing_add($name(carry));
				(result, add_overflow || overflow)
			}
		}

		impl Default for $name {
			fn default() -> Self {
				$name::zero()
			}
		}

		impl From<u64> for $name {
			fn from(value: u64) -> $name {
				let mut ret = [0; $n_words];
				ret[0] = value;
				$name(ret)
			}
		}

		impl FromJson for $name {
			fn from_json(json: &Json) -> Self {
				match json {
					&Json::String(ref s) => {
						if s.len() >= 2 && &s[0..2] == "0x" {
							FromStr::from_str(&s[2..]).unwrap_or(Default::default())
						} else {
							Uint::from_dec_str(s).unwrap_or(Default::default())
						}
					},
					&Json::U64(u) => From::from(u),
					&Json::I64(i) => From::from(i as u64),
					_ => Uint::zero(),
				}
			}
		}

		impl_map_from!($name, u8, u64);
		impl_map_from!($name, u16, u64);
		impl_map_from!($name, u32, u64);
		impl_map_from!($name, usize, u64);

		impl From<i64> for $name {
			fn from(value: i64) -> $name {
				match value >= 0 {
					true => From::from(value as u64),
					false => { panic!("Unsigned integer can't be created from negative value"); }
				}
			}
		}

		impl_map_from!($name, i8, i64);
		impl_map_from!($name, i16, i64);
		impl_map_from!($name, i32, i64);
		impl_map_from!($name, isize, i64);

		impl<'a> From<&'a [u8]> for $name {
			fn from(bytes: &[u8]) -> $name {
				assert!($n_words * 8 >= bytes.len());

				let mut ret = [0; $n_words];
				for i in 0..bytes.len() {
					let rev = bytes.len() - 1 - i;
					let pos = rev / 8;
					ret[pos] += (bytes[i] as u64) << (rev % 8) * 8;
				}
				$name(ret)
			}
		}

		impl FromStr for $name {
			type Err = FromHexError;

			fn from_str(value: &str) -> Result<$name, Self::Err> {
				let bytes: Vec<u8> = match value.len() % 2 == 0 {
					true => try!(value.from_hex()),
					false => try!(("0".to_string() + value).from_hex())
				};

				let bytes_ref: &[u8] = &bytes;
				Ok(From::from(bytes_ref))
			}
		}

		impl OverflowingOps for $name {
			fn overflowing_add(self, other: $name) -> ($name, bool) {
				let $name(ref me) = self;
				let $name(ref you) = other;
				let mut ret = [0u64; $n_words];
				let mut carry = [0u64; $n_words];
				let mut b_carry = false;
				let mut overflow = false;

				for i in 0..$n_words {
					ret[i] = me[i].wrapping_add(you[i]);

					if ret[i] < me[i] {
						if i < $n_words - 1 {
							carry[i + 1] = 1;
							b_carry = true;
						} else {
							overflow = true
						}
					}
				}
				if b_carry { 
					let (ret, add_overflow) = $name(ret).overflowing_add($name(carry));
					(ret, add_overflow || overflow)
				} else { 
					($name(ret), overflow)
				}
			}

			fn overflowing_sub(self, other: $name) -> ($name, bool) {
				let (res, _overflow) = (!other).overflowing_add(From::from(1u64));
				let (res, _overflow) = self.overflowing_add(res);
				(res, self < other)
			}

			fn overflowing_mul(self, other: $name) -> ($name, bool) {
				let mut res = $name::from(0u64);
				let mut overflow = false;
				// TODO: be more efficient about this
				for i in 0..(2 * $n_words) {
					let (v, mul_overflow) = self.overflowing_mul_u32((other >> (32 * i)).low_u32());
					let (new_res, add_overflow) = res.overflowing_add(v << (32 * i));
					res = new_res;
					overflow = overflow || mul_overflow || add_overflow;
				}
				(res, overflow)
			}

			fn overflowing_div(self, other: $name) -> ($name, bool) {
				(self / other, false)
			}

			fn overflowing_rem(self, other: $name) -> ($name, bool) {
				(self % other, false)
			}

			fn overflowing_neg(self) -> ($name, bool) {
				(!self, true)
			}

			fn overflowing_shl(self, _shift32: u32) -> ($name, bool) {
				// TODO [todr] not used for now
				unimplemented!();
			}

			fn overflowing_shr(self, _shift32: u32) -> ($name, bool) {
				// TODO [todr] not used for now
				unimplemented!();
			}
		}

		impl Add<$name> for $name {
			type Output = $name;

			fn add(self, other: $name) -> $name {
				let $name(ref me) = self;
				let $name(ref you) = other;
				let mut ret = [0u64; $n_words];
				let mut carry = [0u64; $n_words];
				let mut b_carry = false;
				for i in 0..$n_words {
					if i < $n_words - 1 {
						ret[i] = me[i].wrapping_add(you[i]);
						if ret[i] < me[i] {
							carry[i + 1] = 1;
							b_carry = true;
						}
					} else {
						ret[i] = me[i] + you[i];
					}
				}
				if b_carry { $name(ret) + $name(carry) } else { $name(ret) }
			}
		}

		impl Sub<$name> for $name {
			type Output = $name;

			#[inline]
			fn sub(self, other: $name) -> $name {
				panic_on_overflow!(self < other);
				let (res, _overflow) = (!other).overflowing_add(From::from(1u64));
				let (res, _overflow) = self.overflowing_add(res);
				res
			}
		}

		impl Mul<$name> for $name {
			type Output = $name;

			fn mul(self, other: $name) -> $name {
				let mut res = $name::from(0u64);
				// TODO: be more efficient about this
				for i in 0..(2 * $n_words) {
					res = res + (self.mul_u32((other >> (32 * i)).low_u32()) << (32 * i));
				}
				res
			}
		}

		impl Div<$name> for $name {
			type Output = $name;

			fn div(self, other: $name) -> $name {
				let mut sub_copy = self;
				let mut shift_copy = other;
				let mut ret = [0u64; $n_words];

				let my_bits = self.bits();
				let your_bits = other.bits();

				// Check for division by 0
				assert!(your_bits != 0);

				// Early return in case we are dividing by a larger number than us
				if my_bits < your_bits {
					return $name(ret);
				}

				// Bitwise long division
				let mut shift = my_bits - your_bits;
				shift_copy = shift_copy << shift;
				loop {
					if sub_copy >= shift_copy {
						ret[shift / 64] |= 1 << (shift % 64);
						let (copy, _overflow) = sub_copy.overflowing_sub(shift_copy);
						sub_copy = copy
					}
					shift_copy = shift_copy >> 1;
					if shift == 0 { break; }
					shift -= 1;
				}

				$name(ret)
			}
		}

		impl Rem<$name> for $name {
			type Output = $name;

			fn rem(self, other: $name) -> $name {
				let times = self / other;
				self - (times * other)
			}
		}

		impl BitAnd<$name> for $name {
			type Output = $name;

			#[inline]
			fn bitand(self, other: $name) -> $name {
				let $name(ref arr1) = self;
				let $name(ref arr2) = other;
				let mut ret = [0u64; $n_words];
				for i in 0..$n_words {
					ret[i] = arr1[i] & arr2[i];
				}
				$name(ret)
			}
		}

		impl BitXor<$name> for $name {
			type Output = $name;

			#[inline]
			fn bitxor(self, other: $name) -> $name {
				let $name(ref arr1) = self;
				let $name(ref arr2) = other;
				let mut ret = [0u64; $n_words];
				for i in 0..$n_words {
					ret[i] = arr1[i] ^ arr2[i];
				}
				$name(ret)
			}
		}

		impl BitOr<$name> for $name {
			type Output = $name;

			#[inline]
			fn bitor(self, other: $name) -> $name {
				let $name(ref arr1) = self;
				let $name(ref arr2) = other;
				let mut ret = [0u64; $n_words];
				for i in 0..$n_words {
					ret[i] = arr1[i] | arr2[i];
				}
				$name(ret)
			}
		}

		impl Not for $name {
			type Output = $name;

			#[inline]
			fn not(self) -> $name {
				let $name(ref arr) = self;
				let mut ret = [0u64; $n_words];
				for i in 0..$n_words {
					ret[i] = !arr[i];
				}
				$name(ret)
			}
		}

		impl Shl<usize> for $name {
			type Output = $name;

			fn shl(self, shift: usize) -> $name {
				let $name(ref original) = self;
				let mut ret = [0u64; $n_words];
				let word_shift = shift / 64;
				let bit_shift = shift % 64;
				for i in 0..$n_words {
					// Shift
					if i + word_shift < $n_words {
						ret[i + word_shift] += original[i] << bit_shift;
					}
					// Carry
					if bit_shift > 0 && i + word_shift + 1 < $n_words {
						ret[i + word_shift + 1] += original[i] >> (64 - bit_shift);
					}
				}
				$name(ret)
			}
		}

		impl Shr<usize> for $name {
			type Output = $name;

			fn shr(self, shift: usize) -> $name {
				let $name(ref original) = self;
				let mut ret = [0u64; $n_words];
				let word_shift = shift / 64;
				let bit_shift = shift % 64;
				for i in word_shift..$n_words {
					// Shift
					ret[i - word_shift] += original[i] >> bit_shift;
					// Carry
					if bit_shift > 0 && i < $n_words - 1 {
						ret[i - word_shift] += original[i + 1] << (64 - bit_shift);
					}
				}
				$name(ret)
			}
		}

		impl Ord for $name {
			fn cmp(&self, other: &$name) -> Ordering {
				let &$name(ref me) = self;
				let &$name(ref you) = other;
				for i in 0..$n_words {
					if me[$n_words - 1 - i] < you[$n_words - 1 - i] { return Ordering::Less; }
					if me[$n_words - 1 - i] > you[$n_words - 1 - i] { return Ordering::Greater; }
				}
				Ordering::Equal
			}
		}

		impl PartialOrd for $name {
			fn partial_cmp(&self, other: &$name) -> Option<Ordering> {
				Some(self.cmp(other))
			}
		}

		impl fmt::Debug for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				let &$name(ref data) = self;
				try!(write!(f, "0x"));
				for ch in data.iter().rev() {
					try!(write!(f, "{:02x}", ch));
				}
				Ok(())
			}
		}

		impl fmt::Display for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				let &$name(ref data) = self;
				try!(write!(f, "0x"));
				for ch in data.iter().rev() {
					try!(write!(f, "{:02x}", ch));
				}
				Ok(())
			}
		}

		impl Hash for $name {
			fn hash<H>(&self, state: &mut H) where H: Hasher {
				unsafe { state.write(::std::slice::from_raw_parts(self.0.as_ptr() as *mut u8, self.0.len() * 8)); }
				state.finish();
			}
		}
	);
}

construct_uint!(U512, 8);
construct_uint!(U256, 4);
construct_uint!(U128, 2);

impl From<U256> for U512 {
	fn from(value: U256) -> U512 {
		let U256(ref arr) = value;
		let mut ret = [0; 8];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		U512(ret)
	}
}

impl From<U512> for U256 {
	fn from(value: U512) -> U256 {
		let U512(ref arr) = value;
		if arr[4] | arr[5] | arr[6] | arr[7] != 0 {
			panic!("Overflow");
		}
		let mut ret = [0; 4];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		U256(ret)
	}
}

impl From<U256> for U128 {
	fn from(value: U256) -> U128 {
		let U256(ref arr) = value;
		if arr[2] | arr[3] != 0 {
			panic!("Overflow");
		}
		let mut ret = [0; 2];
		ret[0] = arr[0];
		ret[1] = arr[1];
		U128(ret)
	}
}

impl From<U512> for U128 {
	fn from(value: U512) -> U128 {
		let U512(ref arr) = value;
		if arr[2] | arr[3] | arr[4] | arr[5] | arr[6] | arr[7] != 0 {
			panic!("Overflow");
		}
		let mut ret = [0; 2];
		ret[0] = arr[0];
		ret[1] = arr[1];
		U128(ret)
	}
}

impl From<U128> for U512 {
	fn from(value: U128) -> U512 {
		let U128(ref arr) = value;
		let mut ret = [0; 8];
		ret[0] = arr[0];
		ret[1] = arr[1];
		U512(ret)
	}
}

impl From<U128> for U256 {
	fn from(value: U128) -> U256 {
		let U128(ref arr) = value;
		let mut ret = [0; 4];
		ret[0] = arr[0];
		ret[1] = arr[1];
		U256(ret)
	}
}

impl From<U256> for u64 {
	fn from(value: U256) -> u64 {
		value.as_u64()
	}
}

impl From<U256> for u32 {
	fn from(value: U256) -> u32 {
		value.as_u32()
	}
}

pub const ZERO_U256: U256 = U256([0x00u64; 4]);
pub const ONE_U256: U256 = U256([0x01u64, 0x00u64, 0x00u64, 0x00u64]);
pub const BAD_U256: U256 = U256([0xffffffffffffffffu64; 4]);

#[cfg(test)]
mod tests {
	use uint::{Uint, U128, U256, U512};
	use std::str::FromStr;
	use std::num::wrapping::OverflowingOps;

	#[test]
	pub fn uint256_from() {
		let e = U256([10, 0, 0, 0]);

		// test unsigned initialization
		let ua = U256::from(10u8);
		let ub = U256::from(10u16);
		let uc =  U256::from(10u32);
		let ud = U256::from(10u64);
		assert_eq!(e, ua);
		assert_eq!(e, ub);
		assert_eq!(e, uc);
		assert_eq!(e, ud);

		// test initialization from bytes
		let va = U256::from(&[10u8][..]);
		assert_eq!(e, va);

		// more tests for initialization from bytes
		assert_eq!(U256([0x1010, 0, 0, 0]), U256::from(&[0x10u8, 0x10][..]));
		assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from(&[0x12u8, 0xf0][..]));
		assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from(&[0, 0x12u8, 0xf0][..]));
		assert_eq!(U256([0x12f0, 0 , 0, 0]), U256::from(&[0, 0, 0, 0, 0, 0, 0, 0x12u8, 0xf0][..]));
		assert_eq!(U256([0x12f0, 1 , 0, 0]), U256::from(&[1, 0, 0, 0, 0, 0, 0, 0x12u8, 0xf0][..]));
		assert_eq!(U256([0x12f0, 1 , 0x0910203040506077, 0x8090a0b0c0d0e0f0]), U256::from(&[
																						  0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0,
																						  0x09, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x77,
																						  0, 0, 0, 0, 0, 0, 0, 1,
																						  0, 0, 0, 0, 0, 0, 0x12u8, 0xf0][..]));
		assert_eq!(U256([0x00192437100019fa, 0x243710, 0, 0]), U256::from(&[
																		  0x24u8, 0x37, 0x10,
																		  0, 0x19, 0x24, 0x37, 0x10, 0, 0x19, 0xfa][..]));

		// test initializtion from string
		let sa = U256::from_str("0a").unwrap();
		assert_eq!(e, sa);
		assert_eq!(U256([0x1010, 0, 0, 0]), U256::from_str("1010").unwrap());
		assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from_str("12f0").unwrap());
		assert_eq!(U256([0x12f0, 0, 0, 0]), U256::from_str("12f0").unwrap());
		assert_eq!(U256([0x12f0, 0 , 0, 0]), U256::from_str("0000000012f0").unwrap());
		assert_eq!(U256([0x12f0, 1 , 0, 0]), U256::from_str("0100000000000012f0").unwrap());
		assert_eq!(U256([0x12f0, 1 , 0x0910203040506077, 0x8090a0b0c0d0e0f0]), U256::from_str("8090a0b0c0d0e0f00910203040506077000000000000000100000000000012f0").unwrap());
	}

	#[test]
	pub fn uint256_to() {
		let hex = "8090a0b0c0d0e0f00910203040506077583a2cf8264910e1436bda32571012f0";
		let uint = U256::from_str(hex).unwrap();
		let mut bytes = [0u8; 32];
		uint.to_bytes(&mut bytes);
		let uint2 = U256::from(&bytes[..]);
		assert_eq!(uint, uint2);
	}

	#[test]
	pub fn uint256_bits_test() {
		assert_eq!(U256::from(0u64).bits(), 0);
		assert_eq!(U256::from(255u64).bits(), 8);
		assert_eq!(U256::from(256u64).bits(), 9);
		assert_eq!(U256::from(300u64).bits(), 9);
		assert_eq!(U256::from(60000u64).bits(), 16);
		assert_eq!(U256::from(70000u64).bits(), 17);

		//// Try to read the following lines out loud quickly
		let mut shl = U256::from(70000u64);
		shl = shl << 100;
		assert_eq!(shl.bits(), 117);
		shl = shl << 100;
		assert_eq!(shl.bits(), 217);
		shl = shl << 100;
		assert_eq!(shl.bits(), 0);

		//// Bit set check
		//// 01010
		assert!(!U256::from(10u8).bit(0));
		assert!(U256::from(10u8).bit(1));
		assert!(!U256::from(10u8).bit(2));
		assert!(U256::from(10u8).bit(3));
		assert!(!U256::from(10u8).bit(4));

		//// byte check
		assert_eq!(U256::from(10u8).byte(0), 10);
		assert_eq!(U256::from(0xffu64).byte(0), 0xff);
		assert_eq!(U256::from(0xffu64).byte(1), 0);
		assert_eq!(U256::from(0x01ffu64).byte(0), 0xff);
		assert_eq!(U256::from(0x01ffu64).byte(1), 0x1);
		assert_eq!(U256([0u64, 0xfc, 0, 0]).byte(8), 0xfc);
		assert_eq!(U256([0u64, 0, 0, u64::max_value()]).byte(31), 0xff);
		assert_eq!(U256([0u64, 0, 0, (u64::max_value() >> 8) + 1]).byte(31), 0x01);
	}

	#[test]
	pub fn uint256_comp_test() {
		let small = U256([10u64, 0, 0, 0]);
		let big = U256([0x8C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0]);
		let bigger = U256([0x9C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0]);
		let biggest = U256([0x5C8C3EE70C644118u64, 0x0209E7378231E632, 0, 1]);

		assert!(small < big);
		assert!(big < bigger);
		assert!(bigger < biggest);
		assert!(bigger <= biggest);
		assert!(biggest <= biggest);
		assert!(bigger >= big);
		assert!(bigger >= small);
		assert!(small <= small);
	}

	#[test]
	pub fn uint256_arithmetic_test() {
		let init = U256::from(0xDEADBEEFDEADBEEFu64);
		let copy = init;

		let add = init + copy;
		assert_eq!(add, U256([0xBD5B7DDFBD5B7DDEu64, 1, 0, 0]));
		// Bitshifts
		let shl = add << 88;
		assert_eq!(shl, U256([0u64, 0xDFBD5B7DDE000000, 0x1BD5B7D, 0]));
		let shr = shl >> 40;
		assert_eq!(shr, U256([0x7DDE000000000000u64, 0x0001BD5B7DDFBD5B, 0, 0]));
		// Increment
		let incr = shr + U256::from(1u64);
		assert_eq!(incr, U256([0x7DDE000000000001u64, 0x0001BD5B7DDFBD5B, 0, 0]));
		// Subtraction
		let (sub, _of) = incr.overflowing_sub(init);
		assert_eq!(sub, U256([0x9F30411021524112u64, 0x0001BD5B7DDFBD5A, 0, 0]));
		// Multiplication
		let mult = sub.mul_u32(300);
		assert_eq!(mult, U256([0x8C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0]));
		// Division
		assert_eq!(U256::from(105u8) / U256::from(5u8), U256::from(21u8));
		let div = mult / U256::from(300u16);
		assert_eq!(div, U256([0x9F30411021524112u64, 0x0001BD5B7DDFBD5A, 0, 0]));
		//// TODO: bit inversion
	}

	#[test]
	pub fn uint256_extreme_bitshift_test() {
		//// Shifting a u64 by 64 bits gives an undefined value, so make sure that
		//// we're doing the Right Thing here
		let init = U256::from(0xDEADBEEFDEADBEEFu64);

		assert_eq!(init << 64, U256([0, 0xDEADBEEFDEADBEEF, 0, 0]));
		let add = (init << 64) + init;
		assert_eq!(add, U256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0]));
		assert_eq!(add >> 0, U256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0]));
		assert_eq!(add << 0, U256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0]));
		assert_eq!(add >> 64, U256([0xDEADBEEFDEADBEEF, 0, 0, 0]));
		assert_eq!(add << 64, U256([0, 0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0]));
	}

	#[test]
	pub fn uint256_exp10() {
		assert_eq!(U256::exp10(0), U256::from(1u64));
		println!("\none: {:?}", U256::from(1u64));
		println!("ten: {:?}", U256::from(10u64));
		assert_eq!(U256::from(2u64) * U256::from(10u64), U256::from(20u64));
		assert_eq!(U256::exp10(1), U256::from(10u64));
		assert_eq!(U256::exp10(2), U256::from(100u64));
		assert_eq!(U256::exp10(5), U256::from(100000u64));
	}

	#[test]
	pub fn uint256_mul32() {
		assert_eq!(U256::from(0u64).mul_u32(2), U256::from(0u64));
		assert_eq!(U256::from(1u64).mul_u32(2), U256::from(2u64));
		assert_eq!(U256::from(10u64).mul_u32(2), U256::from(20u64));
		assert_eq!(U256::from(10u64).mul_u32(5), U256::from(50u64));
		assert_eq!(U256::from(1000u64).mul_u32(50), U256::from(50000u64));
	}

	#[test]
	fn uint256_pow () {
		assert_eq!(U256::from(10).pow(U256::from(0)), U256::from(1));
		assert_eq!(U256::from(10).pow(U256::from(1)), U256::from(10));
		assert_eq!(U256::from(10).pow(U256::from(2)), U256::from(100));
		assert_eq!(U256::from(10).pow(U256::from(3)), U256::from(1000));
		assert_eq!(U256::from(10).pow(U256::from(20)), U256::exp10(20));
	}

	#[test]
	#[should_panic]
	fn uint256_pow_overflow () {
		U256::from(2).pow(U256::from(0x001));
	}

	#[test]
	fn uint256_overflowing_pow () {
		assert_eq!(
			U256::from(2).overflowing_pow(U256::from(0xfe)),
			(U256::zero(), false)
		);
		assert_eq!(
			U256::from(2).overflowing_pow(U256::from(0x001)),
			(U256::zero(), true)
		);
	}

	#[test]
	pub fn uint256_mul1() {
		assert_eq!(U256::from(1u64) * U256::from(10u64), U256::from(10u64));
	}

	#[test]
	pub fn uint128_add() {
		assert_eq!(
			U128::from_str("fffffffffffffffff").unwrap() + U128::from_str("fffffffffffffffff").unwrap(),
			U128::from_str("1ffffffffffffffffe").unwrap()
		);
	}

	#[test]
	pub fn uint128_add_overflow() {
		assert_eq!(
			U128::from_str("ffffffffffffffffffffffffffffffff").unwrap()
			.overflowing_add(
				U128::from_str("ffffffffffffffffffffffffffffffff").unwrap()
			),
			(U128::from_str("fffffffffffffffffffffffffffffffe").unwrap(), true)
		);
	}

	#[test]
	#[should_panic]
	pub fn uint128_add_overflow_panic() {
		U128::from_str("ffffffffffffffffffffffffffffffff").unwrap()
		+
		U128::from_str("ffffffffffffffffffffffffffffffff").unwrap();
	}

	#[test]
	pub fn uint128_mul() {
		assert_eq!(
			U128::from_str("fffffffff").unwrap() * U128::from_str("fffffffff").unwrap(),
			U128::from_str("ffffffffe000000001").unwrap());
	}

	#[test]
	pub fn uint512_mul() {
		assert_eq!(
			U512::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			*
			U512::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap(),
			U512::from_str("3fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000000000000000001").unwrap()
		);
	}

	#[test]
	pub fn uint256_mul_overflow() {
		assert_eq!(
			U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			.overflowing_mul(
				U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			),
			(U256::from_str("1").unwrap(), true)
			);
	}

	#[test]
	#[should_panic]
	pub fn uint256_mul_overflow_panic() {
		U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
		*
		U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
	}

	#[test]
	pub fn uint256_sub_overflow() {
		assert_eq!(
			U256::from_str("0").unwrap()
			.overflowing_sub(
				U256::from_str("1").unwrap()
			),
			(U256::from_str("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap(), true)
			);
	}

	#[test]
	#[should_panic]
	pub fn uint256_sub_overflow_panic() {
		U256::from_str("0").unwrap()
		-
		U256::from_str("1").unwrap();
	}

	#[ignore]
	#[test]
	pub fn uint256_shl_overflow() {
		assert_eq!(
			U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			.overflowing_shl(4),
			(U256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0").unwrap(), true)
		);
	}

	#[ignore]
	#[test]
	#[should_panic]
	pub fn uint256_shl_overflow2() {
		assert_eq!(
			U256::from_str("0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			.overflowing_shl(4),
			(U256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0").unwrap(), false)
		);
	}

	#[ignore]
	#[test]
	pub fn uint256_shr_overflow() {
		assert_eq!(
			U256::from_str("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			.overflowing_shr(4),
			(U256::from_str("0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap(), true)
		);
	}

	#[ignore]
	#[test]
	pub fn uint256_shr_overflow2() {
		assert_eq!(
			U256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0").unwrap()
			.overflowing_shr(4),
			(U256::from_str("0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap(), false)
		);
	}



	#[test]
	pub fn uint256_mul() {
		assert_eq!(
			U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			*
			U256::from_str("2").unwrap(),
			U256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe").unwrap()
			);
	}

	#[test]
	fn uint256_div() {
		assert_eq!(U256::from(10u64) /  U256::from(1u64), U256::from(10u64));
		assert_eq!(U256::from(10u64) /  U256::from(2u64), U256::from(5u64));
		assert_eq!(U256::from(10u64) /  U256::from(3u64), U256::from(3u64));
	}

	#[test]
	fn uint256_rem() {
		assert_eq!(U256::from(10u64) % U256::from(1u64), U256::from(0u64));
		assert_eq!(U256::from(10u64) % U256::from(3u64), U256::from(1u64));
	}

	#[test]
	fn uint256_from_dec_str() {
		assert_eq!(U256::from_dec_str("10").unwrap(), U256::from(10u64));
		assert_eq!(U256::from_dec_str("1024").unwrap(), U256::from(1024u64));
	}
}

