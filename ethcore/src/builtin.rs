// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

use std::cmp::{max, min};
use std::io::{self, Read};

use byteorder::{ByteOrder, BigEndian};
use crypto::sha2::Sha256 as Sha256Digest;
use crypto::ripemd160::Ripemd160 as Ripemd160Digest;
use crypto::digest::Digest;
use num::{BigUint, Zero, One};

use hash::keccak;
use bigint::prelude::U256;
use bigint::hash::H256;
use bytes::BytesRef;
use ethkey::{Signature, recover as ec_recover};
use ethjson;

#[derive(Debug)]
pub struct Error(pub &'static str);

impl From<&'static str> for Error {
	fn from(val: &'static str) -> Self {
		Error(val)
	}
}

impl Into<::vm::Error> for Error {
	fn into(self) -> ::vm::Error {
		::vm::Error::BuiltIn(self.0)
	}
}

/// Native implementation of a built-in contract.
pub trait Impl: Send + Sync {
	/// execute this built-in on the given input, writing to the given output.
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), Error>;
}

/// A gas pricing scheme for built-in contracts.
pub trait Pricer: Send + Sync {
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
			U256::from(H256::from_slice(&buf[..]))
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
			U256::from(H256::from_slice(&buf[..]))
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
	pricer: Box<Pricer>,
	native: Box<Impl>,
	activate_at: u64,
}

impl Builtin {
	/// Simple forwarder for cost.
	pub fn cost(&self, input: &[u8]) -> U256 { self.pricer.cost(input) }

	/// Simple forwarder for execute.
	pub fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), Error> {
		self.native.execute(input, output)
	}

	/// Whether the builtin is activated at the given block number.
	pub fn is_active(&self, at: u64) -> bool { at >= self.activate_at }
}

impl From<ethjson::spec::Builtin> for Builtin {
	fn from(b: ethjson::spec::Builtin) -> Self {
		let pricer: Box<Pricer> = match b.pricing {
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
		};

		Builtin {
			pricer: pricer,
			native: ethereum_builtin(&b.name),
			activate_at: b.activate_at.map(Into::into).unwrap_or(0),
		}
	}
}

// Ethereum builtin creator.
fn ethereum_builtin(name: &str) -> Box<Impl> {
	match name {
		"identity" => Box::new(Identity) as Box<Impl>,
		"ecrecover" => Box::new(EcRecover) as Box<Impl>,
		"sha256" => Box::new(Sha256) as Box<Impl>,
		"ripemd160" => Box::new(Ripemd160) as Box<Impl>,
		"modexp" => Box::new(ModexpImpl) as Box<Impl>,
		"alt_bn128_add" => Box::new(Bn128AddImpl) as Box<Impl>,
		"alt_bn128_mul" => Box::new(Bn128MulImpl) as Box<Impl>,
		"alt_bn128_pairing" => Box::new(Bn128PairingImpl) as Box<Impl>,
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
struct ModexpImpl;

#[derive(Debug)]
struct Bn128AddImpl;

#[derive(Debug)]
struct Bn128MulImpl;

#[derive(Debug)]
struct Bn128PairingImpl;

impl Impl for Identity {
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), Error> {
		output.write(0, input);
		Ok(())
	}
}

impl Impl for EcRecover {
	fn execute(&self, i: &[u8], output: &mut BytesRef) -> Result<(), Error> {
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
				output.write(12, &r[12..r.len()]);
			}
		}

		Ok(())
	}
}

impl Impl for Sha256 {
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), Error> {
		let mut sha = Sha256Digest::new();
		sha.input(input);

		let mut out = [0; 32];
		sha.result(&mut out);

		output.write(0, &out);

		Ok(())
	}
}

impl Impl for Ripemd160 {
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), Error> {
		let mut sha = Ripemd160Digest::new();
		sha.input(input);

		let mut out = [0; 32];
		sha.result(&mut out[12..32]);

		output.write(0, &out);

		Ok(())
	}
}

// calculate modexp: exponentiation by squaring. the `num` crate has pow, but not modular.
fn modexp(mut base: BigUint, mut exp: BigUint, modulus: BigUint) -> BigUint {
	use num::Integer;

	if modulus <= BigUint::one() { // n^m % 0 || n^m % 1
		return BigUint::zero();
	}

	if exp.is_zero() { // n^0 % m
		return BigUint::one();
	}

	if base.is_zero() { // 0^n % m, n>0
		return BigUint::zero();
	}

	let mut result = BigUint::one();
	base = base % &modulus;

	// fast path for base divisible by modulus.
	if base.is_zero() { return BigUint::zero() }
	while !exp.is_zero() {
		if exp.is_odd() {
			result = (result * &base) % &modulus;
		}

		exp = exp >> 1;
		base = (base.clone() * base) % &modulus;
	}
	result
}

impl Impl for ModexpImpl {
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), Error> {
		let mut reader = input.chain(io::repeat(0));
		let mut buf = [0; 32];

		// read lengths as usize.
		// ignoring the first 24 bytes might technically lead us to fall out of consensus,
		// but so would running out of addressable memory!
		let mut read_len = |reader: &mut io::Chain<&[u8], io::Repeat>| {
			reader.read_exact(&mut buf[..]).expect("reading from zero-extended memory cannot fail; qed");
			BigEndian::read_u64(&buf[24..]) as usize
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
			let mut read_num = |len| {
				reader.read_exact(&mut buf[..len]).expect("reading from zero-extended memory cannot fail; qed");
				BigUint::from_bytes_be(&buf[..len])
			};

			let base = read_num(base_len);
			let exp = read_num(exp_len);
			let modulus = read_num(mod_len);
			modexp(base, exp, modulus)
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

fn read_fr(reader: &mut io::Chain<&[u8], io::Repeat>) -> Result<::bn::Fr, Error> {
	let mut buf = [0u8; 32];

	reader.read_exact(&mut buf[..]).expect("reading from zero-extended memory cannot fail; qed");
	::bn::Fr::from_slice(&buf[0..32]).map_err(|_| Error::from("Invalid field element"))
}

fn read_point(reader: &mut io::Chain<&[u8], io::Repeat>) -> Result<::bn::G1, Error> {
	use bn::{Fq, AffineG1, G1, Group};

	let mut buf = [0u8; 32];

	reader.read_exact(&mut buf[..]).expect("reading from zero-extended memory cannot fail; qed");
	let px = Fq::from_slice(&buf[0..32]).map_err(|_| Error::from("Invalid point x coordinate"))?;

	reader.read_exact(&mut buf[..]).expect("reading from zero-extended memory cannot fail; qed");
	let py = Fq::from_slice(&buf[0..32]).map_err(|_| Error::from("Invalid point y coordinate"))?;
	Ok(
		if px == Fq::zero() && py == Fq::zero() {
			G1::zero()
		} else {
			AffineG1::new(px, py).map_err(|_| Error::from("Invalid curve point"))?.into()
		}
	)
}

impl Impl for Bn128AddImpl {
	// Can fail if any of the 2 points does not belong the bn128 curve
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), Error> {
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

impl Impl for Bn128MulImpl {
	// Can fail if first paramter (bn128 curve point) does not actually belong to the curve
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), Error> {
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

impl Impl for Bn128PairingImpl {
	/// Can fail if:
	///     - input length is not a multiple of 192
	///     - any of odd points does not belong to bn128 curve
	///     - any of even points does not belong to the twisted bn128 curve over the field F_p^2 = F_p[i] / (i^2 + 1)
	fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), Error> {
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

impl Bn128PairingImpl {
	fn execute_with_error(&self, input: &[u8], output: &mut BytesRef) -> Result<(), Error> {
		use bn::{AffineG1, AffineG2, Fq, Fq2, pairing, G1, G2, Gt, Group};

		let elements = input.len() / 192; // (a, b_a, b_b - each 64-byte affine coordinates)
		let ret_val = if input.len() == 0 {
			U256::one()
		} else {
			let mut vals = Vec::new();
			for idx in 0..elements {
				let a_x = Fq::from_slice(&input[idx*192..idx*192+32])
					.map_err(|_| Error::from("Invalid a argument x coordinate"))?;

				let a_y = Fq::from_slice(&input[idx*192+32..idx*192+64])
					.map_err(|_| Error::from("Invalid a argument y coordinate"))?;

				let b_a_y = Fq::from_slice(&input[idx*192+64..idx*192+96])
					.map_err(|_| Error::from("Invalid b argument imaginary coeff x coordinate"))?;

				let b_a_x = Fq::from_slice(&input[idx*192+96..idx*192+128])
					.map_err(|_| Error::from("Invalid b argument imaginary coeff y coordinate"))?;

				let b_b_y = Fq::from_slice(&input[idx*192+128..idx*192+160])
					.map_err(|_| Error::from("Invalid b argument real coeff x coordinate"))?;

				let b_b_x = Fq::from_slice(&input[idx*192+160..idx*192+192])
					.map_err(|_| Error::from("Invalid b argument real coeff y coordinate"))?;

				let b_a = Fq2::new(b_a_x, b_a_y);
				let b_b = Fq2::new(b_b_x, b_b_y);
				let b = if b_a.is_zero() && b_b.is_zero() {
					G2::zero()
				} else {
					G2::from(AffineG2::new(b_a, b_b).map_err(|_| Error::from("Invalid b argument - not on curve"))?)
				};
				let a = if a_x.is_zero() && a_y.is_zero() {
					G1::zero()
				} else {
					G1::from(AffineG1::new(a_x, a_y).map_err(|_| Error::from("Invalid a argument - not on curve"))?)
				};
				vals.push((a, b));
			};

			let mul = vals.into_iter().fold(Gt::one(), |s, (a, b)| s * pairing(a, b));

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
	use super::{Builtin, Linear, ethereum_builtin, Pricer, ModexpPricer, modexp as me};
	use ethjson;
	use bigint::prelude::U256;
	use bytes::BytesRef;
	use rustc_hex::FromHex;
	use num::{BigUint, Zero, One};

	#[test]
	fn modexp_func() {
		// n^0 % m == 1
		let mut base = BigUint::parse_bytes(b"12345", 10).unwrap();
		let mut exp = BigUint::zero();
		let mut modulus = BigUint::parse_bytes(b"789", 10).unwrap();
		assert_eq!(me(base, exp, modulus), BigUint::one());

		// 0^n % m == 0
		base = BigUint::zero();
		exp = BigUint::parse_bytes(b"12345", 10).unwrap();
		modulus = BigUint::parse_bytes(b"789", 10).unwrap();
		assert_eq!(me(base, exp, modulus), BigUint::zero());

		// n^m % 1 == 0
		base = BigUint::parse_bytes(b"12345", 10).unwrap();
		exp = BigUint::parse_bytes(b"789", 10).unwrap();
		modulus = BigUint::one();
		assert_eq!(me(base, exp, modulus), BigUint::zero());

		// if n % d == 0, then n^m % d == 0
		base = BigUint::parse_bytes(b"12345", 10).unwrap();
		exp = BigUint::parse_bytes(b"789", 10).unwrap();
		modulus = BigUint::parse_bytes(b"15", 10).unwrap();
		assert_eq!(me(base, exp, modulus), BigUint::zero());

		// others
		base = BigUint::parse_bytes(b"12345", 10).unwrap();
		exp = BigUint::parse_bytes(b"789", 10).unwrap();
		modulus = BigUint::parse_bytes(b"97", 10).unwrap();
		assert_eq!(me(base, exp, modulus), BigUint::parse_bytes(b"55", 10).unwrap());
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
				if !e.0.contains(msg) {
					panic!("There should be error containing '{}' here, but got: '{}'", msg, e.0);
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
	#[should_panic]
	fn from_unknown_linear() {
		let _ = ethereum_builtin("foo");
	}

	#[test]
	fn is_active() {
		let pricer = Box::new(Linear { base: 10, word: 20} );
		let b = Builtin {
			pricer: pricer as Box<Pricer>,
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
			pricer: pricer as Box<Pricer>,
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
