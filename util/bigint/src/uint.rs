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

//! Big unsigned integer types.
//!
//! Implementation of a various large-but-fixed sized unsigned integer types.
//! The functions here are designed to be fast. There are optional `x86_64`
//! implementations for even more speed, hidden behind the `x64_arithmetic`
//! feature flag.

use std::{mem, fmt};
use std::str::{FromStr};
use std::hash::Hash;
use std::ops::{Shr, Shl, BitAnd, BitOr, BitXor, Not, Div, Rem, Mul, Add, Sub};
use std::cmp::Ordering;
use rustc_serialize::hex::{FromHex, FromHexError};

/// Conversion from decimal string error
#[derive(Debug, PartialEq)]
pub enum FromDecStrErr {
	/// Char not from range 0-9
	InvalidCharacter,
	/// Value does not fit into type
	InvalidLength,
}

macro_rules! impl_map_from {
	($thing:ident, $from:ty, $to:ty) => {
		impl From<$from> for $thing {
			fn from(value: $from) -> $thing {
				From::from(value as $to)
			}
		}
	}
}

#[cfg(not(all(asm_available, target_arch="x86_64")))]
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
		let mut carry = 0u64;

		for i in 0..$n_words {
			let (res1, overflow1) = me[i].overflowing_add(you[i]);
			let (res2, overflow2) = res1.overflowing_add(carry);

			ret[i] = res2;
			carry = overflow1 as u64 + overflow2 as u64;
		}

		($name(ret), carry > 0)
	})
}

#[cfg(all(asm_available, target_arch="x86_64"))]
macro_rules! uint_overflowing_add {
	(U256, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut result: [u64; $n_words] = unsafe { mem::uninitialized() };
		let self_t: &[u64; $n_words] = &$self_expr.0;
		let other_t: &[u64; $n_words] = &$other.0;

		let overflow: u8;
		unsafe {
			asm!("
				add $9, $0
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
	(U512, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut result: [u64; $n_words] = unsafe { mem::uninitialized() };
		let self_t: &[u64; $n_words] = &$self_expr.0;
		let other_t: &[u64; $n_words] = &$other.0;

		let overflow: u8;

		unsafe {
			asm!("
				add $15, $0
				adc $16, $1
				adc $17, $2
				adc $18, $3
				lodsq
				adc $11, %rax
				stosq
				lodsq
				adc $12, %rax
				stosq
				lodsq
				adc $13, %rax
				stosq
				lodsq
				adc $14, %rax
				stosq
				setc %al

				": "=r"(result[0]), "=r"(result[1]), "=r"(result[2]), "=r"(result[3]),

			  "={al}"(overflow) /* $0 - $4 */

            : "{rdi}"(&result[4] as *const u64) /* $5 */
			  "{rsi}"(&other_t[4] as *const u64) /* $6 */
			  "0"(self_t[0]), "1"(self_t[1]), "2"(self_t[2]), "3"(self_t[3]),
		  	  "m"(self_t[4]), "m"(self_t[5]), "m"(self_t[6]), "m"(self_t[7]),
			  /* $7 - $14 */

			  "mr"(other_t[0]), "mr"(other_t[1]), "mr"(other_t[2]), "mr"(other_t[3]),
              "m"(other_t[4]), "m"(other_t[5]), "m"(other_t[6]), "m"(other_t[7]) /* $15 - $22 */
			: "rdi", "rsi"
			:
			);
		}
		(U512(result), overflow != 0)
	});

	($name:ident, $n_words:expr, $self_expr: expr, $other: expr) => (
		uint_overflowing_add_reg!($name, $n_words, $self_expr, $other)
	)
}

#[cfg(not(all(asm_available, target_arch="x86_64")))]
macro_rules! uint_overflowing_sub {
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		uint_overflowing_sub_reg!($name, $n_words, $self_expr, $other)
	})
}

macro_rules! uint_overflowing_sub_reg {
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let $name(ref me) = $self_expr;
		let $name(ref you) = $other;

		let mut ret = [0u64; $n_words];
		let mut carry = 0u64;

		for i in 0..$n_words {
			let (res1, overflow1) = me[i].overflowing_sub(you[i]);
			let (res2, overflow2) = res1.overflowing_sub(carry);

			ret[i] = res2;
			carry = overflow1 as u64 + overflow2 as u64;
		}

		($name(ret), carry > 0)

	})
}

#[cfg(all(asm_available, target_arch="x86_64"))]
macro_rules! uint_overflowing_sub {
	(U256, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut result: [u64; $n_words] = unsafe { mem::uninitialized() };
		let self_t: &[u64; $n_words] = &$self_expr.0;
		let other_t: &[u64; $n_words] = &$other.0;

		let overflow: u8;
		unsafe {
			asm!("
				sub $9, $0
				sbb $10, $1
				sbb $11, $2
				sbb $12, $3
				setb %al
				"
				: "=r"(result[0]), "=r"(result[1]), "=r"(result[2]), "=r"(result[3]), "={al}"(overflow)
				: "0"(self_t[0]), "1"(self_t[1]), "2"(self_t[2]), "3"(self_t[3]), "mr"(other_t[0]), "mr"(other_t[1]), "mr"(other_t[2]), "mr"(other_t[3])
				:
				:
			);
		}
		(U256(result), overflow != 0)
	});
	(U512, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut result: [u64; $n_words] = unsafe { mem::uninitialized() };
		let self_t: &[u64; $n_words] = &$self_expr.0;
		let other_t: &[u64; $n_words] = &$other.0;

		let overflow: u8;

		unsafe {
			asm!("
				sub $15, $0
				sbb $16, $1
				sbb $17, $2
				sbb $18, $3
				lodsq
				sbb $19, %rax
				stosq
				lodsq
				sbb $20, %rax
				stosq
				lodsq
				sbb $21, %rax
				stosq
				lodsq
				sbb $22, %rax
				stosq
				setb %al
				"
			: "=r"(result[0]), "=r"(result[1]), "=r"(result[2]), "=r"(result[3]),

			  "={al}"(overflow) /* $0 - $4 */

			: "{rdi}"(&result[4] as *const u64) /* $5 */
		 	 "{rsi}"(&self_t[4] as *const u64) /* $6 */
			  "0"(self_t[0]), "1"(self_t[1]), "2"(self_t[2]), "3"(self_t[3]),
			  "m"(self_t[4]), "m"(self_t[5]), "m"(self_t[6]), "m"(self_t[7]),
			  /* $7 - $14 */

			  "m"(other_t[0]), "m"(other_t[1]), "m"(other_t[2]), "m"(other_t[3]),
			  "m"(other_t[4]), "m"(other_t[5]), "m"(other_t[6]), "m"(other_t[7]) /* $15 - $22 */
			: "rdi", "rsi"
			:
			);
		}
		(U512(result), overflow != 0)
	});
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		uint_overflowing_sub_reg!($name, $n_words, $self_expr, $other)
	})
}

#[cfg(all(asm_available, target_arch="x86_64"))]
macro_rules! uint_overflowing_mul {
	(U256, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let mut result: [u64; $n_words] = unsafe { mem::uninitialized() };
		let self_t: &[u64; $n_words] = &$self_expr.0;
		let other_t: &[u64; $n_words] = &$other.0;

		let overflow: u64;
		unsafe {
			asm!("
				mov $5, %rax
				mulq $9
				mov %rax, $0
				mov %rdx, $1

				mov $5, %rax
				mulq $10
				add %rax, $1
				adc $$0, %rdx
				mov %rdx, $2

				mov $5, %rax
				mulq $11
				add %rax, $2
				adc $$0, %rdx
				mov %rdx, $3

				mov $5, %rax
				mulq $12
				add %rax, $3
				adc $$0, %rdx
				mov %rdx, %rcx

				mov $6, %rax
				mulq $9
				add %rax, $1
				adc %rdx, $2
				adc $$0, $3
				adc $$0, %rcx

				mov $6, %rax
				mulq $10
				add %rax, $2
				adc %rdx, $3
				adc $$0, %rcx
				adc $$0, $3
				adc $$0, %rcx

				mov $6, %rax
				mulq $11
				add %rax, $3
				adc $$0, %rdx
				or %rdx, %rcx

				mov $7, %rax
				mulq $9
				add %rax, $2
				adc %rdx, $3
				adc $$0, %rcx

				mov $7, %rax
				mulq $10
				add %rax, $3
				adc $$0, %rdx
				or %rdx, %rcx

				mov $8, %rax
				mulq $9
				add %rax, $3
				or %rdx, %rcx

				cmpq $$0, %rcx
				jne 2f

				mov $8, %rcx
				jrcxz 12f

				mov $12, %rcx
				mov $11, %rax
				or %rax, %rcx
				mov $10, %rax
				or %rax, %rcx
				jmp 2f

				12:
				mov $12, %rcx
				jrcxz 11f

				mov $7, %rcx
				mov $6, %rax
				or %rax, %rcx

				cmpq $$0, %rcx
				jne 2f

				11:
				mov $11, %rcx
				jrcxz 2f
				mov $7, %rcx

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

#[cfg(not(all(asm_available, target_arch="x86_64")))]
macro_rules! uint_overflowing_mul {
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		uint_overflowing_mul_reg!($name, $n_words, $self_expr, $other)
	})
}

macro_rules! uint_overflowing_mul_reg {
	($name:ident, $n_words: expr, $self_expr: expr, $other: expr) => ({
		let $name(ref me) = $self_expr;
		let $name(ref you) = $other;
		let mut ret = [0u64; 2*$n_words];

		for i in 0..$n_words {
			if you[i] == 0 {
				continue;
			}

			let mut carry2 = 0u64;
			let (b_u, b_l) = split(you[i]);

			for j in 0..$n_words {
				if me[j] == 0 && carry2 == 0 {
					continue;
				}

				let a = split(me[j]);

				// multiply parts
				let (c_l, overflow_l) = mul_u32(a, b_l, ret[i + j]);
				let (c_u, overflow_u) = mul_u32(a, b_u, c_l >> 32);
				ret[i + j] = (c_l & 0xFFFFFFFF) + (c_u << 32);

				// No overflow here
				let res = (c_u >> 32) + (overflow_u << 32);
				// possible overflows
				let (res, o1) = res.overflowing_add(overflow_l);
				let (res, o2) = res.overflowing_add(carry2);
				let (res, o3) = res.overflowing_add(ret[i + j + 1]);
				ret[i + j + 1] = res;

				// Only single overflow possible there
				carry2 = (o1 | o2 | o3) as u64;
			}
		}

		let mut res = [0u64; $n_words];
		let mut overflow = false;
		for i in 0..$n_words {
			res[i] = ret[i];
		}

		for i in $n_words..2*$n_words {
			overflow |= ret[i] != 0;
		}

		($name(res), overflow)
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

#[inline(always)]
fn mul_u32(a: (u64, u64), b: u64, carry: u64) -> (u64, u64) {
	let upper = b * a.0;
	let lower = b * a.1;

	let (res1, overflow1) = lower.overflowing_add(upper << 32);
	let (res2, overflow2) = res1.overflowing_add(carry);

	let carry = (upper >> 32) + overflow1 as u64 + overflow2 as u64;
	(res2, carry)
}

#[inline(always)]
fn split(a: u64) -> (u64, u64) {
	(a >> 32, a & 0xFFFFFFFF)
}

/// Large, fixed-length unsigned integer type.
pub trait Uint: Sized + Default + FromStr + From<u64> + fmt::Debug + fmt::Display + PartialOrd + Ord + PartialEq + Eq + Hash {

	/// Returns new instance equalling zero.
	fn zero() -> Self;
	/// Returns new instance equalling one.
	fn one() -> Self;
	/// Returns the largest value that can be represented by this integer type.
	fn max_value() -> Self;

	/// Convert from a decimal string.
	fn from_dec_str(value: &str) -> Result<Self, FromDecStrErr>;

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
	/// Convert U256 to the sequence of bytes with a big endian
	fn to_big_endian(&self, bytes: &mut[u8]);
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

	/// Returns
	fn is_zero(&self) -> bool;
}

macro_rules! construct_uint {
	($name:ident, $n_words:expr) => (
		/// Little-endian large integer type
		#[repr(C)]
		#[derive(Copy, Clone, Eq, PartialEq, Hash)]
		pub struct $name(pub [u64; $n_words]);

		impl Uint for $name {

			fn from_dec_str(value: &str) -> Result<Self, FromDecStrErr> {
				if !value.bytes().all(|b| b >= 48 && b <= 57) {
					return Err(FromDecStrErr::InvalidCharacter)
				}

				let mut res = Self::default();
				for b in value.bytes().map(|b| b - 48) {
					let (r, overflow) = res.overflowing_mul_u32(10);
					if overflow {
						return Err(FromDecStrErr::InvalidLength);
					}
					let (r, overflow) = r.overflowing_add(b.into());
					if overflow {
						return Err(FromDecStrErr::InvalidLength);
					}
					res = r;
				}
				Ok(res)
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

			#[inline]
			fn is_zero(&self) -> bool {
				let &$name(ref arr) = self;
				for i in 0..$n_words { if arr[i] != 0 { return false; } }
				return true;
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

			#[cfg(any(
				target_arch = "arm",
				target_arch = "mips",
				target_arch = "powerpc",
				target_arch = "x86",
				target_arch = "x86_64",
				target_arch = "aarch64",
				target_arch = "powerpc64"))]
			#[inline]
			fn to_big_endian(&self, bytes: &mut[u8]) {
				debug_assert!($n_words * 8 == bytes.len());
				let &$name(ref arr) = self;
				unsafe {
					let mut out: *mut u64 = mem::transmute(bytes.as_mut_ptr());
					out = out.offset($n_words);
					for i in 0..$n_words {
						out = out.offset(-1);
						*out = arr[i].swap_bytes();
					}
				}
			}

			#[cfg(not(any(
				target_arch = "arm",
				target_arch = "mips",
				target_arch = "powerpc",
				target_arch = "x86",
				target_arch = "x86_64",
				target_arch = "aarch64",
				target_arch = "powerpc64")))]
			#[inline]
			fn to_big_endian(&self, bytes: &mut[u8]) {
				debug_assert!($n_words * 8 == bytes.len());
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
					_ => Self::exp10(n - 1).mul_u32(10)
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

			#[inline]
			fn max_value() -> Self {
				let mut result = [0; $n_words];
				for i in 0..$n_words {
					result[i] = u64::max_value();
				}
				$name(result)
			}

			/// Fast exponentation by squaring
			/// https://en.wikipedia.org/wiki/Exponentiation_by_squaring
			fn pow(self, expon: Self) -> Self {
				if expon.is_zero() {
					return Self::one()
				}
				let is_even = |x : &Self| x.low_u64() & 1 == 0;

				let u_one = Self::one();
				let mut y = u_one;
				let mut n = expon;
				let mut x = self;
				while n > u_one {
					if is_even(&n) {
						x = x * x;
						n = n >> 1;
					} else {
						y = x * y;
						x = x * x;
						// to reduce odd number by 1 we should just clear the last bit
						n.0[$n_words-1] = n.0[$n_words-1] & ((!0u64)>>1);
						n = n >> 1;
					}
				}
				x * y
			}

			/// Fast exponentation by squaring
			/// https://en.wikipedia.org/wiki/Exponentiation_by_squaring
			fn overflowing_pow(self, expon: Self) -> (Self, bool) {
				if expon.is_zero() { return (Self::one(), false) }

				let is_even = |x : &Self| x.low_u64() & 1 == 0;

				let u_one = Self::one();
				let mut y = u_one;
				let mut n = expon;
				let mut x = self;
				let mut overflow = false;

				while n > u_one {
					if is_even(&n) {
						x = overflowing!(x.overflowing_mul(x), overflow);
						n = n >> 1;
					} else {
						y = overflowing!(x.overflowing_mul(y), overflow);
						x = overflowing!(x.overflowing_mul(x), overflow);
						n = (n - u_one) >> 1;
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
		}

		impl $name {
			/// Multiplication by u32
			#[allow(dead_code)] // not used when multiplied with inline assembly
			fn mul_u32(self, other: u32) -> Self {
				let (ret, overflow) = self.overflowing_mul_u32(other);
				panic_on_overflow!(overflow);
				ret
			}

			/// Overflowing multiplication by u32
			#[allow(dead_code)] // not used when multiplied with inline assembly
			fn overflowing_mul_u32(self, other: u32) -> (Self, bool) {
				let $name(ref arr) = self;
				let mut ret = [0u64; $n_words];
				let mut carry = 0;
				let o = other as u64;

				for i in 0..$n_words {
					let (res, carry2) = mul_u32(split(arr[i]), o, carry);
					ret[i] = res;
					carry = carry2;
				}

				($name(ret), carry > 0)
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
				let (result, overflow) = self.overflowing_add(other);
				panic_on_overflow!(overflow);
				result
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
				let (result, overflow) = self.overflowing_mul(other);
				panic_on_overflow!(overflow);
				result
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

				// shift
				for i in word_shift..$n_words {
					ret[i] = original[i - word_shift] << bit_shift;
				}
				// carry
				if bit_shift > 0 {
					for i in word_shift+1..$n_words {
						ret[i] += original[i - 1 - word_shift] >> (64 - bit_shift);
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

				// shift
				for i in word_shift..$n_words {
					ret[i - word_shift] = original[i] >> bit_shift;
				}

				// Carry
				if bit_shift > 0 {
					for i in word_shift+1..$n_words {
						ret[i - word_shift - 1] += original[i] << (64 - bit_shift);
					}
				}

				$name(ret)
			}
		}

		impl Ord for $name {
			fn cmp(&self, other: &$name) -> Ordering {
				let &$name(ref me) = self;
				let &$name(ref you) = other;
				let mut i = $n_words;
				while i > 0 {
					i -= 1;
					if me[i] < you[i] { return Ordering::Less; }
					if me[i] > you[i] { return Ordering::Greater; }
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
				if self.is_zero() {
					return write!(f, "0");
				}

				let mut s = String::new();
				let mut current = *self;
				let ten = $name::from(10);

				while !current.is_zero() {
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

		impl From<&'static str> for $name {
			fn from(s: &'static str) -> Self {
				s.parse().unwrap()
			}
		}
	);
}

construct_uint!(U512, 8);
construct_uint!(U256, 4);
construct_uint!(U128, 2);

impl U256 {
	/// Multiplies two 256-bit integers to produce full 512-bit integer
	/// No overflow possible
	#[cfg(all(asm_available, target_arch="x86_64"))]
	pub fn full_mul(self, other: U256) -> U512 {
		let self_t: &[u64; 4] = &self.0;
		let other_t: &[u64; 4] = &other.0;
		let mut result: [u64; 8] = unsafe { mem::uninitialized() };
		unsafe {
			asm!("
				mov $8, %rax
				mulq $12
				mov %rax, $0
				mov %rdx, $1

				mov $8, %rax
				mulq $13
				add %rax, $1
				adc $$0, %rdx
				mov %rdx, $2

				mov $8, %rax
				mulq $14
				add %rax, $2
				adc $$0, %rdx
				mov %rdx, $3

				mov $8, %rax
				mulq $15
				add %rax, $3
				adc $$0, %rdx
				mov %rdx, $4

				mov $9, %rax
				mulq $12
				add %rax, $1
				adc %rdx, $2
				adc $$0, $3
				adc $$0, $4
				xor $5, $5
				adc $$0, $5
				xor $6, $6
				adc $$0, $6
				xor $7, $7
				adc $$0, $7

				mov $9, %rax
				mulq $13
				add %rax, $2
				adc %rdx, $3
				adc $$0, $4
				adc $$0, $5
				adc $$0, $6
				adc $$0, $7

				mov $9, %rax
				mulq $14
				add %rax, $3
				adc %rdx, $4
				adc $$0, $5
				adc $$0, $6
				adc $$0, $7

				mov $9, %rax
				mulq $15
				add %rax, $4
				adc %rdx, $5
				adc $$0, $6
				adc $$0, $7

				mov $10, %rax
				mulq $12
				add %rax, $2
				adc %rdx, $3
				adc $$0, $4
				adc $$0, $5
				adc $$0, $6
				adc $$0, $7

				mov $10, %rax
				mulq $13
				add %rax, $3
				adc %rdx, $4
				adc $$0, $5
				adc $$0, $6
				adc $$0, $7

				mov $10, %rax
				mulq $14
				add %rax, $4
				adc %rdx, $5
				adc $$0, $6
				adc $$0, $7

				mov $10, %rax
				mulq $15
				add %rax, $5
				adc %rdx, $6
				adc $$0, $7

				mov $11, %rax
				mulq $12
				add %rax, $3
				adc %rdx, $4
				adc $$0, $5
				adc $$0, $6
				adc $$0, $7

				mov $11, %rax
				mulq $13
				add %rax, $4
				adc %rdx, $5
				adc $$0, $6
				adc $$0, $7

				mov $11, %rax
				mulq $14
				add %rax, $5
				adc %rdx, $6
				adc $$0, $7

				mov $11, %rax
				mulq $15
				add %rax, $6
				adc %rdx, $7
				"
            : /* $0 */ "={r8}"(result[0]), /* $1 */ "={r9}"(result[1]), /* $2 */ "={r10}"(result[2]),
			  /* $3 */ "={r11}"(result[3]), /* $4 */ "={r12}"(result[4]), /* $5 */ "={r13}"(result[5]),
			  /* $6 */ "={r14}"(result[6]), /* $7 */ "={r15}"(result[7])

            : /* $8 */ "m"(self_t[0]), /* $9 */ "m"(self_t[1]), /* $10 */  "m"(self_t[2]),
			  /* $11 */ "m"(self_t[3]), /* $12 */ "m"(other_t[0]), /* $13 */ "m"(other_t[1]),
			  /* $14 */ "m"(other_t[2]), /* $15 */ "m"(other_t[3])
			: "rax", "rdx"
			:
			);
		}
		U512(result)
	}

	/// Multiplies two 256-bit integers to produce full 512-bit integer
	/// No overflow possible
	#[cfg(not(all(asm_available, target_arch="x86_64")))]
	pub fn full_mul(self, other: U256) -> U512 {
		let U256(ref me) = self;
		let U256(ref you) = other;
		let mut ret = [0u64; 8];

		for i in 0..4 {
			if you[i] == 0 {
				continue;
			}

			let mut carry2 = 0u64;
			let (b_u, b_l) = split(you[i]);

			for j in 0..4 {
				if me[j] == 0 && carry2 == 0 {
					continue;
				}

				let a = split(me[j]);

				// multiply parts
				let (c_l, overflow_l) = mul_u32(a, b_l, ret[i + j]);
				let (c_u, overflow_u) = mul_u32(a, b_u, c_l >> 32);
				ret[i + j] = (c_l & 0xFFFFFFFF) + (c_u << 32);

				// No overflow here
				let res = (c_u >> 32) + (overflow_u << 32);
				// possible overflows
				let (res, o1) = res.overflowing_add(overflow_l);
				let (res, o2) = res.overflowing_add(carry2);
				let (res, o3) = res.overflowing_add(ret[i + j + 1]);
				ret[i + j + 1] = res;

				// Only single overflow possible there
				carry2 = (o1 | o2 | o3) as u64;
			}
		}

		U512(ret)
	}
}

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

impl<'a> From<&'a U256> for U512 {
	fn from(value: &'a U256) -> U512 {
		let U256(ref arr) = *value;
		let mut ret = [0; 8];
		ret[0] = arr[0];
		ret[1] = arr[1];
		ret[2] = arr[2];
		ret[3] = arr[3];
		U512(ret)
	}
}

impl<'a> From<&'a U512> for U256 {
	fn from(value: &'a U512) -> U256 {
		let U512(ref arr) = *value;
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

known_heap_size!(0, U128, U256);

#[cfg(test)]
mod tests {
	use uint::{Uint, U128, U256, U512};
	use std::str::FromStr;
	use super::FromDecStrErr;

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
		uint.to_big_endian(&mut bytes);
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

		let a = U256::from_str("ff000000000000000000000000000000000000000000000000000000000000d1").unwrap();
		let b = U256::from_str("00ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff2e").unwrap();
		println!("{:x}", a);
		println!("{:x}", b);
		assert_eq!(!a, b);
		assert_eq!(a, !b);
	}

	#[test]
	pub fn uint256_simple_mul() {
		let a = U256::from_str("10000000000000000").unwrap();
		let b = U256::from_str("10000000000000000").unwrap();

		let c = U256::from_str("100000000000000000000000000000000").unwrap();
		println!("Multiplying");
		let result = a.overflowing_mul(b);
		println!("Got result");
		assert_eq!(result, (c, false))
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
	pub fn uint256_mul2() {
		let a = U512::from_str("10000000000000000fffffffffffffffe").unwrap();
		let b = U512::from_str("ffffffffffffffffffffffffffffffff").unwrap();

		assert_eq!(a * b, U512::from_str("10000000000000000fffffffffffffffcffffffffffffffff0000000000000002").unwrap());
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
	pub fn uint256_shl() {
		assert_eq!(
			U256::from_str("7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			<< 4,
			U256::from_str("fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0").unwrap()
		);
	}

	#[test]
	pub fn uint256_shl_words() {
		assert_eq!(
			U256::from_str("0000000000000001ffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			<< 64,
			U256::from_str("ffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000").unwrap()
		);
		assert_eq!(
			U256::from_str("0000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff").unwrap()
			<< 64,
			U256::from_str("ffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000").unwrap()
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
		assert_eq!(U256::from_dec_str("115792089237316195423570985008687907853269984665640564039457584007913129639936"), Err(FromDecStrErr::InvalidLength));
		assert_eq!(U256::from_dec_str("0x11"), Err(FromDecStrErr::InvalidCharacter));
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

    #[test]
    fn u512_multi_adds() {
		let (result, _) = U512([0, 0, 0, 0, 0, 0, 0, 0]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 0, 0]));
		assert_eq!(result, U512([0, 0, 0, 0, 0, 0, 0, 0]));

		let (result, _) = U512([1, 0, 0, 0, 0, 0, 0, 1]).overflowing_add(U512([1, 0, 0, 0, 0, 0, 0, 1]));
		assert_eq!(result, U512([2, 0, 0, 0, 0, 0, 0, 2]));

		let (result, _) = U512([0, 0, 0, 0, 0, 0, 0, 1]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 0, 1]));
		assert_eq!(result, U512([0, 0, 0, 0, 0, 0, 0, 2]));

		let (result, _) = U512([0, 0, 0, 0, 0, 0, 2, 1]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 3, 1]));
		assert_eq!(result, U512([0, 0, 0, 0, 0, 0, 5, 2]));

		let (result, _) = U512([1, 2, 3, 4, 5, 6, 7, 8]).overflowing_add(U512([9, 10, 11, 12, 13, 14, 15, 16]));
		assert_eq!(result, U512([10, 12, 14, 16, 18, 20, 22, 24]));

		let (_, overflow) = U512([0, 0, 0, 0, 0, 0, 2, 1]).overflowing_add(U512([0, 0, 0, 0, 0, 0, 3, 1]));
		assert!(!overflow);

		let (_, overflow) = U512([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
			.overflowing_add(U512([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
		assert!(overflow);

		let (_, overflow) = U512([0, 0, 0, 0, 0, 0, 0, ::std::u64::MAX])
			.overflowing_add(U512([0, 0, 0, 0, 0, 0, 0, ::std::u64::MAX]));
        assert!(overflow);

		let (_, overflow) = U512([0, 0, 0, 0, 0, 0, 0, ::std::u64::MAX])
			.overflowing_add(U512([0, 0, 0, 0, 0, 0, 0, 0]));
		assert!(!overflow);
	}

    #[test]
    fn u256_multi_adds() {
        let (result, _) = U256([0, 0, 0, 0]).overflowing_add(U256([0, 0, 0, 0]));
        assert_eq!(result, U256([0, 0, 0, 0]));

        let (result, _) = U256([0, 0, 0, 1]).overflowing_add(U256([0, 0, 0, 1]));
        assert_eq!(result, U256([0, 0, 0, 2]));

        let (result, overflow) = U256([0, 0, 2, 1]).overflowing_add(U256([0, 0, 3, 1]));
        assert_eq!(result, U256([0, 0, 5, 2]));
        assert!(!overflow);

        let (_, overflow) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
			.overflowing_add(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
        assert!(overflow);

        let (_, overflow) = U256([0, 0, 0, ::std::u64::MAX]).overflowing_add(U256([0, 0, 0, ::std::u64::MAX]));
        assert!(overflow);
    }


	#[test]
	fn u256_multi_subs() {
		let (result, _) = U256([0, 0, 0, 0]).overflowing_sub(U256([0, 0, 0, 0]));
		assert_eq!(result, U256([0, 0, 0, 0]));

		let (result, _) = U256([0, 0, 0, 1]).overflowing_sub(U256([0, 0, 0, 1]));
		assert_eq!(result, U256([0, 0, 0, 0]));

		let (_, overflow) = U256([0, 0, 2, 1]).overflowing_sub(U256([0, 0, 3, 1]));
		assert!(overflow);

		let (result, overflow) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
								.overflowing_sub(U256([::std::u64::MAX/2, ::std::u64::MAX/2, ::std::u64::MAX/2, ::std::u64::MAX/2]));
		assert!(!overflow);
		assert_eq!(U256([::std::u64::MAX/2+1, ::std::u64::MAX/2+1, ::std::u64::MAX/2+1, ::std::u64::MAX/2+1]), result);

		let (result, overflow) = U256([0, 0, 0, 1]).overflowing_sub(U256([0, 0, 1, 0]));
		assert!(!overflow);
		assert_eq!(U256([0, 0, ::std::u64::MAX, 0]), result);

		let (result, overflow) = U256([0, 0, 0, 1]).overflowing_sub(U256([1, 0, 0, 0]));
		assert!(!overflow);
		assert_eq!(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]), result);
	}

	#[test]
	fn u512_multi_subs() {
		let (result, _) = U512([0, 0, 0, 0, 0, 0, 0, 0]).overflowing_sub(U512([0, 0, 0, 0, 0, 0, 0, 0]));
		assert_eq!(result, U512([0, 0, 0, 0, 0, 0, 0, 0]));

		let (result, _) = U512([10, 9, 8, 7, 6, 5, 4, 3]).overflowing_sub(U512([9, 8, 7, 6, 5, 4, 3, 2]));
		assert_eq!(result, U512([1, 1, 1, 1, 1, 1, 1, 1]));

		let (_, overflow) = U512([10, 9, 8, 7, 6, 5, 4, 3]).overflowing_sub(U512([9, 8, 7, 6, 5, 4, 3, 2]));
		assert!(!overflow);

		let (_, overflow) = U512([9, 8, 7, 6, 5, 4, 3, 2]).overflowing_sub(U512([10, 9, 8, 7, 6, 5, 4, 3]));
		assert!(overflow);
	}

	#[test]
	fn u256_multi_carry_all() {
		let (result, _) = U256([::std::u64::MAX, 0, 0, 0]).overflowing_mul(U256([::std::u64::MAX, 0, 0, 0]));
		assert_eq!(U256([1, ::std::u64::MAX-1, 0, 0]), result);

		let (result, _) = U256([0, ::std::u64::MAX, 0, 0]).overflowing_mul(U256([::std::u64::MAX, 0, 0, 0]));
		assert_eq!(U256([0, 1, ::std::u64::MAX-1, 0]), result);

		let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]).overflowing_mul(U256([::std::u64::MAX, 0, 0, 0]));
		assert_eq!(U256([1, ::std::u64::MAX, ::std::u64::MAX-1, 0]), result);

		let (result, _) = U256([::std::u64::MAX, 0, 0, 0]).overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
		assert_eq!(U256([1, ::std::u64::MAX, ::std::u64::MAX-1, 0]), result);

		let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0])
			.overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
		assert_eq!(U256([1, 0, ::std::u64::MAX-1, ::std::u64::MAX]), result);

		let (result, _) = U256([::std::u64::MAX, 0, 0, 0]).overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
		assert_eq!(U256([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1]), result);

		let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]).overflowing_mul(U256([::std::u64::MAX, 0, 0, 0]));
		assert_eq!(U256([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1]), result);

		let (result, _) = U256([::std::u64::MAX, 0, 0, 0]).overflowing_mul(
			U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
		assert_eq!(U256([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]), result);

		let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
			.overflowing_mul(U256([::std::u64::MAX, 0, 0, 0]));
		assert_eq!(U256([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]), result);

		let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0])
			.overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
		assert_eq!(U256([1, 0, ::std::u64::MAX, ::std::u64::MAX-1]), result);

		let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0])
			.overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
		assert_eq!(U256([1, 0, ::std::u64::MAX, ::std::u64::MAX-1]), result);

		let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
			.overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
		assert_eq!(U256([1, 0, ::std::u64::MAX, ::std::u64::MAX]), result);

		let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0])
			.overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
		assert_eq!(U256([1, 0, ::std::u64::MAX, ::std::u64::MAX]), result);

		let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0])
			.overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
		assert_eq!(U256([1, 0, 0, ::std::u64::MAX-1]), result);

		let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0])
			.overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
		assert_eq!(U256([1, 0, 0, ::std::u64::MAX]), result);

		let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
			.overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
		assert_eq!(U256([1, 0, 0, ::std::u64::MAX]), result);

		let (result, _) = U256([0, 0, 0, ::std::u64::MAX]).overflowing_mul(U256([0, 0, 0, ::std::u64::MAX]));
		assert_eq!(U256([0, 0, 0, 0]), result);

		let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, ::std::u64::MAX]));
		assert_eq!(U256([0, 0, 0, ::std::u64::MAX]), result);

		let (result, _) = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX])
			.overflowing_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
		assert_eq!(U256([1, 0, 0, 0]), result);
	}

	#[test]
	fn u256_multi_muls() {
		let (result, _) = U256([0, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, 0]));
		assert_eq!(U256([0, 0, 0, 0]), result);

		let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([1, 0, 0, 0]));
		assert_eq!(U256([1, 0, 0, 0]), result);

		let (result, _) = U256([5, 0, 0, 0]).overflowing_mul(U256([5, 0, 0, 0]));
		assert_eq!(U256([25, 0, 0, 0]), result);

		let (result, _) = U256([0, 5, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
		assert_eq!(U256([0, 0, 25, 0]), result);

		let (result, _) = U256([0, 0, 0, 1]).overflowing_mul(U256([1, 0, 0, 0]));
		assert_eq!(U256([0, 0, 0, 1]), result);

		let (result, _) = U256([0, 0, 0, 5]).overflowing_mul(U256([2, 0, 0, 0]));
		assert_eq!(U256([0, 0, 0, 10]), result);

		let (result, _) = U256([0, 0, 1, 0]).overflowing_mul(U256([0, 5, 0, 0]));
		assert_eq!(U256([0, 0, 0, 5]), result);

		let (result, _) = U256([0, 0, 8, 0]).overflowing_mul(U256([0, 0, 7, 0]));
		assert_eq!(U256([0, 0, 0, 0]), result);

		let (result, _) = U256([2, 0, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
		assert_eq!(U256([0, 10, 0, 0]), result);

		let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, ::std::u64::MAX]));
		assert_eq!(U256([0, 0, 0, ::std::u64::MAX]), result);
	}

    #[test]
    fn u256_multi_muls_overflow() {
		let (_, overflow) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, 0]));
		assert!(!overflow);

		let (_, overflow) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, ::std::u64::MAX]));
		assert!(!overflow);

		let (_, overflow) = U256([0, 1, 0, 0]).overflowing_mul(U256([0, 0, 0, ::std::u64::MAX]));
		assert!(overflow);

		let (_, overflow) = U256([0, 1, 0, 0]).overflowing_mul(U256([0, 1, 0, 0]));
		assert!(!overflow);

		let (_, overflow) = U256([0, 1, 0, ::std::u64::MAX]).overflowing_mul(U256([0, 1, 0, ::std::u64::MAX]));
		assert!(overflow);

		let (_, overflow) = U256([0, ::std::u64::MAX, 0, 0]).overflowing_mul(U256([0, ::std::u64::MAX, 0, 0]));
		assert!(!overflow);

		let (_, overflow) = U256([1, 0, 0, 0]).overflowing_mul(U256([10, 0, 0, 0]));
		assert!(!overflow);

		let (_, overflow) = U256([2, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, ::std::u64::MAX / 2]));
		assert!(!overflow);

		let (_, overflow) = U256([0, 0, 8, 0]).overflowing_mul(U256([0, 0, 7, 0]));
		assert!(overflow);
    }

	#[test]
	fn big_endian() {
		let source = U256([1, 0, 0, 0]);
		let mut target = vec![0u8; 32];

		assert_eq!(source, U256::from(1));

		source.to_big_endian(&mut target);
		assert_eq!(
			vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
				0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8],
			target);

		let source = U256([512, 0, 0, 0]);
		let mut target = vec![0u8; 32];

		source.to_big_endian(&mut target);
		assert_eq!(
			vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
				0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8],
			target);

		let source = U256([0, 512, 0, 0]);
		let mut target = vec![0u8; 32];

		source.to_big_endian(&mut target);
		assert_eq!(
			vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
				0u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
			target);

		let source = U256::from_str("0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20").unwrap();
		source.to_big_endian(&mut target);
		assert_eq!(
			vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11,
				0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20],
			target);
	}

	#[test]
	#[cfg_attr(feature="dev", allow(cyclomatic_complexity))]
	fn u256_multi_full_mul() {
		let result = U256([0, 0, 0, 0]).full_mul(U256([0, 0, 0, 0]));
		assert_eq!(U512([0, 0, 0, 0, 0, 0, 0, 0]), result);

		let result = U256([1, 0, 0, 0]).full_mul(U256([1, 0, 0, 0]));
		assert_eq!(U512([1, 0, 0, 0, 0, 0, 0, 0]), result);

		let result = U256([5, 0, 0, 0]).full_mul(U256([5, 0, 0, 0]));
		assert_eq!(U512([25, 0, 0, 0, 0, 0, 0, 0]), result);

		let result = U256([0, 5, 0, 0]).full_mul(U256([0, 5, 0, 0]));
		assert_eq!(U512([0, 0, 25, 0, 0, 0, 0, 0]), result);

		let result = U256([0, 0, 0, 4]).full_mul(U256([4, 0, 0, 0]));
		assert_eq!(U512([0, 0, 0, 16, 0, 0, 0, 0]), result);

		let result = U256([0, 0, 0, 5]).full_mul(U256([2, 0, 0, 0]));
		assert_eq!(U512([0, 0, 0, 10, 0, 0, 0, 0]), result);

		let result = U256([0, 0, 2, 0]).full_mul(U256([0, 5, 0, 0]));
		assert_eq!(U512([0, 0, 0, 10, 0, 0, 0, 0]), result);

		let result = U256([0, 3, 0, 0]).full_mul(U256([0, 0, 3, 0]));
		assert_eq!(U512([0, 0, 0, 9, 0, 0, 0, 0]), result);

		let result = U256([0, 0, 8, 0]).full_mul(U256([0, 0, 6, 0]));
		assert_eq!(U512([0, 0, 0, 0, 48, 0, 0, 0]), result);

		let result = U256([9, 0, 0, 0]).full_mul(U256([0, 3, 0, 0]));
		assert_eq!(U512([0, 27, 0, 0, 0, 0, 0, 0]), result);

		let result = U256([::std::u64::MAX, 0, 0, 0]).full_mul(U256([::std::u64::MAX, 0, 0, 0]));
		assert_eq!(U512([1, ::std::u64::MAX-1, 0, 0, 0, 0, 0, 0]), result);

		let result = U256([0, ::std::u64::MAX, 0, 0]).full_mul(U256([::std::u64::MAX, 0, 0, 0]));
		assert_eq!(U512([0, 1, ::std::u64::MAX-1, 0, 0, 0, 0, 0]), result);

		let result = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]).full_mul(U256([::std::u64::MAX, 0, 0, 0]));
		assert_eq!(U512([1, ::std::u64::MAX, ::std::u64::MAX-1, 0, 0, 0, 0, 0]), result);

		let result = U256([::std::u64::MAX, 0, 0, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
		assert_eq!(U512([1, ::std::u64::MAX, ::std::u64::MAX-1, 0, 0, 0, 0, 0]), result);

		let result = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
		assert_eq!(U512([1, 0, ::std::u64::MAX-1, ::std::u64::MAX, 0, 0, 0, 0]), result);

		let result = U256([::std::u64::MAX, 0, 0, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
		assert_eq!(U512([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1, 0, 0, 0, 0]), result);

		let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]).full_mul(U256([::std::u64::MAX, 0, 0, 0]));
		assert_eq!(U512([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1, 0, 0, 0, 0]), result);

		let result = U256([::std::u64::MAX, 0, 0, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
		assert_eq!(U512([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1, 0, 0, 0]), result);

		let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]).full_mul(U256([::std::u64::MAX, 0, 0, 0]));
		assert_eq!(U512([1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1, 0, 0, 0]), result);

		let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
		assert_eq!(U512([1, 0, ::std::u64::MAX, ::std::u64::MAX-1, ::std::u64::MAX, 0, 0, 0]), result);

		let result = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
		assert_eq!(U512([1, 0, ::std::u64::MAX, ::std::u64::MAX-1, ::std::u64::MAX, 0, 0, 0]), result);

		let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]));
		assert_eq!(U512([1, 0, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1, ::std::u64::MAX, 0, 0]), result);

		let result = U256([::std::u64::MAX, ::std::u64::MAX, 0, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
		assert_eq!(U512([1, 0, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX-1, ::std::u64::MAX, 0, 0]), result);

		let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
		assert_eq!(U512([1, 0, 0, ::std::u64::MAX-1, ::std::u64::MAX, ::std::u64::MAX, 0, 0]), result);

		let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
		assert_eq!(U512([1, 0, 0, ::std::u64::MAX,  ::std::u64::MAX-1, ::std::u64::MAX, ::std::u64::MAX, 0]), result);

		let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, 0]));
		assert_eq!(U512([1, 0, 0, ::std::u64::MAX,  ::std::u64::MAX-1, ::std::u64::MAX, ::std::u64::MAX, 0]), result);

		let result = U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]).full_mul(U256([::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]));
		assert_eq!(U512([1, 0, 0, 0, ::std::u64::MAX-1, ::std::u64::MAX, ::std::u64::MAX, ::std::u64::MAX]), result);

		let result = U256([0, 0, 0, ::std::u64::MAX]).full_mul(U256([0, 0, 0, ::std::u64::MAX]));
		assert_eq!(U512([0, 0, 0, 0, 0, 0, 1, ::std::u64::MAX-1]), result);

		let result = U256([1, 0, 0, 0]).full_mul(U256([0, 0, 0, ::std::u64::MAX]));
		assert_eq!(U512([0, 0, 0, ::std::u64::MAX, 0, 0, 0, 0]), result);

		let result = U256([1, 2, 3, 4]).full_mul(U256([5, 0, 0, 0]));
		assert_eq!(U512([5, 10, 15, 20, 0, 0, 0, 0]), result);

		let result = U256([1, 2, 3, 4]).full_mul(U256([0, 6, 0, 0]));
		assert_eq!(U512([0, 6, 12, 18, 24, 0, 0, 0]), result);

		let result = U256([1, 2, 3, 4]).full_mul(U256([0, 0, 7, 0]));
		assert_eq!(U512([0, 0, 7, 14, 21, 28, 0, 0]), result);

		let result = U256([1, 2, 3, 4]).full_mul(U256([0, 0, 0, 8]));
		assert_eq!(U512([0, 0, 0, 8, 16, 24, 32, 0]), result);

		let result = U256([1, 2, 3, 4]).full_mul(U256([5, 6, 7, 8]));
		assert_eq!(U512([5, 16, 34, 60, 61, 52, 32, 0]), result);
	}

	#[test]
	fn u256_multi_muls2() {

		let (result, _) = U256([0, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, 0]));
		assert_eq!(U256([0, 0, 0, 0]), result);

		let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([1, 0, 0, 0]));
		assert_eq!(U256([1, 0, 0, 0]), result);

		let (result, _) = U256([5, 0, 0, 0]).overflowing_mul(U256([5, 0, 0, 0]));
		assert_eq!(U256([25, 0, 0, 0]), result);

		let (result, _) = U256([0, 5, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
		assert_eq!(U256([0, 0, 25, 0]), result);

		let (result, _) = U256([0, 0, 0, 1]).overflowing_mul(U256([1, 0, 0, 0]));
		assert_eq!(U256([0, 0, 0, 1]), result);

		let (result, _) = U256([0, 0, 0, 5]).overflowing_mul(U256([2, 0, 0, 0]));
		assert_eq!(U256([0, 0, 0, 10]), result);

		let (result, _) = U256([0, 0, 1, 0]).overflowing_mul(U256([0, 5, 0, 0]));
		assert_eq!(U256([0, 0, 0, 5]), result);

		let (result, _) = U256([0, 0, 8, 0]).overflowing_mul(U256([0, 0, 7, 0]));
		assert_eq!(U256([0, 0, 0, 0]), result);

		let (result, _) = U256([2, 0, 0, 0]).overflowing_mul(U256([0, 5, 0, 0]));
		assert_eq!(U256([0, 10, 0, 0]), result);

		let (result, _) = U256([1, 0, 0, 0]).overflowing_mul(U256([0, 0, 0, u64::max_value()]));
		assert_eq!(U256([0, 0, 0, u64::max_value()]), result);

		let x1: U256 = "0000000000000000000000000000000000000000000000000000012365124623".into();
		let x2sqr_right: U256 = "000000000000000000000000000000000000000000014baeef72e0378e2328c9".into();
		let x1sqr = x1 * x1;
		assert_eq!(x2sqr_right, x1sqr);

		let x1cube = x1sqr * x1;
		let x1cube_right: U256 = "0000000000000000000000000000000001798acde139361466f712813717897b".into();
		assert_eq!(x1cube_right, x1cube);

		let x1quad = x1cube * x1;
		let x1quad_right: U256 = "000000000000000000000001adbdd6bd6ff027485484b97f8a6a4c7129756dd1".into();
		assert_eq!(x1quad_right, x1quad);

		let x1penta = x1quad * x1;
		let x1penta_right: U256 = "00000000000001e92875ac24be246e1c57e0507e8c46cc8d233b77f6f4c72993".into();
		assert_eq!(x1penta_right, x1penta);

		let x1septima = x1penta * x1;
		let x1septima_right: U256 = "00022cca1da3f6e5722b7d3cc5bbfb486465ebc5a708dd293042f932d7eee119".into();
		assert_eq!(x1septima_right, x1septima);
	}
}
