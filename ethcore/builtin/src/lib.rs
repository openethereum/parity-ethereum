// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

//! Standard built-in contracts.

use std::{
	cmp::{max, min},
	convert::{TryFrom, TryInto},
	io::{self, Read, Cursor},
	mem::size_of,
};

use bn;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use common_types::errors::EthcoreError;
use ethereum_types::{H256, U256};
use ethjson;
use ethkey::{Signature, recover as ec_recover};
use keccak_hash::keccak;
use log::{warn, trace};
use num::{BigUint, Zero, One};
use parity_bytes::BytesRef;
use parity_crypto::digest;
use eip_152::compress;

/// Native implementation of a built-in contract.
trait Implementation: Send + Sync {
	/// execute this built-in on the given input, writing to the given output.
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str>;
}

/// A gas pricing scheme for built-in contracts.
// TODO: refactor this trait, see https://github.com/paritytech/parity-ethereum/issues/11014
trait Pricer: Send + Sync {
	/// The gas cost of running this built-in for the given input data at block number `at`
	fn cost(&self, input: &[u8], at: u64) -> U256;
}

/// Pricing for the Blake2 compression function (aka "F").
/// Computes the price as a fixed cost per round where the number of rounds is part of the input
/// byte slice.
pub type Blake2FPricer = u64;

impl Pricer for Blake2FPricer {
	fn cost(&self, input: &[u8], _at: u64) -> U256 {
		const FOUR: usize = std::mem::size_of::<u32>();
		// Returning zero if the conversion fails is fine because `execute()` will check the length
		// and bail with the appropriate error.
		if input.len() < FOUR {
			return U256::zero();
		}
		let (rounds_bytes, _) = input.split_at(FOUR);
		let rounds = u32::from_be_bytes(rounds_bytes.try_into().unwrap_or([0u8; FOUR]));
		U256::from(*self as u128 * rounds as u128)
	}
}

/// A linear pricing model. This computes a price using a base cost and a cost per-word.
struct Linear {
	base: usize,
	word: usize,
}

/// A special pricing model for modular exponentiation.
struct ModexpPricer {
	divisor: usize,
}

impl Pricer for Linear {
	fn cost(&self, input: &[u8], _at: u64) -> U256 {
		U256::from(self.base) + U256::from(self.word) * U256::from((input.len() + 31) / 32)
	}
}

/// alt_bn128 constant operations (add and mul) pricing model.
struct AltBn128ConstOperations {
	price: usize,
	eip1108_transition_at: u64,
	eip1108_transition_price: usize,
}

impl Pricer for AltBn128ConstOperations {
	fn cost(&self, _input: &[u8], at: u64) -> U256 {
		if at >= self.eip1108_transition_at {
			self.eip1108_transition_price.into()
		} else {
			self.price.into()
		}
	}
}

/// alt_bn128 pairing price
#[derive(Debug, Copy, Clone)]
struct AltBn128PairingPrice {
	base: usize,
	pair: usize,
}

/// alt_bn128_pairing pricing model. This computes a price using a base cost and a cost per pair.
struct AltBn128PairingPricer {
	price: AltBn128PairingPrice,
	eip1108_transition_at: u64,
	eip1108_transition_price: AltBn128PairingPrice,
}

impl Pricer for AltBn128PairingPricer {
	fn cost(&self, input: &[u8], at: u64) -> U256 {
		let price = if at >= self.eip1108_transition_at {
			self.eip1108_transition_price
		} else {
			self.price
		};

		U256::from(price.base) + U256::from(price.pair) * U256::from(input.len() / 192)
	}
}

impl Pricer for ModexpPricer {
	fn cost(&self, input: &[u8], _at: u64) -> U256 {
		let mut reader = input.chain(io::repeat(0));
		let mut buf = [0; 32];

		// read lengths as U256 here for accurate gas calculation.
		let mut read_len = || {
			reader.read_exact(&mut buf[..]).expect("reading from zero-extended memory cannot fail; qed");
			U256::from_big_endian(&buf[..])
		};
		let base_len = read_len();
		let exp_len = read_len();
		let mod_len = read_len();

		if mod_len.is_zero() && base_len.is_zero() {
			return U256::zero()
		}

		let max_len = U256::from(u32::max_value() / 2);
		if base_len > max_len || mod_len > max_len || exp_len > max_len {
			return U256::max_value();
		}
		let (base_len, exp_len, mod_len) = (base_len.low_u64(), exp_len.low_u64(), mod_len.low_u64());

		let m = max(mod_len, base_len);
		// read fist 32-byte word of the exponent.
		let exp_low = if base_len + 96 >= input.len() as u64 { U256::zero() } else {
			let mut buf = [0; 32];
			let mut reader = input[(96 + base_len as usize)..].chain(io::repeat(0));
			let len = min(exp_len, 32) as usize;
			reader.read_exact(&mut buf[(32 - len)..]).expect("reading from zero-extended memory cannot fail; qed");
			U256::from_big_endian(&buf[..])
		};

		let adjusted_exp_len = Self::adjusted_exp_len(exp_len, exp_low);

		let (gas, overflow) = Self::mult_complexity(m).overflowing_mul(max(adjusted_exp_len, 1));
		if overflow {
			return U256::max_value();
		}
		(gas / self.divisor as u64).into()
	}
}

impl ModexpPricer {
	fn adjusted_exp_len(len: u64, exp_low: U256) -> u64 {
		let bit_index = if exp_low.is_zero() { 0 } else { (255 - exp_low.leading_zeros()) as u64 };
		if len <= 32 {
			bit_index
		} else {
			8 * (len - 32) + bit_index
		}
	}

	fn mult_complexity(x: u64) -> u64 {
		match x {
			x if x <= 64 => x * x,
			x if x <= 1024 => (x * x) / 4 + 96 * x - 3072,
			x => (x * x) / 16 + 480 * x - 199680,
		}
	}
}

/// Pricing scheme, execution definition, and activation block for a built-in contract.
///
/// Call `cost` to compute cost for the given input, `execute` to execute the contract
/// on the given input, and `is_active` to determine whether the contract is active.
///
/// Unless `is_active` is true,
pub struct Builtin {
	pricer: Box<dyn Pricer>,
	native: Box<dyn Implementation>,
	activate_at: u64,
}

impl Builtin {
	/// Simple forwarder for cost.
	pub fn cost(&self, input: &[u8], at: u64) -> U256 {
		self.pricer.cost(input, at)
	}

	/// Simple forwarder for execute.
	pub fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
		self.native.execute(input, output)
	}

	/// Whether the builtin is activated at the given block number.
	pub fn is_active(&self, at: u64) -> bool {
		at >= self.activate_at
	}
}

impl TryFrom<ethjson::spec::Builtin> for Builtin {
	type Error = EthcoreError;

	fn try_from(b: ethjson::spec::Builtin) -> Result<Self, Self::Error> {
		let pricer: Box<dyn Pricer> = match b.pricing {
			ethjson::spec::Pricing::Blake2F { gas_per_round } => {
				Box::new(gas_per_round)
			},
			ethjson::spec::Pricing::Linear(linear) => {
				Box::new(Linear {
					base: linear.base,
					word: linear.word,
				})
			}
			ethjson::spec::Pricing::Modexp(exp) => {
				Box::new(ModexpPricer {
					divisor: if exp.divisor == 0 {
						warn!(target: "builtin", "Zero modexp divisor specified. Falling back to default.");
						10
					} else {
						exp.divisor
					}
				})
			}
			ethjson::spec::Pricing::AltBn128Pairing(pricer) => {
				Box::new(AltBn128PairingPricer {
					price: AltBn128PairingPrice {
						base: pricer.base,
						pair: pricer.pair,
					},
					eip1108_transition_at: b.eip1108_transition.map_or(u64::max_value(), Into::into),
					eip1108_transition_price: AltBn128PairingPrice {
						base: pricer.eip1108_transition_base,
						pair: pricer.eip1108_transition_pair,
					},
				})
			}
			ethjson::spec::Pricing::AltBn128ConstOperations(pricer) => {
				Box::new(AltBn128ConstOperations {
						price: pricer.price,
						eip1108_transition_price: pricer.eip1108_transition_price,
						eip1108_transition_at: b.eip1108_transition.map_or(u64::max_value(), Into::into)
				})
			}
		};

		let native = ethereum_builtin(&b.name)?;
		Ok(Builtin {
			pricer,
			native,
			activate_at: b.activate_at.map_or(0, Into::into),
		})
	}
}

/// Ethereum built-in factory.
fn ethereum_builtin(name: &str) -> Result<Box<dyn Implementation>, EthcoreError> {
	let implementation = match name {
		"identity" => Box::new(Identity) as Box<dyn Implementation>,
		"ecrecover" => Box::new(EcRecover) as Box<dyn Implementation>,
		"sha256" => Box::new(Sha256) as Box<dyn Implementation>,
		"ripemd160" => Box::new(Ripemd160) as Box<dyn Implementation>,
		"modexp" => Box::new(Modexp) as Box<dyn Implementation>,
		"alt_bn128_add" => Box::new(Bn128Add) as Box<dyn Implementation>,
		"alt_bn128_mul" => Box::new(Bn128Mul) as Box<dyn Implementation>,
		"alt_bn128_pairing" => Box::new(Bn128Pairing) as Box<dyn Implementation>,
		"blake2_f" => Box::new(Blake2F) as Box<dyn Implementation>,
		_ => return Err(EthcoreError::Msg(format!("invalid builtin name: {}", name))),
	};
	Ok(implementation)
}

// Ethereum builtins:
//
// - The identity function
// - ec recovery
// - sha256
// - ripemd160
// - modexp (EIP198)
// - alt_bn128_add
// - alt_bn128_mul
// - alt_bn128_pairing
// - blake2_f (The Blake2 compression function F, EIP-152)

#[derive(Debug)]
struct Identity;

#[derive(Debug)]
struct EcRecover;

#[derive(Debug)]
struct Sha256;

#[derive(Debug)]
struct Ripemd160;

#[derive(Debug)]
struct Modexp;

#[derive(Debug)]
struct Bn128Add;

#[derive(Debug)]
struct Bn128Mul;

#[derive(Debug)]
struct Bn128Pairing;

#[derive(Debug)]
struct Blake2F;

impl Implementation for Identity {
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
		output.write(0, input);
		Ok(())
	}
}

impl Implementation for EcRecover {
	fn execute(&self, i: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
		let len = min(i.len(), 128);

		let mut input = [0; 128];
		input[..len].copy_from_slice(&i[..len]);

		let hash = H256::from_slice(&input[0..32]);
		let v = H256::from_slice(&input[32..64]);
		let r = H256::from_slice(&input[64..96]);
		let s = H256::from_slice(&input[96..128]);

		let bit = match v[31] {
			27 | 28 if &v.0[..31] == &[0; 31] => v[31] - 27,
			_ => { return Ok(()); },
		};

		let s = Signature::from_rsv(&r, &s, bit);
		if s.is_valid() {
			if let Ok(p) = ec_recover(&s, &hash) {
				let r = keccak(p);
				output.write(0, &[0; 12]);
				output.write(12, &r.as_bytes()[12..]);
			}
		}

		Ok(())
	}
}

impl Implementation for Sha256 {
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
		let d = digest::sha256(input);
		output.write(0, &*d);
		Ok(())
	}
}

impl Implementation for Blake2F {
	/// Format of `input`:
	/// [4 bytes for rounds][64 bytes for h][128 bytes for m][8 bytes for t_0][8 bytes for t_1][1 byte for f]
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
		const BLAKE2_F_ARG_LEN: usize = 213;
		const PROOF: &str = "Checked the length of the input above; qed";

		if input.len() != BLAKE2_F_ARG_LEN {
			trace!(target: "builtin", "input length for Blake2 F precompile should be exactly 213 bytes, was {}", input.len());
			return Err("input length for Blake2 F precompile should be exactly 213 bytes")
		}

		let mut cursor = Cursor::new(input);
		let rounds = cursor.read_u32::<BigEndian>().expect(PROOF);

		// state vector, h
		let mut h = [0u64; 8];
		for state_word in h.iter_mut() {
			*state_word = cursor.read_u64::<LittleEndian>().expect(PROOF);
		}

		// message block vector, m
		let mut m = [0u64; 16];
		for msg_word in m.iter_mut() {
			*msg_word = cursor.read_u64::<LittleEndian>().expect(PROOF);
		}

		// 2w-bit offset counter, t
		let t = [
			cursor.read_u64::<LittleEndian>().expect(PROOF),
			cursor.read_u64::<LittleEndian>().expect(PROOF),
		];

		// final block indicator flag, "f"
		let f = match input.last() {
				Some(1) => true,
				Some(0) => false,
				_ => {
					trace!(target: "builtin", "incorrect final block indicator flag, was: {:?}", input.last());
					return Err("incorrect final block indicator flag")
				}
			};

		compress(&mut h, m, t, f, rounds as usize);

		let mut output_buf = [0u8; 8 * size_of::<u64>()];
		for (i, state_word) in h.iter().enumerate() {
			output_buf[i*8..(i+1)*8].copy_from_slice(&state_word.to_le_bytes());
		}
		output.write(0, &output_buf[..]);
		Ok(())
	}
}

impl Implementation for Ripemd160 {
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
		let hash = digest::ripemd160(input);
		output.write(0, &[0; 12][..]);
		output.write(12, &hash);
		Ok(())
	}
}

// calculate modexp: left-to-right binary exponentiation to keep multiplicands lower
fn modexp(mut base: BigUint, exp: Vec<u8>, modulus: BigUint) -> BigUint {
	const BITS_PER_DIGIT: usize = 8;

	// n^m % 0 || n^m % 1
	if modulus <= BigUint::one() {
		return BigUint::zero();
	}

	// normalize exponent
	let mut exp = exp.into_iter().skip_while(|d| *d == 0).peekable();

	// n^0 % m
	if let None = exp.peek() {
		return BigUint::one();
	}

	// 0^n % m, n > 0
	if base.is_zero() {
		return BigUint::zero();
	}

	base = base % &modulus;

	// Fast path for base divisible by modulus.
	if base.is_zero() { return BigUint::zero() }

	// Left-to-right binary exponentiation (Handbook of Applied Cryptography - Algorithm 14.79).
	// http://www.cacr.math.uwaterloo.ca/hac/about/chap14.pdf
	let mut result = BigUint::one();

	for digit in exp {
		let mut mask = 1 << (BITS_PER_DIGIT - 1);

		for _ in 0..BITS_PER_DIGIT {
			result = &result * &result % &modulus;

			if digit & mask > 0 {
				result = result * &base % &modulus;
			}

			mask >>= 1;
		}
	}

	result
}

impl Implementation for Modexp {
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
		let mut reader = input.chain(io::repeat(0));
		let mut buf = [0; 32];

		// read lengths as usize.
		// ignoring the first 24 bytes might technically lead us to fall out of consensus,
		// but so would running out of addressable memory!
		let mut read_len = |reader: &mut io::Chain<&[u8], io::Repeat>| {
			reader.read_exact(&mut buf[..]).expect("reading from zero-extended memory cannot fail; qed");
			let mut len_bytes = [0u8; 8];
			len_bytes.copy_from_slice(&buf[24..]);
			u64::from_be_bytes(len_bytes) as usize
		};

		let base_len = read_len(&mut reader);
		let exp_len = read_len(&mut reader);
		let mod_len = read_len(&mut reader);

		// Gas formula allows arbitrary large exp_len when base and modulus are empty, so we need to handle empty base first.
		let r = if base_len == 0 && mod_len == 0 {
			BigUint::zero()
		} else {
			// read the numbers themselves.
			let mut buf = vec![0; max(mod_len, max(base_len, exp_len))];
			let mut read_num = |reader: &mut io::Chain<&[u8], io::Repeat>, len: usize| {
				reader.read_exact(&mut buf[..len]).expect("reading from zero-extended memory cannot fail; qed");
				BigUint::from_bytes_be(&buf[..len])
			};

			let base = read_num(&mut reader, base_len);

			let mut exp_buf = vec![0; exp_len];
			reader.read_exact(&mut exp_buf[..exp_len]).expect("reading from zero-extended memory cannot fail; qed");

			let modulus = read_num(&mut reader, mod_len);

			modexp(base, exp_buf, modulus)
		};

		// write output to given memory, left padded and same length as the modulus.
		let bytes = r.to_bytes_be();

		// always true except in the case of zero-length modulus, which leads to
		// output of length and value 1.
		if bytes.len() <= mod_len {
			let res_start = mod_len - bytes.len();
			output.write(res_start, &bytes);
		}

		Ok(())
	}
}

fn read_fr(reader: &mut io::Chain<&[u8], io::Repeat>) -> Result<bn::Fr, &'static str> {
	let mut buf = [0u8; 32];

	reader.read_exact(&mut buf[..]).expect("reading from zero-extended memory cannot fail; qed");
	bn::Fr::from_slice(&buf[0..32]).map_err(|_| "Invalid field element")
}

fn read_point(reader: &mut io::Chain<&[u8], io::Repeat>) -> Result<bn::G1, &'static str> {
	use bn::{Fq, AffineG1, G1, Group};

	let mut buf = [0u8; 32];

	reader.read_exact(&mut buf[..]).expect("reading from zero-extended memory cannot fail; qed");
	let px = Fq::from_slice(&buf[0..32]).map_err(|_| "Invalid point x coordinate")?;

	reader.read_exact(&mut buf[..]).expect("reading from zero-extended memory cannot fail; qed");
	let py = Fq::from_slice(&buf[0..32]).map_err(|_| "Invalid point y coordinate")?;
	Ok(
		if px == Fq::zero() && py == Fq::zero() {
			G1::zero()
		} else {
			AffineG1::new(px, py).map_err(|_| "Invalid curve point")?.into()
		}
	)
}

impl Implementation for Bn128Add {
	// Can fail if any of the 2 points does not belong the bn128 curve
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
		use bn::AffineG1;

		let mut padded_input = input.chain(io::repeat(0));
		let p1 = read_point(&mut padded_input)?;
		let p2 = read_point(&mut padded_input)?;

		let mut write_buf = [0u8; 64];
		if let Some(sum) = AffineG1::from_jacobian(p1 + p2) {
			// point not at infinity
			sum.x().to_big_endian(&mut write_buf[0..32]).expect("Cannot fail since 0..32 is 32-byte length");
			sum.y().to_big_endian(&mut write_buf[32..64]).expect("Cannot fail since 32..64 is 32-byte length");
		}
		output.write(0, &write_buf);

		Ok(())
	}
}

impl Implementation for Bn128Mul {
	// Can fail if first paramter (bn128 curve point) does not actually belong to the curve
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
		use bn::AffineG1;

		let mut padded_input = input.chain(io::repeat(0));
		let p = read_point(&mut padded_input)?;
		let fr = read_fr(&mut padded_input)?;

		let mut write_buf = [0u8; 64];
		if let Some(sum) = AffineG1::from_jacobian(p * fr) {
			// point not at infinity
			sum.x().to_big_endian(&mut write_buf[0..32]).expect("Cannot fail since 0..32 is 32-byte length");
			sum.y().to_big_endian(&mut write_buf[32..64]).expect("Cannot fail since 32..64 is 32-byte length");
		}
		output.write(0, &write_buf);
		Ok(())
	}
}

impl Implementation for Bn128Pairing {
	/// Can fail if:
	///     - input length is not a multiple of 192
	///     - any of odd points does not belong to bn128 curve
	///     - any of even points does not belong to the twisted bn128 curve over the field F_p^2 = F_p[i] / (i^2 + 1)
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
		if input.len() % 192 != 0 {
			return Err("Invalid input length, must be multiple of 192 (3 * (32*2))")
		}

		if let Err(err) = self.execute_with_error(input, output) {
			trace!(target: "builtin", "Pairing error: {:?}", err);
			return Err(err)
		}
		Ok(())
	}
}

impl Bn128Pairing {
	fn execute_with_error(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
		use bn::{AffineG1, AffineG2, Fq, Fq2, pairing_batch, G1, G2, Gt, Group};

		let elements = input.len() / 192; // (a, b_a, b_b - each 64-byte affine coordinates)
		let ret_val = if input.len() == 0 {
			U256::one()
		} else {
			let mut vals = Vec::new();
			for idx in 0..elements {
				let a_x = Fq::from_slice(&input[idx*192..idx*192+32])
					.map_err(|_| "Invalid a argument x coordinate")?;

				let a_y = Fq::from_slice(&input[idx*192+32..idx*192+64])
					.map_err(|_| "Invalid a argument y coordinate")?;

				let b_a_y = Fq::from_slice(&input[idx*192+64..idx*192+96])
					.map_err(|_| "Invalid b argument imaginary coeff x coordinate")?;

				let b_a_x = Fq::from_slice(&input[idx*192+96..idx*192+128])
					.map_err(|_| "Invalid b argument imaginary coeff y coordinate")?;

				let b_b_y = Fq::from_slice(&input[idx*192+128..idx*192+160])
					.map_err(|_| "Invalid b argument real coeff x coordinate")?;

				let b_b_x = Fq::from_slice(&input[idx*192+160..idx*192+192])
					.map_err(|_| "Invalid b argument real coeff y coordinate")?;

				let b_a = Fq2::new(b_a_x, b_a_y);
				let b_b = Fq2::new(b_b_x, b_b_y);
				let b = if b_a.is_zero() && b_b.is_zero() {
					G2::zero()
				} else {
					G2::from(AffineG2::new(b_a, b_b).map_err(|_| "Invalid b argument - not on curve")?)
				};
				let a = if a_x.is_zero() && a_y.is_zero() {
					G1::zero()
				} else {
					G1::from(AffineG1::new(a_x, a_y).map_err(|_| "Invalid a argument - not on curve")?)
				};
				vals.push((a, b));
			};

			let mul = pairing_batch(&vals);

			if mul == Gt::one() {
				U256::one()
			} else {
				U256::zero()
			}
		};

		let mut buf = [0u8; 32];
		ret_val.to_big_endian(&mut buf);
		output.write(0, &buf);

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::convert::TryFrom;
	use ethereum_types::U256;
	use ethjson::uint::Uint;
	use num::{BigUint, Zero, One};
	use parity_bytes::BytesRef;
	use hex_literal::hex;
	use super::{Builtin, Linear, ethereum_builtin, Pricer, ModexpPricer, modexp as me};

	#[test]
	fn blake2f_cost() {
		let f = Builtin {
			pricer: Box::new(123),
			native: ethereum_builtin("blake2_f").expect("known builtin"),
			activate_at: 0,
		};
		// 5 rounds
		let input = hex!("0000000548c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
		let mut output = [0u8; 64];
		f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).unwrap();

		assert_eq!(f.cost(&input[..], 0), U256::from(123*5));
	}

	#[test]
	fn blake2f_cost_on_invalid_length() {
		let f = Builtin {
			pricer: Box::new(123),
			native: ethereum_builtin("blake2_f").expect("known builtin"),
			activate_at: 0,
		};
		// invalid input (too short)
		let input = hex!("00");

		assert_eq!(f.cost(&input[..], 0), U256::from(0));
	}

	#[test]
	fn blake2_f_is_err_on_invalid_length() {
		let blake2 = ethereum_builtin("blake2_f").expect("known builtin");
		// Test vector 1 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-1
		let input = hex!("00000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
		let mut out = [0u8; 64];

		let result = blake2.execute(&input[..], &mut BytesRef::Fixed(&mut out[..]));
		assert!(result.is_err());
		assert_eq!(result.unwrap_err(), "input length for Blake2 F precompile should be exactly 213 bytes");
	}

	#[test]
	fn blake2_f_is_err_on_invalid_length_2() {
		let blake2 = ethereum_builtin("blake2_f").expect("known builtin");
		// Test vector 2 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-2
		let input = hex!("000000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
		let mut out = [0u8; 64];

		let result = blake2.execute(&input[..], &mut BytesRef::Fixed(&mut out[..]));
		assert!(result.is_err());
		assert_eq!(result.unwrap_err(), "input length for Blake2 F precompile should be exactly 213 bytes");
	}

	#[test]
	fn blake2_f_is_err_on_bad_finalization_flag() {
		let blake2 = ethereum_builtin("blake2_f").expect("known builtin");
		// Test vector 3 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-3
		let input = hex!("0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000002");
		let mut out = [0u8; 64];

		let result = blake2.execute(&input[..], &mut BytesRef::Fixed(&mut out[..]));
		assert!(result.is_err());
		assert_eq!(result.unwrap_err(), "incorrect final block indicator flag");
	}

	#[test]
	fn blake2_f_zero_rounds_is_ok_test_vector_4() {
		let blake2 = ethereum_builtin("blake2_f").expect("known builtin");
		// Test vector 4 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-4
		let input = hex!("0000000048c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
		let expected = hex!("08c9bcf367e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d282e6ad7f520e511f6c3e2b8c68059b9442be0454267ce079217e1319cde05b");
		let mut output = [0u8; 64];
		blake2.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).unwrap();
		assert_eq!(&output[..], &expected[..]);
	}

	#[test]
	fn blake2_f_test_vector_5() {
		let blake2 = ethereum_builtin("blake2_f").expect("known builtin");
		// Test vector 5 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-5
		let input = hex!("0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
		let expected = hex!("ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923");
		let mut out = [0u8; 64];
		blake2.execute(&input[..], &mut BytesRef::Fixed(&mut out[..])).unwrap();
		assert_eq!(&out[..], &expected[..]);
	}

	#[test]
	fn blake2_f_test_vector_6() {
		let blake2 = ethereum_builtin("blake2_f").expect("known builtin");
		// Test vector 6 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-6
		let input = hex!("0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000");
		let expected = hex!("75ab69d3190a562c51aef8d88f1c2775876944407270c42c9844252c26d2875298743e7f6d5ea2f2d3e8d226039cd31b4e426ac4f2d3d666a610c2116fde4735");
		let mut out = [0u8; 64];
		blake2.execute(&input[..], &mut BytesRef::Fixed(&mut out[..])).unwrap();
		assert_eq!(&out[..], &expected[..]);
	}

	#[test]
	fn blake2_f_test_vector_7() {
		let blake2 = ethereum_builtin("blake2_f").expect("known builtin");
		// Test vector 7 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-7
		let input = hex!("0000000148c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
		let expected = hex!("b63a380cb2897d521994a85234ee2c181b5f844d2c624c002677e9703449d2fba551b3a8333bcdf5f2f7e08993d53923de3d64fcc68c034e717b9293fed7a421");
		let mut out = [0u8; 64];
		blake2.execute(&input[..], &mut BytesRef::Fixed(&mut out[..])).unwrap();
		assert_eq!(&out[..], &expected[..]);
	}

	#[ignore]
	#[test]
	fn blake2_f_test_vector_8() {
		let blake2 = ethereum_builtin("blake2_f").expect("known builtin");
		// Test vector 8 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-8
		// Note this test is slow, 4294967295/0xffffffff rounds take a while.
		let input = hex!("ffffffff48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
		let expected = hex!("fc59093aafa9ab43daae0e914c57635c5402d8e3d2130eb9b3cc181de7f0ecf9b22bf99a7815ce16419e200e01846e6b5df8cc7703041bbceb571de6631d2615");
		let mut out = [0u8; 64];
		blake2.execute(&input[..], &mut BytesRef::Fixed(&mut out[..])).unwrap();
		assert_eq!(&out[..], &expected[..]);
	}

	#[test]
	fn modexp_func() {
		// n^0 % m == 1
		let mut base = BigUint::parse_bytes(b"12345", 10).unwrap();
		let mut exp = BigUint::zero();
		let mut modulus = BigUint::parse_bytes(b"789", 10).unwrap();
		assert_eq!(me(base, exp.to_bytes_be(), modulus), BigUint::one());

		// 0^n % m == 0
		base = BigUint::zero();
		exp = BigUint::parse_bytes(b"12345", 10).unwrap();
		modulus = BigUint::parse_bytes(b"789", 10).unwrap();
		assert_eq!(me(base, exp.to_bytes_be(), modulus), BigUint::zero());

		// n^m % 1 == 0
		base = BigUint::parse_bytes(b"12345", 10).unwrap();
		exp = BigUint::parse_bytes(b"789", 10).unwrap();
		modulus = BigUint::one();
		assert_eq!(me(base, exp.to_bytes_be(), modulus), BigUint::zero());

		// if n % d == 0, then n^m % d == 0
		base = BigUint::parse_bytes(b"12345", 10).unwrap();
		exp = BigUint::parse_bytes(b"789", 10).unwrap();
		modulus = BigUint::parse_bytes(b"15", 10).unwrap();
		assert_eq!(me(base, exp.to_bytes_be(), modulus), BigUint::zero());

		// others
		base = BigUint::parse_bytes(b"12345", 10).unwrap();
		exp = BigUint::parse_bytes(b"789", 10).unwrap();
		modulus = BigUint::parse_bytes(b"97", 10).unwrap();
		assert_eq!(me(base, exp.to_bytes_be(), modulus), BigUint::parse_bytes(b"55", 10).unwrap());
	}

	#[test]
	fn identity() {
		let f = ethereum_builtin("identity").expect("known builtin");

		let i = [0u8, 1, 2, 3];

		let mut o2 = [255u8; 2];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o2[..])).expect("Builtin should not fail");
		assert_eq!(i[0..2], o2);

		let mut o4 = [255u8; 4];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o4[..])).expect("Builtin should not fail");
		assert_eq!(i, o4);

		let mut o8 = [255u8; 8];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o8[..])).expect("Builtin should not fail");
		assert_eq!(i, o8[..4]);
		assert_eq!([255u8; 4], o8[4..]);
	}

	#[test]
	fn sha256() {
		let f = ethereum_builtin("sha256").expect("known builtin");

		let i = [0u8; 0];

		let mut o = [255u8; 32];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"));

		let mut o8 = [255u8; 8];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o8[..])).expect("Builtin should not fail");
		assert_eq!(&o8[..], hex!("e3b0c44298fc1c14"));

		let mut o34 = [255u8; 34];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o34[..])).expect("Builtin should not fail");
		assert_eq!(&o34[..], &hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855ffff")[..]);

		let mut ov = vec![];
		f.execute(&i[..], &mut BytesRef::Flexible(&mut ov)).expect("Builtin should not fail");
		assert_eq!(&ov[..], &hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")[..]);
	}

	#[test]
	fn ripemd160() {
		let f = ethereum_builtin("ripemd160").expect("known builtin");

		let i = [0u8; 0];

		let mut o = [255u8; 32];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &hex!("0000000000000000000000009c1185a5c5e9fc54612808977ee8f548b2258d31")[..]);

		let mut o8 = [255u8; 8];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o8[..])).expect("Builtin should not fail");
		assert_eq!(&o8[..], &hex!("0000000000000000")[..]);

		let mut o34 = [255u8; 34];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o34[..])).expect("Builtin should not fail");
		assert_eq!(&o34[..], &hex!("0000000000000000000000009c1185a5c5e9fc54612808977ee8f548b2258d31ffff")[..]);
	}

	#[test]
	fn ecrecover() {
		let f = ethereum_builtin("ecrecover").expect("known builtin");

		let i = hex!("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03");

		let mut o = [255u8; 32];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &hex!("000000000000000000000000c08b5542d177ac6686946920409741463a15dddb")[..]);

		let mut o8 = [255u8; 8];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o8[..])).expect("Builtin should not fail");
		assert_eq!(&o8[..], &hex!("0000000000000000")[..]);

		let mut o34 = [255u8; 34];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o34[..])).expect("Builtin should not fail");
		assert_eq!(&o34[..], &hex!("000000000000000000000000c08b5542d177ac6686946920409741463a15dddbffff")[..]);

		let i_bad = hex!("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001a650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03");
		let mut o = [255u8; 32];
		f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")[..]);

		let i_bad = hex!("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b000000000000000000000000000000000000000000000000000000000000001b0000000000000000000000000000000000000000000000000000000000000000");
		let mut o = [255u8; 32];
		f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")[..]);

		let i_bad = hex!("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b");
		let mut o = [255u8; 32];
		f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")[..]);

		let i_bad = hex!("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001bffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000000000001b");
		let mut o = [255u8; 32];
		f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")[..]);

		let i_bad = hex!("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b000000000000000000000000000000000000000000000000000000000000001bffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
		let mut o = [255u8; 32];
		f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")[..]);

		// TODO: Should this (corrupted version of the above) fail rather than returning some address?
	/*	let i_bad = FromHex::from_hex("48173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03").unwrap();
		let mut o = [255u8; 32];
		f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..]));
		assert_eq!(&o[..], &(FromHex::from_hex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap())[..]);*/
	}

	#[test]
	fn modexp() {

		let f = Builtin {
			pricer: Box::new(ModexpPricer { divisor: 20 }),
			native: ethereum_builtin("modexp").expect("known builtin"),
			activate_at: 0,
		};

		// test for potential gas cost multiplication overflow
		{
			let input = hex!("0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000003b27bafd00000000000000000000000000000000000000000000000000000000503c8ac3");
			let expected_cost = U256::max_value();
			assert_eq!(f.cost(&input[..], 0), expected_cost.into());
		}

		// test for potential exp len overflow
		{
			let input = hex!("
				00000000000000000000000000000000000000000000000000000000000000ff
				2a1e530000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000"
				);

			let mut output = vec![0u8; 32];
			let expected = hex!("0000000000000000000000000000000000000000000000000000000000000000");
			let expected_cost = U256::max_value();

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should fail");
			assert_eq!(output, expected);
			assert_eq!(f.cost(&input[..], 0), expected_cost.into());
		}

		// fermat's little theorem example.
		{
			let input = hex!("
				0000000000000000000000000000000000000000000000000000000000000001
				0000000000000000000000000000000000000000000000000000000000000020
				0000000000000000000000000000000000000000000000000000000000000020
				03
				fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e
				fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f"
			);

			let mut output = vec![0u8; 32];
			let expected = hex!("0000000000000000000000000000000000000000000000000000000000000001");
			let expected_cost = 13056;

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should not fail");
			assert_eq!(output, expected);
			assert_eq!(f.cost(&input[..], 0), expected_cost.into());
		}

		// second example from EIP: zero base.
		{
			let input = hex!("
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000020
				0000000000000000000000000000000000000000000000000000000000000020
				fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e
				fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f"
			);

			let mut output = vec![0u8; 32];
			let expected = hex!("0000000000000000000000000000000000000000000000000000000000000000");
			let expected_cost = 13056;

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should not fail");
			assert_eq!(output, expected);
			assert_eq!(f.cost(&input[..], 0), expected_cost.into());
		}

		// another example from EIP: zero-padding
		{
			let input = hex!("
				0000000000000000000000000000000000000000000000000000000000000001
				0000000000000000000000000000000000000000000000000000000000000002
				0000000000000000000000000000000000000000000000000000000000000020
				03
				ffff
				80"
			);

			let mut output = vec![0u8; 32];
			let expected = hex!("3b01b01ac41f2d6e917c6d6a221ce793802469026d9ab7578fa2e79e4da6aaab");
			let expected_cost = 768;

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should not fail");
			assert_eq!(output, expected);
			assert_eq!(f.cost(&input[..], 0), expected_cost.into());
		}

		// zero-length modulus.
		{
			let input = hex!("
				0000000000000000000000000000000000000000000000000000000000000001
				0000000000000000000000000000000000000000000000000000000000000002
				0000000000000000000000000000000000000000000000000000000000000000
				03
				ffff"
			);

			let mut output = vec![];
			let expected_cost = 0;

			f.execute(&input[..], &mut BytesRef::Flexible(&mut output)).expect("Builtin should not fail");
			assert_eq!(output.len(), 0); // shouldn't have written any output.
			assert_eq!(f.cost(&input[..], 0), expected_cost.into());
		}
	}

	#[test]
	fn bn128_add() {

		let f = Builtin {
			pricer: Box::new(Linear { base: 0, word: 0 }),
			native: ethereum_builtin("alt_bn128_add").expect("known builtin"),
			activate_at: 0,
		};

		// zero-points additions
		{
			let input = hex!("
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000"
			);

			let mut output = vec![0u8; 64];
			let expected = hex!("
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000"
			);

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should not fail");
			assert_eq!(output, &expected[..]);
		}

		// no input, should not fail
		{
			let mut empty = [0u8; 0];
			let input = BytesRef::Fixed(&mut empty);

			let mut output = vec![0u8; 64];
			let expected = hex!("
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000"
			);

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should not fail");
			assert_eq!(output, &expected[..]);
		}

		// should fail - point not on curve
		{
			let input = hex!("
				1111111111111111111111111111111111111111111111111111111111111111
				1111111111111111111111111111111111111111111111111111111111111111
				1111111111111111111111111111111111111111111111111111111111111111
				1111111111111111111111111111111111111111111111111111111111111111"
			);

			let mut output = vec![0u8; 64];

			let res = f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]));
			assert!(res.is_err(), "There should be built-in error here");
		}
	}

	#[test]
	fn bn128_mul() {

		let f = Builtin {
			pricer: Box::new(Linear { base: 0, word: 0 }),
			native: ethereum_builtin("alt_bn128_mul").expect("known builtin"),
			activate_at: 0,
		};

		// zero-point multiplication
		{
			let input = hex!("
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000
				0200000000000000000000000000000000000000000000000000000000000000"
			);

			let mut output = vec![0u8; 64];
			let expected = hex!("
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000"
			);

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should not fail");
			assert_eq!(output, &expected[..]);
		}

		// should fail - point not on curve
		{
			let input = hex!("
				1111111111111111111111111111111111111111111111111111111111111111
				1111111111111111111111111111111111111111111111111111111111111111
				0f00000000000000000000000000000000000000000000000000000000000000"
			);

			let mut output = vec![0u8; 64];

			let res = f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]));
			assert!(res.is_err(), "There should be built-in error here");
		}
	}

	fn builtin_pairing() -> Builtin {
		Builtin {
			pricer: Box::new(Linear { base: 0, word: 0 }),
			native: ethereum_builtin("alt_bn128_pairing").expect("known builtin"),
			activate_at: 0,
		}
	}

	fn empty_test(f: Builtin, expected: Vec<u8>) {
		let mut empty = [0u8; 0];
		let input = BytesRef::Fixed(&mut empty);

		let mut output = vec![0u8; expected.len()];

		f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should not fail");
		assert_eq!(output, expected);
	}

	fn error_test(f: Builtin, input: &[u8], msg_contains: Option<&str>) {
		let mut output = vec![0u8; 64];
		let res = f.execute(input, &mut BytesRef::Fixed(&mut output[..]));
		if let Some(msg) = msg_contains {
			if let Err(e) = res {
				if !e.contains(msg) {
					panic!("There should be error containing '{}' here, but got: '{}'", msg, e);
				}
			}
		} else {
			assert!(res.is_err(), "There should be built-in error here");
		}
	}

	#[test]
	fn bn128_pairing_empty() {
		// should not fail, because empty input is a valid input of 0 elements
		empty_test(
			builtin_pairing(),
			hex!("0000000000000000000000000000000000000000000000000000000000000001").to_vec(),
		);
	}

	#[test]
	fn bn128_pairing_notcurve() {
		// should fail - point not on curve
		error_test(
			builtin_pairing(),
			&hex!("
				1111111111111111111111111111111111111111111111111111111111111111
				1111111111111111111111111111111111111111111111111111111111111111
				1111111111111111111111111111111111111111111111111111111111111111
				1111111111111111111111111111111111111111111111111111111111111111
				1111111111111111111111111111111111111111111111111111111111111111
				1111111111111111111111111111111111111111111111111111111111111111"
			),
			Some("not on curve"),
		);
	}

	#[test]
	fn bn128_pairing_fragmented() {
		// should fail - input length is invalid
		error_test(
			builtin_pairing(),
			&hex!("
				1111111111111111111111111111111111111111111111111111111111111111
				1111111111111111111111111111111111111111111111111111111111111111
				111111111111111111111111111111"
			),
			Some("Invalid input length"),
		);
	}

	#[test]
	#[should_panic]
	fn from_unknown_linear() {
		let _ = ethereum_builtin("foo").unwrap();
	}

	#[test]
	fn is_active() {
		let pricer = Box::new(Linear { base: 10, word: 20} );
		let b = Builtin {
			pricer: pricer as Box<dyn Pricer>,
			native: ethereum_builtin("identity").expect("known builtin"),
			activate_at: 100_000,
		};

		assert!(!b.is_active(99_999));
		assert!(b.is_active(100_000));
		assert!(b.is_active(100_001));
	}

	#[test]
	fn from_named_linear() {
		let pricer = Box::new(Linear { base: 10, word: 20 });
		let b = Builtin {
			pricer: pricer as Box<dyn Pricer>,
			native: ethereum_builtin("identity").expect("known builtin"),
			activate_at: 1,
		};

		assert_eq!(b.cost(&[0; 0], 0), U256::from(10));
		assert_eq!(b.cost(&[0; 1], 0), U256::from(30));
		assert_eq!(b.cost(&[0; 32], 0), U256::from(30));
		assert_eq!(b.cost(&[0; 33], 0), U256::from(50));

		let i = [0u8, 1, 2, 3];
		let mut o = [255u8; 4];
		b.execute(&i[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(i, o);
	}

	#[test]
	fn from_json() {
		let b = Builtin::try_from(ethjson::spec::Builtin {
			name: "identity".to_owned(),
			pricing: ethjson::spec::Pricing::Linear(ethjson::spec::Linear {
				base: 10,
				word: 20,
			}),
			activate_at: None,
			eip1108_transition: None,
		}).expect("known builtin");

		assert_eq!(b.cost(&[0; 0], 0), U256::from(10));
		assert_eq!(b.cost(&[0; 1], 0), U256::from(30));
		assert_eq!(b.cost(&[0; 32], 0), U256::from(30));
		assert_eq!(b.cost(&[0; 33], 0), U256::from(50));

		let i = [0u8, 1, 2, 3];
		let mut o = [255u8; 4];
		b.execute(&i[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(i, o);
	}

	#[test]
	fn bn128_pairing_eip1108_transition() {
		let b = Builtin::try_from(ethjson::spec::Builtin {
			name: "alt_bn128_pairing".to_owned(),
			pricing: ethjson::spec::Pricing::AltBn128Pairing(ethjson::spec::builtin::AltBn128Pairing {
				base: 100_000,
				pair: 80_000,
				eip1108_transition_base: 45_000,
				eip1108_transition_pair: 34_000,
			}),
			activate_at: Some(Uint(U256::from(10))),
			eip1108_transition: Some(Uint(U256::from(20))),
		}).expect("known builtin");

		assert_eq!(b.cost(&[0; 192 * 3], 10), U256::from(340_000), "80 000 * 3 + 100 000 == 340 000");
		assert_eq!(b.cost(&[0; 192 * 7], 20), U256::from(283_000), "34 000 * 7 + 45 000 == 283 000");
	}

	#[test]
	fn bn128_add_eip1108_transition() {
		let b = Builtin::try_from(ethjson::spec::Builtin {
			name: "alt_bn128_add".to_owned(),
			pricing: ethjson::spec::Pricing::AltBn128ConstOperations(ethjson::spec::builtin::AltBn128ConstOperations {
				price: 500,
				eip1108_transition_price: 150,
			}),
			activate_at: Some(Uint(U256::from(10))),
			eip1108_transition: Some(Uint(U256::from(20))),
		}).expect("known builtin");

		assert_eq!(b.cost(&[0; 192], 10), U256::from(500));
		assert_eq!(b.cost(&[0; 10], 20), U256::from(150), "after istanbul hardfork gas cost for add should be 150");
	}

	#[test]
	fn bn128_mul_eip1108_transition() {
		let b = Builtin::try_from(ethjson::spec::Builtin {
			name: "alt_bn128_mul".to_owned(),
			pricing: ethjson::spec::Pricing::AltBn128ConstOperations(ethjson::spec::builtin::AltBn128ConstOperations {
				price: 40_000,
				eip1108_transition_price: 6000,
			}),
			activate_at: Some(Uint(U256::from(10))),
			eip1108_transition: Some(Uint(U256::from(20))),
		}).expect("known builtin");

		assert_eq!(b.cost(&[0; 192], 10), U256::from(40_000));
		assert_eq!(b.cost(&[0; 10], 20), U256::from(6_000), "after istanbul hardfork gas cost for mul should be 6 000");
	}
}
