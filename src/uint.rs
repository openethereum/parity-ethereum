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

use std::fmt;
use std::cmp::{Ord, PartialOrd, Ordering};
use std::ops::*;
use std::str::FromStr;
use rustc_serialize::hex::{FromHex, FromHexError};

macro_rules! impl_map_from {
    ($thing:ident, $from:ty, $to:ty) => {
        impl From<$from> for $thing {
            fn from(value: $from) -> $thing {
                From::from(value as $to)
            }
        }
    }
}

macro_rules! impl_array_newtype {
    ($thing:ident, $ty:ty, $len:expr) => {
        impl $thing {
            #[inline]
            /// Converts the object to a raw pointer
            pub fn as_ptr(&self) -> *const $ty {
                let &$thing(ref dat) = self;
                dat.as_ptr()
            }

            #[inline]
            /// Converts the object to a mutable raw pointer
            pub fn as_mut_ptr(&mut self) -> *mut $ty {
                let &mut $thing(ref mut dat) = self;
                dat.as_mut_ptr()
            }

            #[inline]
            /// Returns the length of the object as an array
            pub fn len(&self) -> usize { $len }

            #[inline]
            /// Returns whether the object, as an array, is empty. Always false.
            pub fn is_empty(&self) -> bool { false }
        }

        impl<'a> From<&'a [$ty]> for $thing {
            fn from(data: &'a [$ty]) -> $thing {
                assert_eq!(data.len(), $len);
                unsafe {
                    use std::intrinsics::copy_nonoverlapping;
                    use std::mem;
                    let mut ret: $thing = mem::uninitialized();
                    copy_nonoverlapping(data.as_ptr(),
                                        ret.as_mut_ptr(),
                                        mem::size_of::<$thing>());
                    ret
                }
            }
        }

        impl Index<usize> for $thing {
            type Output = $ty;

            #[inline]
            fn index(&self, index: usize) -> &$ty {
                let &$thing(ref dat) = self;
                &dat[index]
            }
        }

        impl_index_newtype!($thing, $ty);

        impl PartialEq for $thing {
            #[inline]
            fn eq(&self, other: &$thing) -> bool {
                &self[..] == &other[..]
            }
        }

        impl Eq for $thing {}

        impl Clone for $thing {
            #[inline]
            fn clone(&self) -> $thing {
                $thing::from(&self[..])
            }
        }

        impl Copy for $thing {}
    }
}

macro_rules! impl_index_newtype {
    ($thing:ident, $ty:ty) => {
        impl Index<Range<usize>> for $thing {
            type Output = [$ty];

            #[inline]
            fn index(&self, index: Range<usize>) -> &[$ty] {
                &self.0[index]
            }
        }

        impl Index<RangeTo<usize>> for $thing {
            type Output = [$ty];

            #[inline]
            fn index(&self, index: RangeTo<usize>) -> &[$ty] {
                &self.0[index]
            }
        }

        impl Index<RangeFrom<usize>> for $thing {
            type Output = [$ty];

            #[inline]
            fn index(&self, index: RangeFrom<usize>) -> &[$ty] {
                &self.0[index]
            }
        }

        impl Index<RangeFull> for $thing {
            type Output = [$ty];

            #[inline]
            fn index(&self, _: RangeFull) -> &[$ty] {
                &self.0[..]
            }
        }
    }
}

macro_rules! construct_uint {
    ($name:ident, $n_words:expr) => (
        /// Little-endian large integer type
        pub struct $name(pub [u64; $n_words]);
        impl_array_newtype!($name, u64, $n_words);

        impl $name {
            /// Conversion to u32
            #[inline]
            fn low_u32(&self) -> u32 {
                let &$name(ref arr) = self;
                arr[0] as u32
            }

            /// Return the least number of bits needed to represent the number
            #[inline]
            pub fn bits(&self) -> usize {
                let &$name(ref arr) = self;
                for i in 1..$n_words {
                    if arr[$n_words - i] > 0 { return (0x40 * ($n_words - i + 1)) - arr[$n_words - i].leading_zeros() as usize; }
                }
                0x40 - arr[0].leading_zeros() as usize
            }

            #[inline]
            pub fn bit(&self, index: usize) -> bool {
                let &$name(ref arr) = self;
                arr[index / 64] & (1 << (index % 64)) != 0
            }

            #[inline]
            pub fn byte(&self, index: usize) -> u8 {
                let &$name(ref arr) = self;
                (arr[index / 8] >> ((index % 8)) * 8) as u8
            }

            /// Multiplication by u32
            fn mul_u32(self, other: u32) -> $name {
                let $name(ref arr) = self;
                let mut carry = [0u64; $n_words];
                let mut ret = [0u64; $n_words];
                for i in 0..$n_words {
                    let upper = other as u64 * (arr[i] >> 32);
                    let lower = other as u64 * (arr[i] & 0xFFFFFFFF);
                    if i < 3 {
                        carry[i + 1] += upper >> 32;
                    }
                    ret[i] = lower + (upper << 32);
                }
                $name(ret) + $name(carry)
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
                println!("{}", value);
                let bytes: &[u8] = &try!(value.from_hex());
                Ok(From::from(bytes))
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
                    ret[i] = me[i].wrapping_add(you[i]);
                    if i < $n_words - 1 && ret[i] < me[i] {
                        carry[i + 1] = 1;
                        b_carry = true;
                    }
                }
                if b_carry { $name(ret) + $name(carry) } else { $name(ret) }
            }
        }

        impl Sub<$name> for $name {
            type Output = $name;

            #[inline]
            fn sub(self, other: $name) -> $name {
                self + !other + From::from(1u64)
            }
        }

        impl Mul<$name> for $name {
            type Output = $name;

            fn mul(self, other: $name) -> $name {
                let mut me = self;
                // TODO: be more efficient about this
                for i in 0..(2 * $n_words) {
                    me = (me + me.mul_u32((other >> (32 * i)).low_u32())) << (32 * i);
                }
                me
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
                        sub_copy = sub_copy - shift_copy;
                    }
                    shift_copy = shift_copy >> 1;
                    if shift == 0 { break; }
                    shift -= 1;
                }

                $name(ret)
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
                    if bit_shift < 64 && i + word_shift < $n_words {
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
    );
}

construct_uint!(U256, 4);
construct_uint!(U128, 2);

impl From<U128> for U256 {
    fn from(value: U128) -> U256 {
        let U128(ref arr) = value; 
        let mut ret = [0; 4];
        ret[0] = arr[0];
        ret[1] = arr[1];
        U256(ret)
    }
}

#[cfg(test)]
mod tests {
    use uint::U256;
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
        let sub = incr - init;
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
}

