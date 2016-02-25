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

// Code derived from original work by Andrew Poelstra <apoelstra@wpsoftware.net>

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
//!
//! Implementation of a various large-but-fixed sized unsigned integer types.
//! The functions here are designed to be fast.
//!

use standard::*;
use from_json::*;
use rustc_serialize::hex::ToHex;
use serde;

macro_rules! impl_map_from {
	($thing:ident, $from:ty, $to:ty) => {
		impl From<$from> for $thing {
			fn from(value: $from) -> $thing {
				From::from(value as $to)
			}
		}
	}
}

#[cfg(not(all(feature="x64asm", target_arch = "x86_64")))]
macro_rules! uint_overflowing_add {
	($name:ident, $n_words:expr, $self_expr: expr, $other: expr) => ({
		uint_overflowing_add_reg!($name, $n_words, $self_expr, $other)
	})
}

macro_rules! uint_overflowing_add_reg {
	($name:ident, $n_words:expr, $self_expr: expr, $other: expr) => ({
		let $name(ref me) = $self_expr;
		let $name(ref you) = $other;
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
					overflow = true;
				}
			}
		}
		if b_carry {
			let ret = overflowing!($name(ret).overflowing_add($name(carry)), overflow);
			(ret, overflow)
		} else {
			($name(ret), overflow)
		}
	})
}


#[cfg(all(feature="x64asm", target_arch = "x86_64"))]
macro_rules! uint_overflowing_add {
	(U256, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut result: [u64; 4] = unsafe { mem::uninitialized() };
		let self_t: &[u64; 4] = unsafe { &mem::transmute($self_expr) };
		let other_t: &[u64; 4] = unsafe { &mem::transmute($other) };

		let overflow: u8;
        unsafe {
            asm!("
                adc $9, $0
                adc $10, $1
                adc $11, $2
                adc $12, $3
                setc %al
                "
            : "=r"(result[0]), "=r"(result[1]), "=r"(result[2]), "=r"(result[3]), "={al}"(overflow)
            : "0"(self_t[0]), "1"(self_t[1]), "2"(self_t[2]), "3"(self_t[3]),
			  "mr"(other_t[0]), "mr"(other_t[1]), "mr"(other_t[2]), "mr"(other_t[3])
            :
            :
			);
		}
		(U256(result), overflow != 0)
	});
	($name:ident, $n_words:expr, $self_expr: expr, $other: expr) => (
		uint_overflowing_add_reg!($name, $n_words, $self_expr, $other)
	)
}

#[cfg(not(all(feature="x64asm", target_arch = "x86_64")))]
macro_rules! uint_overflowing_sub {
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let res = overflowing!((!$other).overflowing_add(From::from(1u64)));
		let res = overflowing!($self_expr.overflowing_add(res));
		(res, $self_expr < $other)
	})
}

#[cfg(all(feature="x64asm", target_arch = "x86_64"))]
macro_rules! uint_overflowing_sub {
	(U256, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut result: [u64; 4] = unsafe { mem::uninitialized() };
		let self_t: &[u64; 4] = unsafe { &mem::transmute($self_expr) };
		let other_t: &[u64; 4] = unsafe { &mem::transmute($other) };

		let overflow: u8;
		unsafe {
			asm!("
                sbb $9, $0
                sbb $10, $1
                sbb $11, $2
                sbb $12, $3
                setb %al"
             	: "=r"(result[0]), "=r"(result[1]), "=r"(result[2]), "=r"(result[3]), "={al}"(overflow)
				: "0"(self_t[0]), "1"(self_t[1]), "2"(self_t[2]), "3"(self_t[3]), "mr"(other_t[0]), "mr"(other_t[1]), "mr"(other_t[2]), "mr"(other_t[3])
				:
				:
			);
		}
		(U256(result), overflow != 0)
	});
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let res = overflowing!((!$other).overflowing_add(From::from(1u64)));
		let res = overflowing!($self_expr.overflowing_add(res));
		(res, $self_expr < $other)
	})
}

#[cfg(all(feature="x64asm", target_arch = "x86_64"))]
macro_rules! uint_overflowing_mul {
	(U256, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut result: [u64; 4] = unsafe { mem::uninitialized() };
		let self_t: &[u64; 4] = unsafe { &mem::transmute($self_expr) };
		let other_t: &[u64; 4] = unsafe { &mem::transmute($other) };

		let overflow: u8;
		unsafe {
			asm!("
				mov $5, %rax
				mulq $9
				mov %rax, $0
				mov %rdx, $1

				mov $6, %rax
				mulq $9
				add %rax, $1
				mov %rdx, $2

				mov $5, %rax
				mulq $10
				add %rax, $1
				adc %rdx, $2

				mov $6, %rax
				mulq $10
				add %rax, $2
				mov %rdx, $3

				mov $7, %rax
				mulq $9
				add %rax, $2
				adc %rdx, $3

				mov $5, %rax
				mulq $11
    			add %rax, $2
				adc %rdx, $3

				mov $8, %rax
				mulq $9
				adc %rax, $3
				adc $$0, %rdx
				mov %rdx, %rcx

				mov $7, %rax
				mulq $10
				add %rax, $3
				adc $$0, %rdx
				or %rdx, %rcx

				mov $6, %rax
				mulq $11
				add %rax, $3
				adc $$0, %rdx
				or %rdx, %rcx

				mov $5, %rax
				mulq $12
				add %rax, $3
				adc $$0, %rdx
				or %rdx, %rcx

				jrcxz 2f

				mov $8, %rax
				cmpq $$0, %rax
				sete %cl

				mov $7, %rax
				cmpq $$0, %rax
				sete %dl
				or %dl, %cl

				jrcxz 2f

				mov $3, %rax
				cmpq $$0, %rax
				sete %dl

				mov $2, %rax
				cmpq $$0, %rax
			    sete %bl
			    or %bl, %dl

			    and %dl, %cl

			    2:
			    "
				: /* $0 */ "={r8}"(result[0]), /* $1 */ "={r9}"(result[1]), /* $2 */ "={r10}"(result[2]),
				  /* $3 */ "={r11}"(result[3]), /* $4 */  "={rcx}"(overflow)

				: /* $5 */ "m"(self_t[0]), /* $6 */ "m"(self_t[1]), /* $7 */  "m"(self_t[2]),
				  /* $8 */ "m"(self_t[3]), /* $9 */ "m"(other_t[0]), /* $10 */ "m"(other_t[1]),
				  /* $11 */ "m"(other_t[2]), /* $12 */ "m"(other_t[3])
				: "rax", "rdx"
				:

			);
		}
		(U256(result), overflow > 0)
	});
	($name:ident, $n_words:expr, $self_expr: expr, $other: expr) => (
		uint_overflowing_mul_reg!($name, $n_words, $self_expr, $other)
	)
}

#[cfg(not(all(feature="x64asm", target_arch = "x86_64")))]
macro_rules! uint_overflowing_mul {
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		uint_overflowing_mul_reg!($name, $n_words, $self_expr, $other)
	})
}

macro_rules! uint_overflowing_mul_reg {
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut res = $name::from(0u64);
		let mut overflow = false;
		// TODO: be more efficient about this
		for i in 0..(2 * $n_words) {
			let v = overflowing!($self_expr.overflowing_mul_u32(($other >> (32 * i)).low_u32()), overflow);
			let res2 = overflowing!(v.overflowing_shl(32 * i as u32), overflow);
			res = overflowing!(res.overflowing_add(res2), overflow);
		}
		(res, overflow)
	})
}

macro_rules! overflowing {
	($op: expr, $overflow: expr) => (
		{
			let (overflow_x, overflow_overflow) = $op;
			$overflow |= overflow_overflow;
			overflow_x
		}
	);
	($op: expr) => (
		{
			let (overflow_x, _overflow_overflow) = $op;
			overflow_x
		}
	);
}

macro_rules! panic_on_overflow {
	($name: expr) => {
		if $name {
			panic!("arithmetic operation overflow")
		}
	}
}

/// Large, fixed-length unsigned integer type.
pub trait Uint: Sized + Default + FromStr + From<u64> + FromJson + fmt::Debug + fmt::Display + PartialOrd + Ord + PartialEq + Eq + Hash {

	/// Returns new instance equalling zero.
	fn zero() -> Self;
	/// Returns new instance equalling one.
	fn one() -> Self;

	/// Error type for converting from a decimal string.
	type FromDecStrErr;
	/// Convert from a decimal string.
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
	/// Return if specific bit is set
	fn bit(&self, index: usize) -> bool;
	/// Return single byte
	fn byte(&self, index: usize) -> u8;
	/// Get this Uint as slice of bytes
	fn to_bytes(&self, bytes: &mut[u8]);

	/// Create `Uint(10**n)`
	fn exp10(n: usize) -> Self;
	/// Return eponentation `self**other`. Panic on overflow.
	fn pow(self, other: Self) -> Self;
	/// Return wrapped eponentation `self**other` and flag if there was an overflow
	fn overflowing_pow(self, other: Self) -> (Self, bool);

	/// Add this `Uint` to other returning result and possible overflow
	fn overflowing_add(self, other: Self) -> (Self, bool);

	/// Subtract another `Uint` from this returning result and possible overflow
	fn overflowing_sub(self, other: Self) -> (Self, bool);

	/// Multiple this `Uint` with other returning result and possible overflow
	fn overflowing_mul(self, other: Self) -> (Self, bool);

	/// Divide this `Uint` by other returning result and possible overflow
	fn overflowing_div(self, other: Self) -> (Self, bool);

	/// Returns reminder of division of this `Uint` by other and possible overflow
	fn overflowing_rem(self, other: Self) -> (Self, bool);

	/// Returns negation of this `Uint` and overflow (always true)
	fn overflowing_neg(self) -> (Self, bool);

	/// Shifts this `Uint` and returns overflow
	fn overflowing_shl(self, shift: u32) -> (Self, bool);
}

macro_rules! construct_uint {
	($name:ident, $n_words:expr) => (
		/// Little-endian large integer type
		#[derive(Copy, Clone, Eq, PartialEq)]
		pub struct $name(pub [u64; $n_words]);

		impl Uint for $name {
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

			#[inline]
			fn low_u32(&self) -> u32 {
				let &$name(ref arr) = self;
				arr[0] as u32
			}

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
				(arr[index / 8] >> (((index % 8)) * 8)) as u8
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
			fn exp10(n: usize) -> Self {
				match n {
					0 => Self::from(1u64),
					_ => Self::exp10(n - 1) * Self::from(10u64)
				}
			}

			#[inline]
			fn zero() -> Self {
				From::from(0u64)
			}

			#[inline]
			fn one() -> Self {
				From::from(1u64)
			}

			/// Fast exponentation by squaring
			/// https://en.wikipedia.org/wiki/Exponentiation_by_squaring
			fn pow(self, expon: Self) -> Self {
				if expon == Self::zero() {
					return Self::one()
				}
				let is_even = |x : &Self| x.low_u64() & 1 == 0;

				let u_one = Self::one();
				let u_two = Self::from(2);
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

			/// Fast exponentation by squaring
			/// https://en.wikipedia.org/wiki/Exponentiation_by_squaring
			fn overflowing_pow(self, expon: Self) -> (Self, bool) {
				if expon == Self::zero() {
					return (Self::one(), false)
				}
				let is_even = |x : &Self| x.low_u64() & 1 == 0;

				let u_one = Self::one();
				let u_two = Self::from(2);
				let mut y = u_one;
				let mut n = expon;
				let mut x = self;
				let mut overflow = false;

				while n > u_one {
					if is_even(&n) {
						x = overflowing!(x.overflowing_mul(x), overflow);
						n = n / u_two;
					} else {
						y = overflowing!(x.overflowing_mul(y), overflow);
						x = overflowing!(x.overflowing_mul(x), overflow);
						n = (n - u_one) / u_two;
					}
				}
				let res = overflowing!(x.overflowing_mul(y), overflow);
				(res, overflow)
			}

			/// Optimized instructions
			#[inline(always)]
			fn overflowing_add(self, other: $name) -> ($name, bool) {
				uint_overflowing_add!($name, $n_words, self, other)
			}

			#[inline(always)]
			fn overflowing_sub(self, other: $name) -> ($name, bool) {
				uint_overflowing_sub!($name, $n_words, self, other)
			}

			#[inline(always)]
			fn overflowing_mul(self, other: $name) -> ($name, bool) {
				uint_overflowing_mul!($name, $n_words, self, other)
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

			fn overflowing_shl(self, shift32: u32) -> ($name, bool) {
				let $name(ref original) = self;
				let mut ret = [0u64; $n_words];
				let shift = shift32 as usize;
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
				// Detecting overflow
				let last = $n_words - word_shift - if bit_shift > 0 { 1 } else { 0 };
				let overflow = if bit_shift > 0 {
					(original[last] >> (64 - bit_shift)) > 0
				} else if word_shift > 0 {
					original[last] > 0
				} else {
					false
				};

				for i in last+1..$n_words-1 {
					if original[i] > 0 {
						return ($name(ret), true);
					}
				}
				($name(ret), overflow)
			}
		}

		impl $name {
			/// Multiplication by u32
			fn mul_u32(self, other: u32) -> Self {
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
			fn overflowing_mul_u32(self, other: u32) -> (Self, bool) {
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
				let result = overflowing!(
					$name(ret).overflowing_add($name(carry)),
					overflow
					);
				(result, overflow)
			}
		}

		impl Default for $name {
			fn default() -> Self {
				$name::zero()
			}
		}

		impl serde::Serialize for $name {
			fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
			where S: serde::Serializer {
				let mut hex = "0x".to_owned();
				let mut bytes = [0u8; 8 * $n_words];
				self.to_bytes(&mut bytes);
				let len = cmp::max((self.bits() + 7) / 8, 1);
				hex.push_str(bytes[bytes.len() - len..].to_hex().as_ref());
				serializer.visit_str(hex.as_ref())
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
				match *json {
					Json::String(ref s) => {
						if s.len() >= 2 && &s[0..2] == "0x" {
							FromStr::from_str(&s[2..]).unwrap_or_else(|_| Default::default())
						} else {
							Uint::from_dec_str(s).unwrap_or_else(|_| Default::default())
						}
					},
					Json::U64(u) => From::from(u),
					Json::I64(i) => From::from(i as u64),
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
					ret[pos] += (bytes[i] as u64) << ((rev % 8) * 8);
				}
				$name(ret)
			}
		}

		impl FromStr for $name {
			type Err = FromHexError;

			fn from_str(value: &str) -> Result<$name, Self::Err> {
				let bytes: Vec<u8> = match value.len() % 2 == 0 {
					true => try!(value.from_hex()),
					false => try!(("0".to_owned() + value).from_hex())
				};

				let bytes_ref: &[u8] = &bytes;
				Ok(From::from(bytes_ref))
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
				let (result, overflow) = self.overflowing_sub(other);
				panic_on_overflow!(overflow);
				result
			}
		}

		impl Mul<$name> for $name {
			type Output = $name;

			fn mul(self, other: $name) -> $name {
				let mut res = $name::from(0u64);
				// TODO: be more efficient about this
				for i in 0..(2 * $n_words) {
					let v = self.mul_u32((other >> (32 * i)).low_u32());
					let (r, overflow) = v.overflowing_shl(32 * i as u32);
					panic_on_overflow!(overflow);
					res = res + r;
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
						sub_copy = overflowing!(sub_copy.overflowing_sub(shift_copy));
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

		// TODO: optimise and traitify.

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
				fmt::Display::fmt(self, f)
			}
		}

		impl fmt::Display for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				if *self == $name::zero() {
					return write!(f, "0");
				}

				let mut s = String::new();
				let mut current = *self;
				let ten = $name::from(10);

				while current != $name::zero() {
					s = format!("{}{}", (current % ten).low_u32(), s);
					current = current / ten;
				}

				write!(f, "{}", s)
			}
		}

		impl fmt::LowerHex for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
				let &$name(ref data) = self;
				try!(write!(f, "0x"));
				let mut latch = false;
				for ch in data.iter().rev() {
					for x in 0..16 {
						let nibble = (ch & (15u64 << ((15 - x) * 4) as u64)) >> (((15 - x) * 4) as u64);
						if !latch { latch = nibble != 0 }
						if latch {
							try!(write!(f, "{:x}", nibble));
						}
					}
				}
				Ok(())
			}
		}

		#[cfg_attr(feature="dev", allow(derive_hash_xor_eq))] // We are pretty sure it's ok.
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

/// Constant value of `U256::zero()` that can be used for a reference saving an additional instance creation.
pub const ZERO_U256: U256 = U256([0x00u64; 4]);
/// Constant value of `U256::one()` that can be used for a reference saving an additional instance creation.
pub const ONE_U256: U256 = U256([0x01u64, 0x00u64, 0x00u64, 0x00u64]);

#[cfg(test)]
mod tests {
	use uint::{Uint, U128, U256, U512};
	use std::str::FromStr;

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
	#[cfg_attr(feature="dev", allow(eq_op))]
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
		let sub = overflowing!(incr.overflowing_sub(init));
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
	fn uint256_pow_overflow_panic () {
		U256::from(2).pow(U256::from(0x100));
	}

	#[test]
	fn uint256_overflowing_pow () {
		// assert_eq!(
		// 	U256::from(2).overflowing_pow(U256::from(0xff)),
		// 	(U256::from_str("8000000000000000000000000000000000000000000000000000000000000000").unwrap(), false)
		// );
		assert_eq!(
			U256::from(2).overflowing_pow(U256::from(0x100)),
			(U256::zero(), true)
		);
	}

	#[test]
	pub fn uint256_mul1() {
		assert_eq!(U256::from(1u64) * U256::from(10u64), U256::from(10u64));
	}

	#[test]
	pub fn uint256_overflowing_mul() {
		assert_eq!(
			U256::from_str("100000000000000000000000000000000").unwrap().overflowing_mul(
				U256::from_str("100000000000000000000000000000000").unwrap()
			),
			(U256::zero(), true)
		);
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
	// overflows panic only in debug builds. Running this test with `--release` flag, always fails
	#[ignore]
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

	#[test]
	pub fn uint256_shl_overflow() {
		assert_eq!(
			U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			.overflowing_shl(4),
			(U256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0").unwrap(), true)
		);
	}

	#[test]
	pub fn uint256_shl_overflow_words() {
		assert_eq!(
			U256::from_str("0000000000000001ffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			.overflowing_shl(64),
			(U256::from_str("ffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000").unwrap(), true)
		);
		assert_eq!(
			U256::from_str("0000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			.overflowing_shl(64),
			(U256::from_str("ffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000").unwrap(), false)
		);
	}

	#[test]
	pub fn uint256_shl_overflow_words2() {
		assert_eq!(
			U256::from_str("00000000000000000000000000000001ffffffffffffffffffffffffffffffff").unwrap()
			.overflowing_shl(128),
			(U256::from_str("ffffffffffffffffffffffffffffffff00000000000000000000000000000000").unwrap(), true)
		);
		assert_eq!(
			U256::from_str("00000000000000000000000000000000ffffffffffffffffffffffffffffffff").unwrap()
			.overflowing_shl(128),
			(U256::from_str("ffffffffffffffffffffffffffffffff00000000000000000000000000000000").unwrap(), false)
		);
		assert_eq!(
			U256::from_str("00000000000000000000000000000000ffffffffffffffffffffffffffffffff").unwrap()
			.overflowing_shl(129),
			(U256::from_str("fffffffffffffffffffffffffffffffe00000000000000000000000000000000").unwrap(), true)
		);
	}


	#[test]
	pub fn uint256_shl_overflow2() {
		assert_eq!(
			U256::from_str("0fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			.overflowing_shl(4),
			(U256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0").unwrap(), false)
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

	#[test]
	fn display_uint() {
		let s = "12345678987654321023456789";
		assert_eq!(format!("{}", U256::from_dec_str(s).unwrap()), s);
	}

	#[test]
	fn display_uint_zero() {
		assert_eq!(format!("{}", U256::from(0)), "0");
	}
}

