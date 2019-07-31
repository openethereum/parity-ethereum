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
	io::{self, Read},
};

use bn;
use ethereum_types::{H256, U256};
use ethjson;
use ethkey::{Signature, recover as ec_recover};
use keccak_hash::keccak;
use log::{warn, trace};
use num::{BigUint, Zero, One};
use parity_bytes::BytesRef;
use parity_crypto::digest;
use eth_pairings;

/// Native implementation of a built-in contract.
trait Implementation: Send + Sync {
	/// execute this built-in on the given input, writing to the given output.
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str>;
}

/// A gas pricing scheme for built-in contracts.
trait Pricer: Send + Sync {
	/// The gas cost of running this built-in for the given input data.
	fn cost(&self, input: &[u8]) -> U256;
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
	fn cost(&self, input: &[u8]) -> U256 {
		U256::from(self.base) + U256::from(self.word) * U256::from((input.len() + 31) / 32)
	}
}

/// A alt_bn128_parinig pricing model. This computes a price using a base cost and a cost per pair.
struct AltBn128PairingPricer {
	base: usize,
	pair: usize,
}

impl Pricer for AltBn128PairingPricer {
	fn cost(&self, input: &[u8]) -> U256 {
		let cost = U256::from(self.base) + U256::from(self.pair) * U256::from(input.len() / 192);
		cost
	}
}

impl Pricer for ModexpPricer {
	fn cost(&self, input: &[u8]) -> U256 {
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

/// A EIP1962 pricing model. For now refers to implementation model.
struct EIP1962Pricer;

impl Pricer for EIP1962Pricer {
	fn cost(&self, input: &[u8]) -> U256 {
		let cost = eth_pairings::gas_meter::GasMeter::meter(&input);
		if cost.is_err() {
			return U256::max_value();
		}
		let cost = cost.expect("is non-error now");

		let cost = U256::from(cost);
		cost
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
	pub fn cost(&self, input: &[u8]) -> U256 {
		self.pricer.cost(input)
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

impl From<ethjson::spec::Builtin> for Builtin {
	fn from(b: ethjson::spec::Builtin) -> Self {
		let pricer: Box<dyn Pricer> = match b.pricing {
			ethjson::spec::Pricing::Linear(linear) => {
				Box::new(Linear {
					base: linear.base,
					word: linear.word,
				})
			}
			ethjson::spec::Pricing::Modexp(exp) => {
				Box::new(ModexpPricer {
					divisor: if exp.divisor == 0 {
						warn!("Zero modexp divisor specified. Falling back to default.");
						10
					} else {
						exp.divisor
					}
				})
			}
			ethjson::spec::Pricing::AltBn128Pairing(pricer) => {
				Box::new(AltBn128PairingPricer {
					base: pricer.base,
					pair: pricer.pair,
				})
			}
			ethjson::spec::Pricing::EIP1962(_) => {
				Box::new(EIP1962Pricer)
			}
		};

		Builtin {
			pricer: pricer,
			native: ethereum_builtin(&b.name),
			activate_at: b.activate_at.map(Into::into).unwrap_or(0),
		}
	}
}

/// Ethereum built-in factory.
fn ethereum_builtin(name: &str) -> Box<dyn Implementation> {
	match name {
		"identity" => Box::new(Identity) as Box<dyn Implementation>,
		"ecrecover" => Box::new(EcRecover) as Box<dyn Implementation>,
		"sha256" => Box::new(Sha256) as Box<dyn Implementation>,
		"ripemd160" => Box::new(Ripemd160) as Box<dyn Implementation>,
		"modexp" => Box::new(Modexp) as Box<dyn Implementation>,
		"alt_bn128_add" => Box::new(Bn128Add) as Box<dyn Implementation>,
		"alt_bn128_mul" => Box::new(Bn128Mul) as Box<dyn Implementation>,
		"alt_bn128_pairing" => Box::new(Bn128Pairing) as Box<dyn Implementation>,
		"eip_1962" => Box::new(EIP1962) as Box<dyn Implementation>,
		_ => panic!("invalid builtin name: {}", name),
	}
}

// Ethereum builtins:
//
// - The identity function
// - ec recovery
// - sha256
// - ripemd160
// - modexp (EIP198)

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
struct EIP1962;

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
			sum.y().to_big_endian(&mut write_buf[32..64]).expect("Cannot fail since 32..64 is 32-byte length");;
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
			sum.y().to_big_endian(&mut write_buf[32..64]).expect("Cannot fail since 32..64 is 32-byte length");;
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
			return Err("Invalid input length, must be multiple of 192 (3 * (32*2))".into())
		}

		if let Err(err) = self.execute_with_error(input, output) {
			trace!("Pairining error: {:?}", err);
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

impl Implementation for EIP1962 {
	/// Can fail in many cases, so leave it for an implementation
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
		let result = eth_pairings::public_interface::API::run(&input);
		if result.is_err() {
			trace!("EIP1962 error: {:?}", result.err().expect("is error"));
			return Err("Precompile call was unsuccessful");
		}
		let result: Vec<u8> = result.expect("some result");
		let mut buf = [0u8; 32];
		buf[31] = result[0];
		output.write(0, &buf);

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use ethereum_types::U256;
	use ethjson;
	use num::{BigUint, Zero, One};
	use parity_bytes::BytesRef;
	use rustc_hex::FromHex;
	use super::{Builtin, Linear, ethereum_builtin, Pricer, ModexpPricer, modexp as me, EIP1962Pricer};

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
		let f = ethereum_builtin("identity");

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
		let f = ethereum_builtin("sha256");

		let i = [0u8; 0];

		let mut o = [255u8; 32];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &(FromHex::from_hex("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap())[..]);

		let mut o8 = [255u8; 8];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o8[..])).expect("Builtin should not fail");
		assert_eq!(&o8[..], &(FromHex::from_hex("e3b0c44298fc1c14").unwrap())[..]);

		let mut o34 = [255u8; 34];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o34[..])).expect("Builtin should not fail");
		assert_eq!(&o34[..], &(FromHex::from_hex("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855ffff").unwrap())[..]);

		let mut ov = vec![];
		f.execute(&i[..], &mut BytesRef::Flexible(&mut ov)).expect("Builtin should not fail");
		assert_eq!(&ov[..], &(FromHex::from_hex("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap())[..]);
	}

	#[test]
	fn ripemd160() {
		let f = ethereum_builtin("ripemd160");

		let i = [0u8; 0];

		let mut o = [255u8; 32];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &(FromHex::from_hex("0000000000000000000000009c1185a5c5e9fc54612808977ee8f548b2258d31").unwrap())[..]);

		let mut o8 = [255u8; 8];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o8[..])).expect("Builtin should not fail");
		assert_eq!(&o8[..], &(FromHex::from_hex("0000000000000000").unwrap())[..]);

		let mut o34 = [255u8; 34];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o34[..])).expect("Builtin should not fail");
		assert_eq!(&o34[..], &(FromHex::from_hex("0000000000000000000000009c1185a5c5e9fc54612808977ee8f548b2258d31ffff").unwrap())[..]);
	}

	#[test]
	fn ecrecover() {
		let f = ethereum_builtin("ecrecover");

		let i = FromHex::from_hex("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03").unwrap();

		let mut o = [255u8; 32];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &(FromHex::from_hex("000000000000000000000000c08b5542d177ac6686946920409741463a15dddb").unwrap())[..]);

		let mut o8 = [255u8; 8];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o8[..])).expect("Builtin should not fail");
		assert_eq!(&o8[..], &(FromHex::from_hex("0000000000000000").unwrap())[..]);

		let mut o34 = [255u8; 34];
		f.execute(&i[..], &mut BytesRef::Fixed(&mut o34[..])).expect("Builtin should not fail");
		assert_eq!(&o34[..], &(FromHex::from_hex("000000000000000000000000c08b5542d177ac6686946920409741463a15dddbffff").unwrap())[..]);

		let i_bad = FromHex::from_hex("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001a650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03").unwrap();
		let mut o = [255u8; 32];
		f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &(FromHex::from_hex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap())[..]);

		let i_bad = FromHex::from_hex("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b000000000000000000000000000000000000000000000000000000000000001b0000000000000000000000000000000000000000000000000000000000000000").unwrap();
		let mut o = [255u8; 32];
		f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &(FromHex::from_hex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap())[..]);

		let i_bad = FromHex::from_hex("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b").unwrap();
		let mut o = [255u8; 32];
		f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &(FromHex::from_hex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap())[..]);

		let i_bad = FromHex::from_hex("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001bffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000000000001b").unwrap();
		let mut o = [255u8; 32];
		f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &(FromHex::from_hex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap())[..]);

		let i_bad = FromHex::from_hex("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b000000000000000000000000000000000000000000000000000000000000001bffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
		let mut o = [255u8; 32];
		f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(&o[..], &(FromHex::from_hex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap())[..]);

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
			native: ethereum_builtin("modexp"),
			activate_at: 0,
		};

		// test for potential gas cost multiplication overflow
		{
			let input = FromHex::from_hex("0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000003b27bafd00000000000000000000000000000000000000000000000000000000503c8ac3").unwrap();
			let expected_cost = U256::max_value();
			assert_eq!(f.cost(&input[..]), expected_cost.into());
		}

		// test for potential exp len overflow
		{
			let input = FromHex::from_hex("\
				00000000000000000000000000000000000000000000000000000000000000ff\
				2a1e530000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000"
				).unwrap();

			let mut output = vec![0u8; 32];
			let expected = FromHex::from_hex("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
			let expected_cost = U256::max_value();

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should fail");
			assert_eq!(output, expected);
			assert_eq!(f.cost(&input[..]), expected_cost.into());
		}

		// fermat's little theorem example.
		{
			let input = FromHex::from_hex("\
				0000000000000000000000000000000000000000000000000000000000000001\
				0000000000000000000000000000000000000000000000000000000000000020\
				0000000000000000000000000000000000000000000000000000000000000020\
				03\
				fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e\
				fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f"
			).unwrap();

			let mut output = vec![0u8; 32];
			let expected = FromHex::from_hex("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
			let expected_cost = 13056;

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should not fail");
			assert_eq!(output, expected);
			assert_eq!(f.cost(&input[..]), expected_cost.into());
		}

		// second example from EIP: zero base.
		{
			let input = FromHex::from_hex("\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000020\
				0000000000000000000000000000000000000000000000000000000000000020\
				fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e\
				fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f"
			).unwrap();

			let mut output = vec![0u8; 32];
			let expected = FromHex::from_hex("0000000000000000000000000000000000000000000000000000000000000000").unwrap();
			let expected_cost = 13056;

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should not fail");
			assert_eq!(output, expected);
			assert_eq!(f.cost(&input[..]), expected_cost.into());
		}

		// another example from EIP: zero-padding
		{
			let input = FromHex::from_hex("\
				0000000000000000000000000000000000000000000000000000000000000001\
				0000000000000000000000000000000000000000000000000000000000000002\
				0000000000000000000000000000000000000000000000000000000000000020\
				03\
				ffff\
				80"
			).unwrap();

			let mut output = vec![0u8; 32];
			let expected = FromHex::from_hex("3b01b01ac41f2d6e917c6d6a221ce793802469026d9ab7578fa2e79e4da6aaab").unwrap();
			let expected_cost = 768;

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should not fail");
			assert_eq!(output, expected);
			assert_eq!(f.cost(&input[..]), expected_cost.into());
		}

		// zero-length modulus.
		{
			let input = FromHex::from_hex("\
				0000000000000000000000000000000000000000000000000000000000000001\
				0000000000000000000000000000000000000000000000000000000000000002\
				0000000000000000000000000000000000000000000000000000000000000000\
				03\
				ffff"
			).unwrap();

			let mut output = vec![];
			let expected_cost = 0;

			f.execute(&input[..], &mut BytesRef::Flexible(&mut output)).expect("Builtin should not fail");
			assert_eq!(output.len(), 0); // shouldn't have written any output.
			assert_eq!(f.cost(&input[..]), expected_cost.into());
		}
	}

	#[test]
	fn bn128_add() {

		let f = Builtin {
			pricer: Box::new(Linear { base: 0, word: 0 }),
			native: ethereum_builtin("alt_bn128_add"),
			activate_at: 0,
		};

		// zero-points additions
		{
			let input = FromHex::from_hex("\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000"
			).unwrap();

			let mut output = vec![0u8; 64];
			let expected = FromHex::from_hex("\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000"
			).unwrap();

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should not fail");
			assert_eq!(output, expected);
		}

		// no input, should not fail
		{
			let mut empty = [0u8; 0];
			let input = BytesRef::Fixed(&mut empty);

			let mut output = vec![0u8; 64];
			let expected = FromHex::from_hex("\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000"
			).unwrap();

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should not fail");
			assert_eq!(output, expected);
		}

		// should fail - point not on curve
		{
			let input = FromHex::from_hex("\
				1111111111111111111111111111111111111111111111111111111111111111\
				1111111111111111111111111111111111111111111111111111111111111111\
				1111111111111111111111111111111111111111111111111111111111111111\
				1111111111111111111111111111111111111111111111111111111111111111"
			).unwrap();

			let mut output = vec![0u8; 64];

			let res = f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]));
			assert!(res.is_err(), "There should be built-in error here");
		}
	}

	#[test]
	fn bn128_mul() {

		let f = Builtin {
			pricer: Box::new(Linear { base: 0, word: 0 }),
			native: ethereum_builtin("alt_bn128_mul"),
			activate_at: 0,
		};

		// zero-point multiplication
		{
			let input = FromHex::from_hex("\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0200000000000000000000000000000000000000000000000000000000000000"
			).unwrap();

			let mut output = vec![0u8; 64];
			let expected = FromHex::from_hex("\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000"
			).unwrap();

			f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..])).expect("Builtin should not fail");
			assert_eq!(output, expected);
		}

		// should fail - point not on curve
		{
			let input = FromHex::from_hex("\
				1111111111111111111111111111111111111111111111111111111111111111\
				1111111111111111111111111111111111111111111111111111111111111111\
				0f00000000000000000000000000000000000000000000000000000000000000"
			).unwrap();

			let mut output = vec![0u8; 64];

			let res = f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]));
			assert!(res.is_err(), "There should be built-in error here");
		}
	}

	fn builtin_pairing() -> Builtin {
		Builtin {
			pricer: Box::new(Linear { base: 0, word: 0 }),
			native: ethereum_builtin("alt_bn128_pairing"),
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

	fn bytes(s: &'static str) -> Vec<u8> {
		FromHex::from_hex(s).expect("static str should contain valid hex bytes")
	}

	#[test]
	fn bn128_pairing_empty() {
		// should not fail, because empty input is a valid input of 0 elements
		empty_test(
			builtin_pairing(),
			bytes("0000000000000000000000000000000000000000000000000000000000000001"),
		);
	}

	#[test]
	fn bn128_pairing_notcurve() {
		// should fail - point not on curve
		error_test(
			builtin_pairing(),
			&bytes("\
				1111111111111111111111111111111111111111111111111111111111111111\
				1111111111111111111111111111111111111111111111111111111111111111\
				1111111111111111111111111111111111111111111111111111111111111111\
				1111111111111111111111111111111111111111111111111111111111111111\
				1111111111111111111111111111111111111111111111111111111111111111\
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
			&bytes("\
				1111111111111111111111111111111111111111111111111111111111111111\
				1111111111111111111111111111111111111111111111111111111111111111\
				111111111111111111111111111111"
			),
			Some("Invalid input length"),
		);
	}

	#[test]
	fn eip1962() {
		let f = Builtin {
			pricer: Box::new(EIP1962Pricer),
			native: ethereum_builtin("eip_1962"),
			activate_at: 0,
		};

		// test for returning max gas on invalid input
		{
			let input = FromHex::from_hex("00").unwrap();
			let expected_cost = U256::max_value();
			assert_eq!(f.cost(&input[..]), expected_cost.into());
		}

		// test for a valid return value for some BLS12 test vector
		{
			let input = FromHex::from_hex("\
				0701804a5a95350598fa3a7b559937d87888e67a656e460b98755a5a329757cd\
				057e4e8547b85c1ac34a75d0edf2fd8582068e75678a288ff23f41fdadca7697\
				a790de1543129f600cbda57c157cbd91d6d819c0ab305f46008b178844cfbaa4\
				388dc3aeec3a15938878844f48d1380c8f03bc2274b0abb11ed67a0b00c0b41f\
				5ab2790000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000025605cae48c0a763b192625adfafe0c9bbce3e0817b817194475986f7be\
				e5dec75fa2fbc8424c77e115695da2dbf091368b153a5bacc50baa33e77f42c4\
				5614e39ac3a13930721649f6b5073ed72a6acd44e7e76bf8ca6d4a5a95350598\
				fa3a7b559937d87888e67a656e460b98755a5a329757cd057e4e8547b85c1ac3\
				4a75d0edf2fd8582068e75678a288ff23f41fdadca7697a790de1543129f600c\
				bda57c157cbd91d6d819c0ab305f46008b178844cfbaa4388dc3aeec3a159388\
				78844f48d1380c8f03bc2274b0abb11ed67a0b00c0b41f5ab274000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000000000000000000\
				0000000000000000000000000000000000000000000000000001021606349c43\
				3d748f5dff1efbb0f7d5ef90af3e1dd910e601022ea117ba2c5ad7a74ebfd33c\
				256aea0f659a6e59207b59129639178a1291d587ba2965a306f329ed5389243f\
				c7492b02cf74befb8c1168cbe891ea626b316f990680142381d7cbfa67e9d737\
				6cf804b77a2608108a63de0db1d60155533e8372270553e5ca9ecaf94ea7088a\
				71ed52dbfa2e873ae8ac6f9fa8a2c162275995a83fef619e38fbc0b9478b65d9\
				1e8a0970b108f99ab8ddefb9f687c927b1f66d9c2ff512e83906ba6eb29843d1\
				7882cf52496e8f4446511b6e884b075a3f22a257891d2147c5779cc70dbac326\
				5cdd15b2b7caa88e15a6c39c5ba78c6b88c8f7c2fb25547ca3d7370dbdc14c01\
				e302087631b8d3260c0a7c49f2cc8d4f787d09ca3fb361ac7ec616d1bdeb302e\
				14a6c839085d893100babb625e1c910c45c91fa4bcdeb2690d194486e5871a0ca\
				020b92cc9e7e850802a5d66c768d076fb9439ec8a7a2c882206caedd9b397216a\
				ba2281576b86cd99f3117217a80fbba43b03d14725f822cee5d00a53762f6a270\
				dd1159b2bddb59b0850c28778fad13d782375209912cebbb4488146c02f34be1e\
				73f1903f1a9296889e8fc0db9f68f090994dbc88f9c89e6e021bfbe548b4c945b\
				481c7d3bbe2ba81c876622b392f11fd94edd71c0786da0ae2898f3c1807ab038f\
				456aefc300d99879c9dbaaf3dbacd87af9fd28e95b992164c7aff4c6fcaec8e71\
				1675cdf65d126e6b1f3a86c870720a4d02b9ddf239ce4cb7d8f6c5f669bbf363d\
				34cfac53b69c42981ceb1215dcae6c83c1de1b8de6292c175ac21ccc895a203e0\
				07110d377fb2f0d25b386666e7fade1ce43f7cd12b5eaab9f5d0368f00d278df9\
				f694776e9698471ea57e64f8e9e1cd973115a86e1393428c8db5cff95609adbca\
				ca5f39e4ddf9478e0939db944fe8c00cf77a8f67711184588d7bde62253fa275f\
				c982497316de9aedcfe2e65ad9fabb018a6ea0e0dcf76d6512c71a4d08935d822\
				fc7d1562fdda9a685d3c06b48126d52149d212ddb2f62fdeba4eb9fc919fb88ac\
				53a58c4df18d491dcfdb028b8ecee1a0df2d04953895804818fe1b6c4ef3c8fa5\
				87316ba9182a7d241480ad6be2ea117ba2c5ad7a74ebfd33c256aea0f659a6e59\
				207b59129639178a1291d587ba2965a306f329ed5389243fc7492b02cf74befb8\
				c1168cbe891ea626b316f990680142381d7cbfa67e9d7376cf804b77a2608108a\
				63de0db1d60155533e8372270553e5ca9ecaf94ea7088a71ed52dbfa2e873ae8a\
				c6f9fa8a2c162275995a80a6b3396cc9d398133ca335eb9ee7f75c95c74ab52ba\
				85a063aace301b0f10b25552a573e1bc90071e55af2c0cff373c2bf8fae449a12\
				3d37562c31c5884ee868c25f1579a9520de6e5ab99734f9c26708e087d13059c7\
				7b2c9d434f1b6f9600b3c6e598efb1417691878536298cfb45f0bbdd85a5145a3\
				018343364a6dda8af3fb361ac7ec616d1bdeb302e14a6c839085d893100babb62\
				5e1c910c45c91fa4bcdeb2690d194486e5871a0ca020b92cc9e7e850802a5d66c\
				768d076fb9439ec8a7a2c882206caedd9b397216aba2281576b86cd99f3117217\
				a80fbba43b03d14725f822cee5d00a53762f6a270dd1159b2bddb59b0850c2877\
				8fad13d782375209912cebbb4488146c02f34be1e73f1903f1a9296889e8fc0db\
				9f68f090994dbc88f9c89e6e021bfbe548b4c945b481c7d3bbe2ba81c876622b3\
				92f11fd94edd71c0786da0ae2898f3c1807ab038f456aefc300d99879c9dbaaf3\
				dbacd87af9fd28e95b992164c7aff4c6fcaec8e711675cdf65d126e6b1f3a86c8\
				70720a4d02b9ddf239ce4cb7d8f6c5f669bbf363d34cfac53b69c42981ceb1215\
				dcae6c83c1de1b8de6292c175ac21ccc895a203e007110d377fb2f0d25b386666\
				e7fade1ce43f7cd12b5eaab9f5d0368f00d278df9f694776e9698471ea57e64f8\
				e9e1cd973115a86e1393428c8db5cff95609adbcaca5f39e4ddf9478e0939db94\
				4fe8c00cf77a8f67711184588d7bde62253fa275fc982497316de9aedcfe2e65a\
				d9fabb018a6ea0e0dcf76d6512c71a4d08935d822fc7d1562fdda9a685d3c06b4\
				8126d52149d212ddb2f62fdeba4eb9fc919fb88ac53a58c4df18d491dcfdb028b\
				8ecee1a0df2d04953895804818fe1b6c4ef3c8fa587316ba9182a7d241480ad6be"
			).unwrap();

			let mut output = vec![0u8; 64];

			let res = f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]));
			assert!(res.is_ok(), "Should not return error for a valid pairing");
			assert!(output[31] == 1u8, "Should return `true` for a valid pairing");
		}

	}

	#[test]
	#[should_panic]
	fn from_unknown_linear() {
		let _ = ethereum_builtin("foo");
	}

	#[test]
	fn is_active() {
		let pricer = Box::new(Linear { base: 10, word: 20} );
		let b = Builtin {
			pricer: pricer as Box<dyn Pricer>,
			native: ethereum_builtin("identity"),
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
			native: ethereum_builtin("identity"),
			activate_at: 1,
		};

		assert_eq!(b.cost(&[0; 0]), U256::from(10));
		assert_eq!(b.cost(&[0; 1]), U256::from(30));
		assert_eq!(b.cost(&[0; 32]), U256::from(30));
		assert_eq!(b.cost(&[0; 33]), U256::from(50));

		let i = [0u8, 1, 2, 3];
		let mut o = [255u8; 4];
		b.execute(&i[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(i, o);
	}

	#[test]
	fn from_json() {
		let b = Builtin::from(ethjson::spec::Builtin {
			name: "identity".to_owned(),
			pricing: ethjson::spec::Pricing::Linear(ethjson::spec::Linear {
				base: 10,
				word: 20,
			}),
			activate_at: None,
		});

		assert_eq!(b.cost(&[0; 0]), U256::from(10));
		assert_eq!(b.cost(&[0; 1]), U256::from(30));
		assert_eq!(b.cost(&[0; 32]), U256::from(30));
		assert_eq!(b.cost(&[0; 33]), U256::from(50));

		let i = [0u8, 1, 2, 3];
		let mut o = [255u8; 4];
		b.execute(&i[..], &mut BytesRef::Fixed(&mut o[..])).expect("Builtin should not fail");
		assert_eq!(i, o);
	}
}
