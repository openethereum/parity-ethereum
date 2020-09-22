// Copyright 2015-2020 Parity Technologies (UK) Ltd.
// This file is part of OpenEthereum.

// OpenEthereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// OpenEthereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with OpenEthereum.  If not, see <http://www.gnu.org/licenses/>.

//! Standard built-in contracts.

#![allow(missing_docs)]

use std::{
    cmp::{max, min},
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
    io::{self, Cursor, Read},
    mem::size_of,
    str::FromStr,
};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use eip_152::compress;
use eth_pairings::public_interface::eip2537::{
    EIP2537Executor, SCALAR_BYTE_LENGTH, SERIALIZED_G1_POINT_BYTE_LENGTH,
    SERIALIZED_G2_POINT_BYTE_LENGTH,
};
use ethereum_types::{H256, U256};
use ethjson;
use ethkey::{recover as ec_recover, Signature};
use keccak_hash::keccak;
use log::{trace, warn};
use num::{BigUint, One, Zero};
use parity_bytes::BytesRef;
use parity_crypto::digest;

/// Native implementation of a built-in contract.
pub trait Implementation: Send + Sync {
    /// execute this built-in on the given input, writing to the given output.
    fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str>;
}

/// A gas pricing scheme for built-in contracts.
trait Pricer: Send + Sync {
    /// The gas cost of running this built-in for the given input data at block number `at`
    fn cost(&self, input: &[u8]) -> U256;
}

/// Pricing for the Blake2 compression function (aka "F").
/// Computes the price as a fixed cost per round where the number of rounds is part of the input
/// byte slice.
pub type Blake2FPricer = u64;

impl Pricer for Blake2FPricer {
    fn cost(&self, input: &[u8]) -> U256 {
        const FOUR: usize = std::mem::size_of::<u32>();
        // Returning zero if the conversion fails is fine because `execute()` will check the length
        // and bail with the appropriate error.
        if input.len() < FOUR {
            return U256::zero();
        }
        let (rounds_bytes, _) = input.split_at(FOUR);
        let rounds = u32::from_be_bytes(rounds_bytes.try_into().unwrap_or([0u8; 4]));
        U256::from(*self as u64 * rounds as u64)
    }
}

/// Pricing model
#[derive(Debug)]
enum Pricing {
    AltBn128Pairing(AltBn128PairingPricer),
    AltBn128ConstOperations(AltBn128ConstOperations),
    Blake2F(Blake2FPricer),
    Linear(Linear),
    Modexp(ModexpPricer),
    Bls12Pairing(Bls12PairingPricer),
    Bls12ConstOperations(Bls12ConstOperations),
    Bls12MultiexpG1(Bls12MultiexpPricerG1),
    Bls12MultiexpG2(Bls12MultiexpPricerG2),
}

impl Pricer for Pricing {
    fn cost(&self, input: &[u8]) -> U256 {
        match self {
            Pricing::AltBn128Pairing(inner) => inner.cost(input),
            Pricing::AltBn128ConstOperations(inner) => inner.cost(input),
            Pricing::Blake2F(inner) => inner.cost(input),
            Pricing::Linear(inner) => inner.cost(input),
            Pricing::Modexp(inner) => inner.cost(input),
            Pricing::Bls12Pairing(inner) => inner.cost(input),
            Pricing::Bls12ConstOperations(inner) => inner.cost(input),
            Pricing::Bls12MultiexpG1(inner) => inner.cost(input),
            Pricing::Bls12MultiexpG2(inner) => inner.cost(input),
        }
    }
}

/// A linear pricing model. This computes a price using a base cost and a cost per-word.
#[derive(Debug)]
struct Linear {
    base: u64,
    word: u64,
}

/// A special pricing model for modular exponentiation.
#[derive(Debug)]
struct ModexpPricer {
    divisor: u64,
}

impl Pricer for Linear {
    fn cost(&self, input: &[u8]) -> U256 {
        U256::from(self.base) + U256::from(self.word) * U256::from((input.len() + 31) / 32)
    }
}

/// alt_bn128 pairing price
#[derive(Debug, Copy, Clone)]
struct AltBn128PairingPrice {
    base: u64,
    pair: u64,
}

/// alt_bn128_pairing pricing model. This computes a price using a base cost and a cost per pair.
#[derive(Debug)]
struct AltBn128PairingPricer {
    price: AltBn128PairingPrice,
}

/// Pricing for constant alt_bn128 operations (ECADD and ECMUL)
#[derive(Debug, Copy, Clone)]
pub struct AltBn128ConstOperations {
    /// Fixed price.
    pub price: u64,
}

impl Pricer for AltBn128ConstOperations {
    fn cost(&self, _input: &[u8]) -> U256 {
        self.price.into()
    }
}

impl Pricer for AltBn128PairingPricer {
    fn cost(&self, input: &[u8]) -> U256 {
        U256::from(self.price.base) + U256::from(self.price.pair) * U256::from(input.len() / 192)
    }
}

impl Pricer for ModexpPricer {
    fn cost(&self, input: &[u8]) -> U256 {
        let mut reader = input.chain(io::repeat(0));
        let mut buf = [0; 32];

        // read lengths as U256 here for accurate gas calculation.
        let mut read_len = || {
            reader
                .read_exact(&mut buf[..])
                .expect("reading from zero-extended memory cannot fail; qed");
            U256::from(H256::from_slice(&buf[..]))
        };
        let base_len = read_len();
        let exp_len = read_len();
        let mod_len = read_len();

        if mod_len.is_zero() && base_len.is_zero() {
            return U256::zero();
        }

        let max_len = U256::from(u32::max_value() / 2);
        if base_len > max_len || mod_len > max_len || exp_len > max_len {
            return U256::max_value();
        }
        let (base_len, exp_len, mod_len) =
            (base_len.low_u64(), exp_len.low_u64(), mod_len.low_u64());

        let m = max(mod_len, base_len);
        // read fist 32-byte word of the exponent.
        let exp_low = if base_len + 96 >= input.len() as u64 {
            U256::zero()
        } else {
            buf.iter_mut().for_each(|b| *b = 0);
            let mut reader = input[(96 + base_len as usize)..].chain(io::repeat(0));
            let len = min(exp_len, 32) as usize;
            reader
                .read_exact(&mut buf[(32 - len)..])
                .expect("reading from zero-extended memory cannot fail; qed");
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
        let bit_index = if exp_low.is_zero() {
            0
        } else {
            (255 - exp_low.leading_zeros()) as u64
        };
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
            x => (x * x) / 16 + 480 * x - 199_680,
        }
    }
}

/// Bls12 pairing price
#[derive(Debug, Copy, Clone)]
struct Bls12PairingPrice {
    base: u64,
    pair: u64,
}

/// bls12_pairing pricing model. This computes a price using a base cost and a cost per pair.
#[derive(Debug)]
struct Bls12PairingPricer {
    price: Bls12PairingPrice,
}

/// Pricing for constant Bls12 operations (ADD and MUL in G1 and G2, as well as mappings)
#[derive(Debug, Copy, Clone)]
pub struct Bls12ConstOperations {
    /// Fixed price.
    pub price: u64,
}

/// Discount table for multiexponentiation (Pippenger's Algorithm)
/// Later on is normalized using the divisor
pub const BLS12_MULTIEXP_DISCOUNTS_TABLE: [[u64; 2]; BLS12_MULTIEXP_PAIRS_FOR_MAX_DISCOUNT] = [
    [1, 1200],
    [2, 888],
    [3, 764],
    [4, 641],
    [5, 594],
    [6, 547],
    [7, 500],
    [8, 453],
    [9, 438],
    [10, 423],
    [11, 408],
    [12, 394],
    [13, 379],
    [14, 364],
    [15, 349],
    [16, 334],
    [17, 330],
    [18, 326],
    [19, 322],
    [20, 318],
    [21, 314],
    [22, 310],
    [23, 306],
    [24, 302],
    [25, 298],
    [26, 294],
    [27, 289],
    [28, 285],
    [29, 281],
    [30, 277],
    [31, 273],
    [32, 269],
    [33, 268],
    [34, 266],
    [35, 265],
    [36, 263],
    [37, 262],
    [38, 260],
    [39, 259],
    [40, 257],
    [41, 256],
    [42, 254],
    [43, 253],
    [44, 251],
    [45, 250],
    [46, 248],
    [47, 247],
    [48, 245],
    [49, 244],
    [50, 242],
    [51, 241],
    [52, 239],
    [53, 238],
    [54, 236],
    [55, 235],
    [56, 233],
    [57, 232],
    [58, 231],
    [59, 229],
    [60, 228],
    [61, 226],
    [62, 225],
    [63, 223],
    [64, 222],
    [65, 221],
    [66, 220],
    [67, 219],
    [68, 219],
    [69, 218],
    [70, 217],
    [71, 216],
    [72, 216],
    [73, 215],
    [74, 214],
    [75, 213],
    [76, 213],
    [77, 212],
    [78, 211],
    [79, 211],
    [80, 210],
    [81, 209],
    [82, 208],
    [83, 208],
    [84, 207],
    [85, 206],
    [86, 205],
    [87, 205],
    [88, 204],
    [89, 203],
    [90, 202],
    [91, 202],
    [92, 201],
    [93, 200],
    [94, 199],
    [95, 199],
    [96, 198],
    [97, 197],
    [98, 196],
    [99, 196],
    [100, 195],
    [101, 194],
    [102, 193],
    [103, 193],
    [104, 192],
    [105, 191],
    [106, 191],
    [107, 190],
    [108, 189],
    [109, 188],
    [110, 188],
    [111, 187],
    [112, 186],
    [113, 185],
    [114, 185],
    [115, 184],
    [116, 183],
    [117, 182],
    [118, 182],
    [119, 181],
    [120, 180],
    [121, 179],
    [122, 179],
    [123, 178],
    [124, 177],
    [125, 176],
    [126, 176],
    [127, 175],
    [128, 174],
];

/// Max discount allowed
pub const BLS12_MULTIEXP_MAX_DISCOUNT: u64 = 174;
/// Max discount is reached at this number of pairs
pub const BLS12_MULTIEXP_PAIRS_FOR_MAX_DISCOUNT: usize = 128;
/// Divisor for discounts table
pub const BLS12_MULTIEXP_DISCOUNT_DIVISOR: u64 = 1000;
/// Length of single G1 + G2 points pair for pairing operation
pub const BLS12_G1_AND_G2_PAIR_LEN: usize =
    SERIALIZED_G1_POINT_BYTE_LENGTH + SERIALIZED_G2_POINT_BYTE_LENGTH;

/// Marter trait for length of input per one pair (point + scalar)
pub trait PointScalarLength: Copy + Clone + std::fmt::Debug + Send + Sync {
    /// Length itself
    const LENGTH: usize;
}
/// Marker trait that indicated that we perform operations in G1
#[derive(Clone, Copy, Debug)]
pub struct G1Marker;
impl PointScalarLength for G1Marker {
    const LENGTH: usize = SERIALIZED_G1_POINT_BYTE_LENGTH + SCALAR_BYTE_LENGTH;
}
/// Marker trait that indicated that we perform operations in G2
#[derive(Clone, Copy, Debug)]
pub struct G2Marker;
impl PointScalarLength for G2Marker {
    const LENGTH: usize = SERIALIZED_G2_POINT_BYTE_LENGTH + SCALAR_BYTE_LENGTH;
}

/// Pricing for constant Bls12 operations (ADD and MUL in G1 and G2, as well as mappings)
#[derive(Debug, Copy, Clone)]
pub struct Bls12MultiexpPricer<P: PointScalarLength> {
    /// Base const of the operation (G1 or G2 multiplication)
    pub base_price: Bls12ConstOperations,

    _marker: std::marker::PhantomData<P>,
}

impl Pricer for Bls12ConstOperations {
    fn cost(&self, _input: &[u8]) -> U256 {
        self.price.into()
    }
}

impl Pricer for Bls12PairingPricer {
    fn cost(&self, input: &[u8]) -> U256 {
        U256::from(self.price.base)
            + U256::from(self.price.pair) * U256::from(input.len() / BLS12_G1_AND_G2_PAIR_LEN)
    }
}

impl<P: PointScalarLength> Pricer for Bls12MultiexpPricer<P> {
    fn cost(&self, input: &[u8]) -> U256 {
        let num_pairs = input.len() / P::LENGTH;
        if num_pairs == 0 {
            return U256::zero();
        }
        let discount = if num_pairs > BLS12_MULTIEXP_PAIRS_FOR_MAX_DISCOUNT {
            BLS12_MULTIEXP_MAX_DISCOUNT
        } else {
            let table_entry = BLS12_MULTIEXP_DISCOUNTS_TABLE[num_pairs - 1];
            table_entry[1]
        };
        U256::from(self.base_price.price) * U256::from(num_pairs) * U256::from(discount)
            / U256::from(BLS12_MULTIEXP_DISCOUNT_DIVISOR)
    }
}

/// Multiexp pricer in G1
pub type Bls12MultiexpPricerG1 = Bls12MultiexpPricer<G1Marker>;

/// Multiexp pricer in G2
pub type Bls12MultiexpPricerG2 = Bls12MultiexpPricer<G2Marker>;

/// Pricing scheme, execution definition, and activation block for a built-in contract.
///
/// Call `cost` to compute cost for the given input, `execute` to execute the contract
/// on the given input, and `is_active` to determine whether the contract is active.
pub struct Builtin {
    pricer: BTreeMap<u64, Pricing>,
    native: EthereumBuiltin,
}

impl Builtin {
    /// Simple forwarder for cost.
    ///
    /// Return the cost of the most recently activated pricer at the current block number.
    ///
    /// If no pricer is actived `zero` is returned
    ///
    /// If multiple `activation_at` has the same block number the last one is used
    /// (follows `BTreeMap` semantics).
    #[inline]
    pub fn cost(&self, input: &[u8], at: u64) -> U256 {
        if let Some((_, pricer)) = self.pricer.range(0..=at).last() {
            pricer.cost(input)
        } else {
            U256::zero()
        }
    }

    /// Simple forwarder for execute.
    #[inline]
    pub fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
        self.native.execute(input, output)
    }

    /// Whether the builtin is activated at the given block number.
    #[inline]
    pub fn is_active(&self, at: u64) -> bool {
        self.pricer.range(0..=at).last().is_some()
    }
}

impl TryFrom<ethjson::spec::builtin::Builtin> for Builtin {
    type Error = String;

    fn try_from(b: ethjson::spec::builtin::Builtin) -> Result<Self, Self::Error> {
        let native = EthereumBuiltin::from_str(&b.name)?;
        let mut pricer = BTreeMap::new();

        for (activate_at, p) in b.pricing {
            pricer.insert(activate_at, p.price.into());
        }

        Ok(Self { pricer, native })
    }
}

impl From<ethjson::spec::builtin::Pricing> for Pricing {
    fn from(pricing: ethjson::spec::builtin::Pricing) -> Self {
        match pricing {
            ethjson::spec::builtin::Pricing::Blake2F { gas_per_round } => {
                Pricing::Blake2F(gas_per_round)
            }
            ethjson::spec::builtin::Pricing::Linear(linear) => Pricing::Linear(Linear {
                base: linear.base,
                word: linear.word,
            }),
            ethjson::spec::builtin::Pricing::Modexp(exp) => Pricing::Modexp(ModexpPricer {
                divisor: if exp.divisor == 0 {
                    warn!(target: "builtin", "Zero modexp divisor specified. Falling back to default: 10.");
                    10
                } else {
                    exp.divisor
                },
            }),
            ethjson::spec::builtin::Pricing::AltBn128Pairing(pricer) => {
                Pricing::AltBn128Pairing(AltBn128PairingPricer {
                    price: AltBn128PairingPrice {
                        base: pricer.base,
                        pair: pricer.pair,
                    },
                })
            }
            ethjson::spec::builtin::Pricing::AltBn128ConstOperations(pricer) => {
                Pricing::AltBn128ConstOperations(AltBn128ConstOperations {
                    price: pricer.price,
                })
            }
            ethjson::spec::builtin::Pricing::Bls12ConstOperations(pricer) => {
                Pricing::Bls12ConstOperations(Bls12ConstOperations {
                    price: pricer.price,
                })
            }
            ethjson::spec::builtin::Pricing::Bls12Pairing(pricer) => {
                Pricing::Bls12Pairing(Bls12PairingPricer {
                    price: Bls12PairingPrice {
                        base: pricer.base,
                        pair: pricer.pair,
                    },
                })
            }
            ethjson::spec::builtin::Pricing::Bls12G1Multiexp(pricer) => {
                Pricing::Bls12MultiexpG1(Bls12MultiexpPricerG1 {
                    base_price: Bls12ConstOperations { price: pricer.base },
                    _marker: std::marker::PhantomData,
                })
            }
            ethjson::spec::builtin::Pricing::Bls12G2Multiexp(pricer) => {
                Pricing::Bls12MultiexpG2(Bls12MultiexpPricerG2 {
                    base_price: Bls12ConstOperations { price: pricer.base },
                    _marker: std::marker::PhantomData,
                })
            }
        }
    }
}

/// Ethereum builtins:
enum EthereumBuiltin {
    /// The identity function
    Identity(Identity),
    /// ec recovery
    EcRecover(EcRecover),
    /// sha256
    Sha256(Sha256),
    /// ripemd160
    Ripemd160(Ripemd160),
    /// modexp (EIP 198)
    Modexp(Modexp),
    /// alt_bn128_add
    Bn128Add(Bn128Add),
    /// alt_bn128_mul
    Bn128Mul(Bn128Mul),
    /// alt_bn128_pairing
    Bn128Pairing(Bn128Pairing),
    /// blake2_f (The Blake2 compression function F, EIP-152)
    Blake2F(Blake2F),
    /// bls12_381 addition in g1
    Bls12G1Add(Bls12G1Add),
    /// bls12_381 multiplication in g1
    Bls12G1Mul(Bls12G1Mul),
    /// bls12_381 multiexponentiation in g1
    Bls12G1MultiExp(Bls12G1MultiExp),
    /// bls12_381 addition in g2
    Bls12G2Add(Bls12G2Add),
    /// bls12_381 multiplication in g2
    Bls12G2Mul(Bls12G2Mul),
    /// bls12_381 multiexponentiation in g2
    Bls12G2MultiExp(Bls12G2MultiExp),
    /// bls12_381 pairing
    Bls12Pairing(Bls12Pairing),
    /// bls12_381 fp to g1 mapping
    Bls12MapFpToG1(Bls12MapFpToG1),
    /// bls12_381 fp2 to g2 mapping
    Bls12MapFp2ToG2(Bls12MapFp2ToG2),
}

impl FromStr for EthereumBuiltin {
    type Err = String;

    fn from_str(name: &str) -> Result<EthereumBuiltin, Self::Err> {
        match name {
            "identity" => Ok(EthereumBuiltin::Identity(Identity)),
            "ecrecover" => Ok(EthereumBuiltin::EcRecover(EcRecover)),
            "sha256" => Ok(EthereumBuiltin::Sha256(Sha256)),
            "ripemd160" => Ok(EthereumBuiltin::Ripemd160(Ripemd160)),
            "modexp" => Ok(EthereumBuiltin::Modexp(Modexp)),
            "alt_bn128_add" => Ok(EthereumBuiltin::Bn128Add(Bn128Add)),
            "alt_bn128_mul" => Ok(EthereumBuiltin::Bn128Mul(Bn128Mul)),
            "alt_bn128_pairing" => Ok(EthereumBuiltin::Bn128Pairing(Bn128Pairing)),
            "blake2_f" => Ok(EthereumBuiltin::Blake2F(Blake2F)),
            "bls12_381_g1_add" => Ok(EthereumBuiltin::Bls12G1Add(Bls12G1Add)),
            "bls12_381_g1_mul" => Ok(EthereumBuiltin::Bls12G1Mul(Bls12G1Mul)),
            "bls12_381_g1_multiexp" => Ok(EthereumBuiltin::Bls12G1MultiExp(Bls12G1MultiExp)),
            "bls12_381_g2_add" => Ok(EthereumBuiltin::Bls12G2Add(Bls12G2Add)),
            "bls12_381_g2_mul" => Ok(EthereumBuiltin::Bls12G2Mul(Bls12G2Mul)),
            "bls12_381_g2_multiexp" => Ok(EthereumBuiltin::Bls12G2MultiExp(Bls12G2MultiExp)),
            "bls12_381_pairing" => Ok(EthereumBuiltin::Bls12Pairing(Bls12Pairing)),
            "bls12_381_fp_to_g1" => Ok(EthereumBuiltin::Bls12MapFpToG1(Bls12MapFpToG1)),
            "bls12_381_fp2_to_g2" => Ok(EthereumBuiltin::Bls12MapFp2ToG2(Bls12MapFp2ToG2)),
            _ => return Err(format!("invalid builtin name: {}", name)),
        }
    }
}

impl Implementation for EthereumBuiltin {
    fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
        match self {
            EthereumBuiltin::Identity(inner) => inner.execute(input, output),
            EthereumBuiltin::EcRecover(inner) => inner.execute(input, output),
            EthereumBuiltin::Sha256(inner) => inner.execute(input, output),
            EthereumBuiltin::Ripemd160(inner) => inner.execute(input, output),
            EthereumBuiltin::Modexp(inner) => inner.execute(input, output),
            EthereumBuiltin::Bn128Add(inner) => inner.execute(input, output),
            EthereumBuiltin::Bn128Mul(inner) => inner.execute(input, output),
            EthereumBuiltin::Bn128Pairing(inner) => inner.execute(input, output),
            EthereumBuiltin::Blake2F(inner) => inner.execute(input, output),
            EthereumBuiltin::Bls12G1Add(inner) => inner.execute(input, output),
            EthereumBuiltin::Bls12G1Mul(inner) => inner.execute(input, output),
            EthereumBuiltin::Bls12G1MultiExp(inner) => inner.execute(input, output),
            EthereumBuiltin::Bls12G2Add(inner) => inner.execute(input, output),
            EthereumBuiltin::Bls12G2Mul(inner) => inner.execute(input, output),
            EthereumBuiltin::Bls12G2MultiExp(inner) => inner.execute(input, output),
            EthereumBuiltin::Bls12Pairing(inner) => inner.execute(input, output),
            EthereumBuiltin::Bls12MapFpToG1(inner) => inner.execute(input, output),
            EthereumBuiltin::Bls12MapFp2ToG2(inner) => inner.execute(input, output),
        }
    }
}

#[derive(Debug)]
pub struct Identity;

#[derive(Debug)]
pub struct EcRecover;

#[derive(Debug)]
pub struct Sha256;

#[derive(Debug)]
pub struct Ripemd160;

#[derive(Debug)]
pub struct Modexp;

#[derive(Debug)]
pub struct Bn128Add;

#[derive(Debug)]
pub struct Bn128Mul;

#[derive(Debug)]
pub struct Bn128Pairing;

#[derive(Debug)]
pub struct Blake2F;

#[derive(Debug)]
/// The Bls12G1Add builtin.
pub struct Bls12G1Add;

#[derive(Debug)]
/// The Bls12G1Mul builtin.
pub struct Bls12G1Mul;

#[derive(Debug)]
/// The Bls12G1MultiExp builtin.
pub struct Bls12G1MultiExp;

#[derive(Debug)]
/// The Bls12G2Add builtin.
pub struct Bls12G2Add;

#[derive(Debug)]
/// The Bls12G2Mul builtin.
pub struct Bls12G2Mul;

#[derive(Debug)]
/// The Bls12G2MultiExp builtin.
pub struct Bls12G2MultiExp;

#[derive(Debug)]
/// The Bls12Pairing builtin.
pub struct Bls12Pairing;

#[derive(Debug)]
/// The Bls12MapFpToG1 builtin.
pub struct Bls12MapFpToG1;

#[derive(Debug)]
/// The Bls12MapFp2ToG2 builtin.
pub struct Bls12MapFp2ToG2;

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
            27 | 28 if v.0[..31] == [0; 31] => v[31] - 27,
            _ => {
                return Ok(());
            }
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
            return Err("input length for Blake2 F precompile should be exactly 213 bytes".into());
        }

        let mut cursor = Cursor::new(input);
        let rounds = cursor.read_u32::<BigEndian>().expect(PROOF);

        // state vector, h
        let mut h = [0u64; 8];
        for state_word in &mut h {
            *state_word = cursor.read_u64::<LittleEndian>().expect(PROOF);
        }

        // message block vector, m
        let mut m = [0u64; 16];
        for msg_word in &mut m {
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
                return Err("incorrect final block indicator flag".into());
            }
        };

        compress(&mut h, m, t, f, rounds as usize);

        let mut output_buf = [0u8; 8 * size_of::<u64>()];
        for (i, state_word) in h.iter().enumerate() {
            output_buf[i * 8..(i + 1) * 8].copy_from_slice(&state_word.to_le_bytes());
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
    if exp.peek().is_none() {
        return BigUint::one();
    }

    // 0^n % m, n > 0
    if base.is_zero() {
        return BigUint::zero();
    }

    base %= &modulus;

    // Fast path for base divisible by modulus.
    if base.is_zero() {
        return BigUint::zero();
    }

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
            reader
                .read_exact(&mut buf[..])
                .expect("reading from zero-extended memory cannot fail; qed");
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
                reader
                    .read_exact(&mut buf[..len])
                    .expect("reading from zero-extended memory cannot fail; qed");
                BigUint::from_bytes_be(&buf[..len])
            };

            let base = read_num(&mut reader, base_len);

            let mut exp_buf = vec![0; exp_len];
            reader
                .read_exact(&mut exp_buf[..exp_len])
                .expect("reading from zero-extended memory cannot fail; qed");

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

impl Implementation for Bls12G1Add {
    fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
        let result = EIP2537Executor::g1_add(input);

        match result {
            Ok(result_bytes) => {
                output.write(0, &result_bytes[..]);

                Ok(())
            }
            Err(e) => {
                trace!(target: "builtin", "Bls12G1Add error: {:?}", e);

                Err("Bls12G1Add error")
            }
        }
    }
}

impl Implementation for Bls12G1Mul {
    fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
        let result = EIP2537Executor::g1_mul(input);

        match result {
            Ok(result_bytes) => {
                output.write(0, &result_bytes[..]);

                Ok(())
            }
            Err(e) => {
                trace!(target: "builtin", "Bls12G1Mul error: {:?}", e);

                Err("Bls12G1Mul error")
            }
        }
    }
}

impl Implementation for Bls12G1MultiExp {
    fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
        let result = EIP2537Executor::g1_multiexp(input);

        match result {
            Ok(result_bytes) => {
                output.write(0, &result_bytes[..]);

                Ok(())
            }
            Err(e) => {
                trace!(target: "builtin", "Bls12G1MultiExp error: {:?}", e);

                Err("Bls12G1MultiExp error")
            }
        }
    }
}

impl Implementation for Bls12G2Add {
    fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
        let result = EIP2537Executor::g2_add(input);

        match result {
            Ok(result_bytes) => {
                output.write(0, &result_bytes[..]);

                Ok(())
            }
            Err(e) => {
                trace!(target: "builtin", "Bls12G2Add error: {:?}", e);

                Err("Bls12G2Add error")
            }
        }
    }
}

impl Implementation for Bls12G2Mul {
    fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
        let result = EIP2537Executor::g2_mul(input);

        match result {
            Ok(result_bytes) => {
                output.write(0, &result_bytes[..]);

                Ok(())
            }
            Err(e) => {
                trace!(target: "builtin", "Bls12G2Mul error: {:?}", e);

                Err("Bls12G2Mul error")
            }
        }
    }
}

impl Implementation for Bls12G2MultiExp {
    fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
        let result = EIP2537Executor::g2_multiexp(input);

        match result {
            Ok(result_bytes) => {
                output.write(0, &result_bytes[..]);

                Ok(())
            }
            Err(e) => {
                trace!(target: "builtin", "Bls12G2MultiExp error: {:?}", e);

                Err("Bls12G2MultiExp error")
            }
        }
    }
}

impl Implementation for Bls12Pairing {
    fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
        let result = EIP2537Executor::pair(input);

        match result {
            Ok(result_bytes) => {
                output.write(0, &result_bytes[..]);

                Ok(())
            }
            Err(e) => {
                trace!(target: "builtin", "Bls12Pairing error: {:?}", e);

                Err("Bls12Pairing error")
            }
        }
    }
}

impl Implementation for Bls12MapFpToG1 {
    fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
        let result = EIP2537Executor::map_fp_to_g1(input);

        match result {
            Ok(result_bytes) => {
                output.write(0, &result_bytes[..]);

                Ok(())
            }
            Err(e) => {
                trace!(target: "builtin", "Bls12MapFpToG1 error: {:?}", e);

                Err("Bls12MapFpToG1 error")
            }
        }
    }
}

impl Implementation for Bls12MapFp2ToG2 {
    fn execute(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
        let result = EIP2537Executor::map_fp2_to_g2(input);

        match result {
            Ok(result_bytes) => {
                output.write(0, &result_bytes[..]);

                Ok(())
            }
            Err(e) => {
                trace!(target: "builtin", "Bls12MapFp2ToG2 error: {:?}", e);

                Err("Bls12MapFp2ToG2 error")
            }
        }
    }
}

fn read_fr(reader: &mut io::Chain<&[u8], io::Repeat>) -> Result<bn::Fr, &'static str> {
    let mut buf = [0u8; 32];

    reader
        .read_exact(&mut buf[..])
        .expect("reading from zero-extended memory cannot fail; qed");
    bn::Fr::from_slice(&buf[0..32]).map_err(|_| "Invalid field element")
}

fn read_point(reader: &mut io::Chain<&[u8], io::Repeat>) -> Result<bn::G1, &'static str> {
    use bn::{AffineG1, Fq, Group, G1};

    let mut buf = [0u8; 32];

    reader
        .read_exact(&mut buf[..])
        .expect("reading from zero-extended memory cannot fail; qed");
    let px = Fq::from_slice(&buf[0..32]).map_err(|_| "Invalid point x coordinate")?;

    reader
        .read_exact(&mut buf[..])
        .expect("reading from zero-extended memory cannot fail; qed");
    let py = Fq::from_slice(&buf[0..32]).map_err(|_| "Invalid point y coordinate")?;
    Ok(if px == Fq::zero() && py == Fq::zero() {
        G1::zero()
    } else {
        AffineG1::new(px, py)
            .map_err(|_| "Invalid curve point")?
            .into()
    })
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
            sum.x()
                .to_big_endian(&mut write_buf[0..32])
                .expect("Cannot fail since 0..32 is 32-byte length");
            sum.y()
                .to_big_endian(&mut write_buf[32..64])
                .expect("Cannot fail since 32..64 is 32-byte length");
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
            sum.x()
                .to_big_endian(&mut write_buf[0..32])
                .expect("Cannot fail since 0..32 is 32-byte length");
            sum.y()
                .to_big_endian(&mut write_buf[32..64])
                .expect("Cannot fail since 32..64 is 32-byte length");
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
            return Err("Invalid input length, must be multiple of 192 (3 * (32*2))".into());
        }

        if let Err(err) = self.execute_with_error(input, output) {
            trace!(target: "builtin", "Pairing error: {:?}", err);
            return Err(err);
        }
        Ok(())
    }
}

impl Bn128Pairing {
    fn execute_with_error(&self, input: &[u8], output: &mut BytesRef) -> Result<(), &'static str> {
        use bn::{pairing, AffineG1, AffineG2, Fq, Fq2, Group, Gt, G1, G2};

        let ret_val = if input.is_empty() {
            U256::one()
        } else {
            // (a, b_a, b_b - each 64-byte affine coordinates)
            let elements = input.len() / 192;
            let mut vals = Vec::new();
            for idx in 0..elements {
                let a_x = Fq::from_slice(&input[idx * 192..idx * 192 + 32])
                    .map_err(|_| "Invalid a argument x coordinate")?;

                let a_y = Fq::from_slice(&input[idx * 192 + 32..idx * 192 + 64])
                    .map_err(|_| "Invalid a argument y coordinate")?;

                let b_a_y = Fq::from_slice(&input[idx * 192 + 64..idx * 192 + 96])
                    .map_err(|_| "Invalid b argument imaginary coeff x coordinate")?;

                let b_a_x = Fq::from_slice(&input[idx * 192 + 96..idx * 192 + 128])
                    .map_err(|_| "Invalid b argument imaginary coeff y coordinate")?;

                let b_b_y = Fq::from_slice(&input[idx * 192 + 128..idx * 192 + 160])
                    .map_err(|_| "Invalid b argument real coeff x coordinate")?;

                let b_b_x = Fq::from_slice(&input[idx * 192 + 160..idx * 192 + 192])
                    .map_err(|_| "Invalid b argument real coeff y coordinate")?;

                let b_a = Fq2::new(b_a_x, b_a_y);
                let b_b = Fq2::new(b_b_x, b_b_y);
                let b = if b_a.is_zero() && b_b.is_zero() {
                    G2::zero()
                } else {
                    G2::from(
                        AffineG2::new(b_a, b_b).map_err(|_| "Invalid b argument - not on curve")?,
                    )
                };
                let a = if a_x.is_zero() && a_y.is_zero() {
                    G1::zero()
                } else {
                    G1::from(
                        AffineG1::new(a_x, a_y).map_err(|_| "Invalid a argument - not on curve")?,
                    )
                };
                vals.push((a, b));
            }

            let mul = vals
                .into_iter()
                .fold(Gt::one(), |s, (a, b)| s * pairing(a, b));

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
    use super::{
        modexp as me, BTreeMap, Bls12ConstOperations, Bls12PairingPrice, Bls12PairingPricer,
        Builtin, EthereumBuiltin, FromStr, Implementation, Linear, ModexpPricer, Pricing,
    };
    use ethereum_types::U256;
    use ethjson::spec::builtin::{
        AltBn128Pairing as JsonAltBn128PairingPricing, Builtin as JsonBuiltin,
        Linear as JsonLinearPricing, Pricing as JsonPricing, PricingAt,
    };
    use hex_literal::hex;
    use macros::map;
    use maplit::btreemap;
    use num::{BigUint, One, Zero};
    use parity_bytes::BytesRef;
    use std::convert::TryFrom;

    #[test]
    fn blake2f_cost() {
        let f = Builtin {
            pricer: map![0 => Pricing::Blake2F(123)],
            native: EthereumBuiltin::from_str("blake2_f").unwrap(),
        };
        // 5 rounds
        let input = hex!("0000000548c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
        let mut output = [0u8; 64];
        f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
            .unwrap();

        assert_eq!(f.cost(&input[..], 0), U256::from(123 * 5));
    }

    #[test]
    fn blake2f_cost_on_invalid_length() {
        let f = Builtin {
            pricer: map![0 => Pricing::Blake2F(123)],
            native: EthereumBuiltin::from_str("blake2_f").expect("known builtin"),
        };
        // invalid input (too short)
        let input = hex!("00");

        assert_eq!(f.cost(&input[..], 0), U256::from(0));
    }

    #[test]
    fn blake2_f_is_err_on_invalid_length() {
        let blake2 = EthereumBuiltin::from_str("blake2_f").unwrap();
        // Test vector 1 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-1
        let input = hex!("00000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
        let mut out = [0u8; 64];

        let result = blake2.execute(&input[..], &mut BytesRef::Fixed(&mut out[..]));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "input length for Blake2 F precompile should be exactly 213 bytes"
        );
    }

    #[test]
    fn blake2_f_is_err_on_invalid_length_2() {
        let blake2 = EthereumBuiltin::from_str("blake2_f").unwrap();
        // Test vector 2 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-2
        let input = hex!("000000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
        let mut out = [0u8; 64];

        let result = blake2.execute(&input[..], &mut BytesRef::Fixed(&mut out[..]));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "input length for Blake2 F precompile should be exactly 213 bytes"
        );
    }

    #[test]
    fn blake2_f_is_err_on_bad_finalization_flag() {
        let blake2 = EthereumBuiltin::from_str("blake2_f").unwrap();
        // Test vector 3 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-3
        let input = hex!("0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000002");
        let mut out = [0u8; 64];

        let result = blake2.execute(&input[..], &mut BytesRef::Fixed(&mut out[..]));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "incorrect final block indicator flag");
    }

    #[test]
    fn blake2_f_zero_rounds_is_ok_test_vector_4() {
        let blake2 = EthereumBuiltin::from_str("blake2_f").unwrap();
        // Test vector 4 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-4
        let input = hex!("0000000048c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
        let expected = hex!("08c9bcf367e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d282e6ad7f520e511f6c3e2b8c68059b9442be0454267ce079217e1319cde05b");
        let mut output = [0u8; 64];
        blake2
            .execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
            .unwrap();
        assert_eq!(&output[..], &expected[..]);
    }

    #[test]
    fn blake2_f_test_vector_5() {
        let blake2 = EthereumBuiltin::from_str("blake2_f").unwrap();
        // Test vector 5 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-5
        let input = hex!("0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
        let expected = hex!("ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923");
        let mut out = [0u8; 64];
        blake2
            .execute(&input[..], &mut BytesRef::Fixed(&mut out[..]))
            .unwrap();
        assert_eq!(&out[..], &expected[..]);
    }

    #[test]
    fn blake2_f_test_vector_6() {
        let blake2 = EthereumBuiltin::from_str("blake2_f").unwrap();
        // Test vector 6 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-6
        let input = hex!("0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000");
        let expected = hex!("75ab69d3190a562c51aef8d88f1c2775876944407270c42c9844252c26d2875298743e7f6d5ea2f2d3e8d226039cd31b4e426ac4f2d3d666a610c2116fde4735");
        let mut out = [0u8; 64];
        blake2
            .execute(&input[..], &mut BytesRef::Fixed(&mut out[..]))
            .unwrap();
        assert_eq!(&out[..], &expected[..]);
    }

    #[test]
    fn blake2_f_test_vector_7() {
        let blake2 = EthereumBuiltin::from_str("blake2_f").unwrap();
        // Test vector 7 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-7
        let input = hex!("0000000148c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
        let expected = hex!("b63a380cb2897d521994a85234ee2c181b5f844d2c624c002677e9703449d2fba551b3a8333bcdf5f2f7e08993d53923de3d64fcc68c034e717b9293fed7a421");
        let mut out = [0u8; 64];
        blake2
            .execute(&input[..], &mut BytesRef::Fixed(&mut out[..]))
            .unwrap();
        assert_eq!(&out[..], &expected[..]);
    }

    #[ignore]
    #[test]
    fn blake2_f_test_vector_8() {
        let blake2 = EthereumBuiltin::from_str("blake2_f").unwrap();
        // Test vector 8 and expected output from https://github.com/ethereum/EIPs/blob/master/EIPS/eip-152.md#test-vector-8
        // Note this test is slow, 4294967295/0xffffffff rounds take a while.
        let input = hex!("ffffffff48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001");
        let expected = hex!("fc59093aafa9ab43daae0e914c57635c5402d8e3d2130eb9b3cc181de7f0ecf9b22bf99a7815ce16419e200e01846e6b5df8cc7703041bbceb571de6631d2615");
        let mut out = [0u8; 64];
        blake2
            .execute(&input[..], &mut BytesRef::Fixed(&mut out[..]))
            .unwrap();
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
        assert_eq!(
            me(base, exp.to_bytes_be(), modulus),
            BigUint::parse_bytes(b"55", 10).unwrap()
        );
    }

    #[test]
    fn identity() {
        let f = EthereumBuiltin::from_str("identity").unwrap();
        let i = [0u8, 1, 2, 3];

        let mut o2 = [255u8; 2];
        f.execute(&i[..], &mut BytesRef::Fixed(&mut o2[..]))
            .expect("Builtin should not fail");
        assert_eq!(i[0..2], o2);

        let mut o4 = [255u8; 4];
        f.execute(&i[..], &mut BytesRef::Fixed(&mut o4[..]))
            .expect("Builtin should not fail");
        assert_eq!(i, o4);

        let mut o8 = [255u8; 8];
        f.execute(&i[..], &mut BytesRef::Fixed(&mut o8[..]))
            .expect("Builtin should not fail");
        assert_eq!(i, o8[..4]);
        assert_eq!([255u8; 4], o8[4..]);
    }

    #[test]
    fn sha256() {
        let f = EthereumBuiltin::from_str("sha256").unwrap();
        let i = [0u8; 0];

        let mut o = [255u8; 32];
        f.execute(&i[..], &mut BytesRef::Fixed(&mut o[..]))
            .expect("Builtin should not fail");
        assert_eq!(
            &o[..],
            hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        );

        let mut o8 = [255u8; 8];
        f.execute(&i[..], &mut BytesRef::Fixed(&mut o8[..]))
            .expect("Builtin should not fail");
        assert_eq!(&o8[..], hex!("e3b0c44298fc1c14"));

        let mut o34 = [255u8; 34];
        f.execute(&i[..], &mut BytesRef::Fixed(&mut o34[..]))
            .expect("Builtin should not fail");
        assert_eq!(
            &o34[..],
            &hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855ffff")[..]
        );

        let mut ov = vec![];
        f.execute(&i[..], &mut BytesRef::Flexible(&mut ov))
            .expect("Builtin should not fail");
        assert_eq!(
            &ov[..],
            &hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")[..]
        );
    }

    #[test]
    fn ripemd160() {
        let f = EthereumBuiltin::from_str("ripemd160").unwrap();
        let i = [0u8; 0];

        let mut o = [255u8; 32];
        f.execute(&i[..], &mut BytesRef::Fixed(&mut o[..]))
            .expect("Builtin should not fail");
        assert_eq!(
            &o[..],
            &hex!("0000000000000000000000009c1185a5c5e9fc54612808977ee8f548b2258d31")[..]
        );

        let mut o8 = [255u8; 8];
        f.execute(&i[..], &mut BytesRef::Fixed(&mut o8[..]))
            .expect("Builtin should not fail");
        assert_eq!(&o8[..], &hex!("0000000000000000")[..]);

        let mut o34 = [255u8; 34];
        f.execute(&i[..], &mut BytesRef::Fixed(&mut o34[..]))
            .expect("Builtin should not fail");
        assert_eq!(
            &o34[..],
            &hex!("0000000000000000000000009c1185a5c5e9fc54612808977ee8f548b2258d31ffff")[..]
        );
    }

    #[test]
    fn ecrecover() {
        let f = EthereumBuiltin::from_str("ecrecover").unwrap();

        let i = hex!("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03");

        let mut o = [255u8; 32];
        f.execute(&i[..], &mut BytesRef::Fixed(&mut o[..]))
            .expect("Builtin should not fail");
        assert_eq!(
            &o[..],
            &hex!("000000000000000000000000c08b5542d177ac6686946920409741463a15dddb")[..]
        );

        let mut o8 = [255u8; 8];
        f.execute(&i[..], &mut BytesRef::Fixed(&mut o8[..]))
            .expect("Builtin should not fail");
        assert_eq!(&o8[..], &hex!("0000000000000000")[..]);

        let mut o34 = [255u8; 34];
        f.execute(&i[..], &mut BytesRef::Fixed(&mut o34[..]))
            .expect("Builtin should not fail");
        assert_eq!(
            &o34[..],
            &hex!("000000000000000000000000c08b5542d177ac6686946920409741463a15dddbffff")[..]
        );

        let i_bad = hex!("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001a650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03");
        let mut o = [255u8; 32];
        f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..]))
            .expect("Builtin should not fail");
        assert_eq!(
            &o[..],
            &hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")[..]
        );

        let i_bad = hex!("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b000000000000000000000000000000000000000000000000000000000000001b0000000000000000000000000000000000000000000000000000000000000000");
        let mut o = [255u8; 32];
        f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..]))
            .expect("Builtin should not fail");
        assert_eq!(
            &o[..],
            &hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")[..]
        );

        let i_bad = hex!("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b");
        let mut o = [255u8; 32];
        f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..]))
            .expect("Builtin should not fail");
        assert_eq!(
            &o[..],
            &hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")[..]
        );

        let i_bad = hex!("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001bffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000000000001b");
        let mut o = [255u8; 32];
        f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..]))
            .expect("Builtin should not fail");
        assert_eq!(
            &o[..],
            &hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")[..]
        );

        let i_bad = hex!("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b000000000000000000000000000000000000000000000000000000000000001bffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        let mut o = [255u8; 32];
        f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..]))
            .expect("Builtin should not fail");
        assert_eq!(
            &o[..],
            &hex!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")[..]
        );

        // TODO: Should this (corrupted version of the above) fail rather than returning some address?
        /*	let i_bad = FromHex::from_hex("48173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03").unwrap();
        let mut o = [255u8; 32];
        f.execute(&i_bad[..], &mut BytesRef::Fixed(&mut o[..]));
        assert_eq!(&o[..], &(FromHex::from_hex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap())[..]);*/
    }

    #[test]
    fn modexp() {
        let f = Builtin {
            pricer: map![0 => Pricing::Modexp(ModexpPricer { divisor: 20 })],
            native: EthereumBuiltin::from_str("modexp").unwrap(),
        };

        // test for potential gas cost multiplication overflow
        {
            let input = hex!("0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000003b27bafd00000000000000000000000000000000000000000000000000000000503c8ac3");
            let expected_cost = U256::max_value();
            assert_eq!(f.cost(&input[..], 0), expected_cost);
        }

        // test for potential exp len overflow
        {
            let input = hex!(
                "
				00000000000000000000000000000000000000000000000000000000000000ff
				2a1e530000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000"
            );

            let mut output = vec![0u8; 32];
            let expected = hex!("0000000000000000000000000000000000000000000000000000000000000000");
            let expected_cost = U256::max_value();

            f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
                .expect("Builtin should fail");
            assert_eq!(output, expected);
            assert_eq!(f.cost(&input[..], 0), expected_cost);
        }

        // fermat's little theorem example.
        {
            let input = hex!(
                "
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

            f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
                .expect("Builtin should not fail");
            assert_eq!(output, expected);
            assert_eq!(f.cost(&input[..], 0), expected_cost.into());
        }

        // second example from EIP: zero base.
        {
            let input = hex!(
                "
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000020
				0000000000000000000000000000000000000000000000000000000000000020
				fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e
				fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f"
            );

            let mut output = vec![0u8; 32];
            let expected = hex!("0000000000000000000000000000000000000000000000000000000000000000");
            let expected_cost = 13056;

            f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
                .expect("Builtin should not fail");
            assert_eq!(output, expected);
            assert_eq!(f.cost(&input[..], 0), expected_cost.into());
        }

        // another example from EIP: zero-padding
        {
            let input = hex!(
                "
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

            f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
                .expect("Builtin should not fail");
            assert_eq!(output, expected);
            assert_eq!(f.cost(&input[..], 0), expected_cost.into());
        }

        // zero-length modulus.
        {
            let input = hex!(
                "
				0000000000000000000000000000000000000000000000000000000000000001
				0000000000000000000000000000000000000000000000000000000000000002
				0000000000000000000000000000000000000000000000000000000000000000
				03
				ffff"
            );

            let mut output = vec![];
            let expected_cost = 0;

            f.execute(&input[..], &mut BytesRef::Flexible(&mut output))
                .expect("Builtin should not fail");
            assert_eq!(output.len(), 0); // shouldn't have written any output.
            assert_eq!(f.cost(&input[..], 0), expected_cost.into());
        }
    }

    #[test]
    fn bn128_add() {
        let f = Builtin {
            pricer: map![0 => Pricing::Linear(Linear { base: 0, word: 0 })],
            native: EthereumBuiltin::from_str("alt_bn128_add").unwrap(),
        };

        // zero-points additions
        {
            let input = hex!(
                "
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000"
            );

            let mut output = vec![0u8; 64];
            let expected = hex!(
                "
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000"
            );

            f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
                .expect("Builtin should not fail");
            assert_eq!(output, &expected[..]);
        }

        // no input, should not fail
        {
            let mut empty = [0u8; 0];
            let input = BytesRef::Fixed(&mut empty);

            let mut output = vec![0u8; 64];
            let expected = hex!(
                "
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000"
            );

            f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
                .expect("Builtin should not fail");
            assert_eq!(output, &expected[..]);
        }

        // should fail - point not on curve
        {
            let input = hex!(
                "
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
            pricer: map![0 => Pricing::Linear(Linear { base: 0, word: 0 })],
            native: EthereumBuiltin::from_str("alt_bn128_mul").unwrap(),
        };

        // zero-point multiplication
        {
            let input = hex!(
                "
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000
				0200000000000000000000000000000000000000000000000000000000000000"
            );

            let mut output = vec![0u8; 64];
            let expected = hex!(
                "
				0000000000000000000000000000000000000000000000000000000000000000
				0000000000000000000000000000000000000000000000000000000000000000"
            );

            f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
                .expect("Builtin should not fail");
            assert_eq!(output, &expected[..]);
        }

        // should fail - point not on curve
        {
            let input = hex!(
                "
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
            pricer: map![0 => Pricing::Linear(Linear { base: 0, word: 0 })],
            native: EthereumBuiltin::from_str("alt_bn128_pairing").unwrap(),
        }
    }

    fn empty_test(f: Builtin, expected: Vec<u8>) {
        let mut empty = [0u8; 0];
        let input = BytesRef::Fixed(&mut empty);

        let mut output = vec![0u8; expected.len()];

        f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
            .expect("Builtin should not fail");
        assert_eq!(output, expected);
    }

    fn error_test(f: Builtin, input: &[u8], msg_contains: Option<&str>) {
        let mut output = vec![0u8; 64];
        let res = f.execute(input, &mut BytesRef::Fixed(&mut output[..]));
        if let Some(msg) = msg_contains {
            if let Err(e) = res {
                if !e.contains(msg) {
                    panic!(
                        "There should be error containing '{}' here, but got: '{}'",
                        msg, e
                    );
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
            &hex!(
                "
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
            &hex!(
                "
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
        let _ = EthereumBuiltin::from_str("foo").unwrap();
    }

    #[test]
    fn is_active() {
        let pricer = Pricing::Linear(Linear { base: 10, word: 20 });
        let b = Builtin {
            pricer: map![100_000 => pricer],
            native: EthereumBuiltin::from_str("identity").unwrap(),
        };

        assert!(!b.is_active(99_999));
        assert!(b.is_active(100_000));
        assert!(b.is_active(100_001));
    }

    #[test]
    fn from_named_linear() {
        let pricer = Pricing::Linear(Linear { base: 10, word: 20 });
        let b = Builtin {
            pricer: map![0 => pricer],
            native: EthereumBuiltin::from_str("identity").unwrap(),
        };

        assert_eq!(b.cost(&[0; 0], 0), U256::from(10));
        assert_eq!(b.cost(&[0; 1], 0), U256::from(30));
        assert_eq!(b.cost(&[0; 32], 0), U256::from(30));
        assert_eq!(b.cost(&[0; 33], 0), U256::from(50));

        let i = [0u8, 1, 2, 3];
        let mut o = [255u8; 4];
        b.execute(&i[..], &mut BytesRef::Fixed(&mut o[..]))
            .expect("Builtin should not fail");
        assert_eq!(i, o);
    }

    #[test]
    fn from_json() {
        let b = Builtin::try_from(ethjson::spec::Builtin {
            name: "identity".to_owned(),
            pricing: map![
                0 => PricingAt {
                    info: None,
                    price: JsonPricing::Linear(JsonLinearPricing { base: 10, word: 20 })
                }
            ],
        })
        .expect("known builtin");

        assert_eq!(b.cost(&[0; 0], 0), U256::from(10));
        assert_eq!(b.cost(&[0; 1], 0), U256::from(30));
        assert_eq!(b.cost(&[0; 32], 0), U256::from(30));
        assert_eq!(b.cost(&[0; 33], 0), U256::from(50));

        let i = [0u8, 1, 2, 3];
        let mut o = [255u8; 4];
        b.execute(&i[..], &mut BytesRef::Fixed(&mut o[..]))
            .expect("Builtin should not fail");
        assert_eq!(i, o);
    }

    #[test]
    fn bn128_pairing_eip1108_transition() {
        let b = Builtin::try_from(JsonBuiltin {
            name: "alt_bn128_pairing".to_owned(),
            pricing: map![
                10 => PricingAt {
                    info: None,
                    price: JsonPricing::AltBn128Pairing(JsonAltBn128PairingPricing {
                        base: 100_000,
                        pair: 80_000,
                    }),
                },
                20 => PricingAt {
                    info: None,
                    price: JsonPricing::AltBn128Pairing(JsonAltBn128PairingPricing {
                        base: 45_000,
                        pair: 34_000,
                    }),
                }
            ],
        })
        .unwrap();

        assert_eq!(
            b.cost(&[0; 192 * 3], 10),
            U256::from(340_000),
            "80 000 * 3 + 100 000 == 340 000"
        );
        assert_eq!(
            b.cost(&[0; 192 * 7], 20),
            U256::from(283_000),
            "34 000 * 7 + 45 000 == 283 000"
        );
    }

    #[test]
    fn bn128_add_eip1108_transition() {
        let b = Builtin::try_from(JsonBuiltin {
            name: "alt_bn128_add".to_owned(),
            pricing: map![
                10 => PricingAt {
                    info: None,
                    price: JsonPricing::Linear(JsonLinearPricing {
                        base: 500,
                        word: 0,
                    }),
                },
                20 => PricingAt {
                    info: None,
                    price: JsonPricing::Linear(JsonLinearPricing {
                        base: 150,
                        word: 0,
                    }),
                }
            ],
        })
        .unwrap();

        assert_eq!(b.cost(&[0; 192], 10), U256::from(500));
        assert_eq!(
            b.cost(&[0; 10], 20),
            U256::from(150),
            "after istanbul hardfork gas cost for add should be 150"
        );
    }

    #[test]
    fn bn128_mul_eip1108_transition() {
        let b = Builtin::try_from(JsonBuiltin {
            name: "alt_bn128_mul".to_owned(),
            pricing: map![
                10 => PricingAt {
                    info: None,
                    price: JsonPricing::Linear(JsonLinearPricing {
                        base: 40_000,
                        word: 0,
                    }),
                },
                20 => PricingAt {
                    info: None,
                    price: JsonPricing::Linear(JsonLinearPricing {
                        base: 6_000,
                        word: 0,
                    }),
                }
            ],
        })
        .unwrap();

        assert_eq!(b.cost(&[0; 192], 10), U256::from(40_000));
        assert_eq!(
            b.cost(&[0; 10], 20),
            U256::from(6_000),
            "after istanbul hardfork gas cost for mul should be 6 000"
        );
    }

    #[test]
    fn multimap_use_most_recent_on_activate() {
        let b = Builtin::try_from(JsonBuiltin {
            name: "alt_bn128_mul".to_owned(),
            pricing: map![
                10 => PricingAt {
                    info: None,
                    price: JsonPricing::Linear(JsonLinearPricing {
                        base: 40_000,
                        word: 0,
                    }),
                },
                20 => PricingAt {
                    info: None,
                    price: JsonPricing::Linear(JsonLinearPricing {
                        base: 6_000,
                        word: 0,
                    })
                },
                100 => PricingAt {
                    info: None,
                    price: JsonPricing::Linear(JsonLinearPricing {
                        base: 1_337,
                        word: 0,
                    })
                }
            ],
        })
        .unwrap();

        assert_eq!(
            b.cost(&[0; 2], 0),
            U256::zero(),
            "not activated yet; should be zero"
        );
        assert_eq!(b.cost(&[0; 3], 10), U256::from(40_000), "use price #1");
        assert_eq!(b.cost(&[0; 4], 20), U256::from(6_000), "use price #2");
        assert_eq!(b.cost(&[0; 1], 99), U256::from(6_000), "use price #2");
        assert_eq!(b.cost(&[0; 1], 100), U256::from(1_337), "use price #3");
        assert_eq!(
            b.cost(&[0; 1], u64::max_value()),
            U256::from(1_337),
            "use price #3 indefinitely"
        );
    }

    #[test]
    fn multimap_use_last_with_same_activate_at() {
        let b = Builtin::try_from(JsonBuiltin {
            name: "alt_bn128_mul".to_owned(),
            pricing: map![
                1 => PricingAt {
                    info: None,
                    price: JsonPricing::Linear(JsonLinearPricing {
                        base: 40_000,
                        word: 0,
                    }),
                },
                1 => PricingAt {
                    info: None,
                    price: JsonPricing::Linear(JsonLinearPricing {
                        base: 6_000,
                        word: 0,
                    }),
                },
                1 => PricingAt {
                    info: None,
                    price: JsonPricing::Linear(JsonLinearPricing {
                        base: 1_337,
                        word: 0,
                    }),
                }
            ],
        })
        .unwrap();

        assert_eq!(b.cost(&[0; 1], 0), U256::from(0), "not activated yet");
        assert_eq!(b.cost(&[0; 1], 1), U256::from(1_337));
    }

    #[test]
    fn bls12_381_g1_add() {
        let f = Builtin {
            pricer: btreemap![0 => Pricing::Bls12ConstOperations(Bls12ConstOperations{price: 1})],
            native: EthereumBuiltin::from_str("bls12_381_g1_add").unwrap(),
        };

        let input = hex!("
			00000000000000000000000000000000117dbe419018f67844f6a5e1b78a1e597283ad7b8ee7ac5e58846f5a5fd68d0da99ce235a91db3ec1cf340fe6b7afcdb
			0000000000000000000000000000000013316f23de032d25e912ae8dc9b54c8dba1be7cecdbb9d2228d7e8f652011d46be79089dd0a6080a73c82256ce5e4ed2
			000000000000000000000000000000000441e7f7f96198e4c23bd5eb16f1a7f045dbc8c53219ab2bcea91d3a027e2dfe659feac64905f8b9add7e4bfc91bec2b
			0000000000000000000000000000000005fc51bb1b40c87cd4292d4b66f8ca5ce4ef9abd2b69d4464b4879064203bda7c9fc3f896a3844ebc713f7bb20951d95
		");
        let expected = hex!("
			0000000000000000000000000000000016b8ab56b45a9294466809b8e858c1ad15ad0d52cfcb62f8f5753dc94cee1de6efaaebce10701e3ec2ecaa9551024ea
			600000000000000000000000000000000124571eec37c0b1361023188d66ec17c1ec230d31b515e0e81e599ec19e40c8a7c8cdea9735bc3d8b4e37ca7e5dd71f6
		");

        let mut output = [0u8; 128];

        f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
            .expect("Builtin should not fail");
        assert_eq!(&output[..], &expected[..]);
    }

    #[test]
    fn bls12_381_g1_mul() {
        let f = Builtin {
            pricer: btreemap![0 => Pricing::Bls12ConstOperations(Bls12ConstOperations{price: 1})],
            native: EthereumBuiltin::from_str("bls12_381_g1_mul").unwrap(),
        };

        let input = hex!("
			000000000000000000000000000000000b3a1dfe2d1b62538ed49648cb2a8a1d66bdc4f7a492eee59942ab810a306876a7d49e5ac4c6bb1613866c158ded993e
			000000000000000000000000000000001300956110f47ca8e2aacb30c948dfd046bf33f69bf54007d76373c5a66019454da45e3cf14ce2b9d53a50c9b4366aa3
			ac23d04ee3acc757aae6795532ce4c9f34534e506a4d843a26b052a040c79659
		");
        let expected = hex!("
			000000000000000000000000000000001227b7021e9d3dc8bcbf5b346fc503f7f8576965769c5e22bb70056eef03c84b8c80290ae9ce20345770290c55549bce
			00000000000000000000000000000000188ddbbfb4ad2d34a8d3dc0ec92b70b63caa73ad7dea0cc9740bac2309b4bb11107912bd086379746e9a9bcd26d4db58
		");

        let mut output = [0u8; 128];

        f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
            .expect("Builtin should not fail");
        assert_eq!(&output[..], &expected[..]);
    }

    #[test]
    fn bls12_381_g1_multiexp() {
        let f = Builtin {
            pricer: btreemap![0 => Pricing::Bls12ConstOperations(Bls12ConstOperations{price: 1})],
            native: EthereumBuiltin::from_str("bls12_381_g1_multiexp").unwrap(),
        };
        let input = hex!("
			0000000000000000000000000000000012196c5a43d69224d8713389285f26b98f86ee910ab3dd668e413738282003cc5b7357af9a7af54bb713d62255e80f56
			0000000000000000000000000000000006ba8102bfbeea4416b710c73e8cce3032c31c6269c44906f8ac4f7874ce99fb17559992486528963884ce429a992fee
			b3c940fe79b6966489b527955de7599194a9ac69a6ff58b8d99e7b1084f0464e
			00000000000000000000000000000000117dbe419018f67844f6a5e1b78a1e597283ad7b8ee7ac5e58846f5a5fd68d0da99ce235a91db3ec1cf340fe6b7afcdb
			0000000000000000000000000000000013316f23de032d25e912ae8dc9b54c8dba1be7cecdbb9d2228d7e8f652011d46be79089dd0a6080a73c82256ce5e4ed2
			4d0e25bf3f6fc9f4da25d21fdc71773f1947b7a8a775b8177f7eca990b05b71d
			0000000000000000000000000000000008ab7b556c672db7883ec47efa6d98bb08cec7902ebb421aac1c31506b177ac444ffa2d9b400a6f1cbdc6240c607ee11
			0000000000000000000000000000000016b7fa9adf4addc2192271ce7ad3c8d8f902d061c43b7d2e8e26922009b777855bffabe7ed1a09155819eabfa87f276f
			973f40c12c92b703d7b7848ef8b4466d40823aad3943a312b57432b91ff68be1
			0000000000000000000000000000000015ff9a232d9b5a8020a85d5fe08a1dcfb73ece434258fe0e2fddf10ddef0906c42dcb5f5d62fc97f934ba900f17beb33
			0000000000000000000000000000000009cfe4ee2241d9413c616462d7bac035a6766aeaab69c81e094d75b840df45d7e0dfac0265608b93efefb9a8728b98e4
			4c51f97bcdda93904ae26991b471e9ea942e2b5b8ed26055da11c58bc7b5002a
			0000000000000000000000000000000017a17b82e3bfadf3250210d8ef572c02c3610d65ab4d7366e0b748768a28ee6a1b51f77ed686a64f087f36f641e7dca9
			00000000000000000000000000000000077ea73d233ccea51dc4d5acecf6d9332bf17ae51598f4b394a5f62fb387e9c9aa1d6823b64a074f5873422ca57545d3
			8964d5867927bc3e35a0b4c457482373969bff5edff8a781d65573e07fd87b89
			000000000000000000000000000000000c1243478f4fbdc21ea9b241655947a28accd058d0cdb4f9f0576d32f09dddaf0850464550ff07cab5927b3e4c863ce9
			0000000000000000000000000000000015fb54db10ffac0b6cd374eb7168a8cb3df0a7d5f872d8e98c1f623deb66df5dd08ff4c3658f2905ec8bd02598bd4f90
			787c38b944eadbd03fd3187f450571740f6cd00e5b2e560165846eb800e5c944
			000000000000000000000000000000000328f09584b6d6c98a709fc22e184123994613aca95a28ac53df8523b92273eb6f4e2d9b2a7dcebb474604d54a210719
			000000000000000000000000000000001220ebde579911fe2e707446aaad8d3789fae96ae2e23670a4fd856ed82daaab704779eb4224027c1ed9460f39951a1b
			aaee7ae2a237e8e53560c79e7baa9adf9c00a0ea4d6f514e7a6832eb15cef1e1
			0000000000000000000000000000000002ebfa98aa92c32a29ebe17fcb1819ba82e686abd9371fcee8ea793b4c72b6464085044f818f1f5902396df0122830cb
			00000000000000000000000000000000001184715b8432ed190b459113977289a890f68f6085ea111466af15103c9c02467da33e01d6bff87fd57db6ccba442a
			dac6ed3ef45c1d7d3028f0f89e5458797996d3294b95bebe049b76c7d0db317c
			0000000000000000000000000000000009d6424e002439998e91cd509f85751ad25e574830c564e7568347d19e3f38add0cab067c0b4b0801785a78bcbeaf246
			000000000000000000000000000000000ef6d7db03ee654503b46ff0dbc3297536a422e963bda9871a8da8f4eeb98dedebd6071c4880b4636198f4c2375dc795
			bb30985756c3ca075114c92f231575d6befafe4084517f1166a47376867bd108
			0000000000000000000000000000000002d1cdb93191d1f9f0308c2c55d0208a071f5520faca7c52ab0311dbc9ba563bd33b5dd6baa77bf45ac2c3269e945f48
			00000000000000000000000000000000072a52106e6d7b92c594c4dacd20ef5fab7141e45c231457cd7e71463b2254ee6e72689e516fa6a8f29f2a173ce0a190
			fb730105809f64ea522983d6bbb62f7e2e8cbf702685e9be10e2ef71f8187672
			0000000000000000000000000000000000641642f6801d39a09a536f506056f72a619c50d043673d6d39aa4af11d8e3ded38b9c3bbc970dbc1bd55d68f94b50d
			0000000000000000000000000000000009ab050de356a24aea90007c6b319614ba2f2ed67223b972767117769e3c8e31ee4056494628fb2892d3d37afb6ac943
			b6a9408625b0ca8fcbfb21d34eec2d8e24e9a30d2d3b32d7a37d110b13afbfea
			000000000000000000000000000000000fd4893addbd58fb1bf30b8e62bef068da386edbab9541d198e8719b2de5beb9223d87387af82e8b55bd521ff3e47e2d
			000000000000000000000000000000000f3a923b76473d5b5a53501790cb02597bb778bdacb3805a9002b152d22241ad131d0f0d6a260739cbab2c2fe602870e
			3b77283d0a7bb9e17a27e66851792fdd605cc0a339028b8985390fd024374c76
			0000000000000000000000000000000002cb4b24c8aa799fd7cb1e4ab1aab1372113200343d8526ea7bc64dfaf926baf5d90756a40e35617854a2079cd07fba4
			0000000000000000000000000000000003327ca22bd64ebd673cc6d5b02b2a8804d5353c9d251637c4273ad08d581cc0d58da9bea27c37a0b3f4961dbafd276b
			dd994eae929aee7428fdda2e44f8cb12b10b91c83b22abc8bbb561310b62257c
			00000000000000000000000000000000024ad70f2b2105ca37112858e84c6f5e3ffd4a8b064522faae1ecba38fabd52a6274cb46b00075deb87472f11f2e67d9
			0000000000000000000000000000000010a502c8b2a68aa30d2cb719273550b9a3c283c35b2e18a01b0b765344ffaaa5cb30a1e3e6ecd3a53ab67658a5787681
			7010b134989c8368c7f831f9dd9f9a890e2c1435681107414f2e8637153bbf6a
			0000000000000000000000000000000000704cc57c8e0944326ddc7c747d9e7347a7f6918977132eea269f161461eb64066f773352f293a3ac458dc3ccd5026a
			000000000000000000000000000000001099d3c2bb2d082f2fdcbed013f7ac69e8624f4fcf6dfab3ee9dcf7fbbdb8c49ee79de40e887c0b6828d2496e3a6f768
			94c68bc8d91ac8c489ee87dbfc4b94c93c8bbd5fc04c27db8b02303f3a659054
			00000000000000000000000000000000130535a29392c77f045ac90e47f2e7b3cffff94494fe605aad345b41043f6663ada8e2e7ecd3d06f3b8854ef92212f42
			000000000000000000000000000000001699a3cc1f10cd2ed0dc68eb916b4402e4f12bf4746893bf70e26e209e605ea89e3d53e7ac52bd07713d3c8fc671931d
			b3682accc3939283b870357cf83683350baf73aa0d3d68bda82a0f6ae7e51746
		");
        let expected = hex!("
			000000000000000000000000000000000b370fc4ca67fb0c3c270b1b4c4816ef953cd9f7cf6ad20e88099c40aace9c4bb3f4cd215e5796f65080c69c9f4d2a0f
			0000000000000000000000000000000007203220935ddc0190e2d7a99ec3f9231da550768373f9a5933dffd366f48146f8ea5fe5dee6539d925288083bb5a8f1
		");

        let mut output = [0u8; 128];

        f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
            .expect("Builtin should not fail");
        assert_eq!(&output[..], &expected[..]);
    }

    #[test]
    fn bls12_381_g2_add() {
        let f = Builtin {
            pricer: btreemap![0 => Pricing::Bls12ConstOperations(Bls12ConstOperations{price: 1})],
            native: EthereumBuiltin::from_str("bls12_381_g2_add").unwrap(),
        };
        let input = hex!("
			00000000000000000000000000000000161c595d151a765c7dee03c9210414cdffab84b9078b4b98f9df09be5ec299b8f6322c692214f00ede97958f235c352b
			00000000000000000000000000000000106883e0937cb869e579b513bde8f61020fcf26be38f8b98eae3885cedec2e028970415fc653cf10e64727b7f6232e06
			000000000000000000000000000000000f351a82b733af31af453904874b7ca6252957a1ab51ec7f7b6fff85bbf3331f870a7e72a81594a9930859237e7a154d
			0000000000000000000000000000000012fcf20d1750901f2cfed64fd362f010ee64fafe9ddab406cc352b65829b929881a50514d53247d1cca7d6995d0bc9b2
			00000000000000000000000000000000148b7dfc21521d79ff817c7a0305f1048851e283be13c07d5c04d28b571d48172838399ba539529e8d037ffd1f729558
			0000000000000000000000000000000003015abea326c15098f5205a8b2d3cd74d72dac59d60671ca6ef8c9c714ea61ffdacd46d1024b5b4f7e6b3b569fabaf2
			0000000000000000000000000000000011f0c512fe7dc2dd8abdc1d22c2ecd2e7d1b84f8950ab90fc93bf54badf7bb9a9bad8c355d52a5efb110dca891e4cc3d
			0000000000000000000000000000000019774010814d1d94caf3ecda3ef4f5c5986e966eaf187c32a8a5a4a59452af0849690cf71338193f2d8435819160bcfb
		");
        let expected = hex!("
			000000000000000000000000000000000383ab7a17cc57e239e874af3f1aaabba0e64625b848676712f05f56132dbbd1cadfabeb3fe1f461daba3f1720057ddd
			00000000000000000000000000000000096967e9b3747f1b8e344535eaa0c51e70bc77412bfaa2a7ce76f11f570c9febb8f4227316866a416a50436d098e6f9a
			000000000000000000000000000000001079452b7519a7b090d668d54c266335b1cdd1080ed867dd17a2476b11c2617da829bf740e51cb7dfd60d73ed02c0c67
			00000000000000000000000000000000015fc3a972e05cbd9014882cfe6f2f16d0291c403bf28b05056ac625e4f71dfb1295c85d73145ef554614e6eb2d5bf02
		");

        let mut output = [0u8; 256];

        f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
            .expect("Builtin should not fail");
        assert_eq!(&output[..], &expected[..]);
    }

    #[test]
    fn bls12_381_g2_mul() {
        let f = Builtin {
            pricer: btreemap![0 => Pricing::Bls12ConstOperations(Bls12ConstOperations{price: 1})],
            native: EthereumBuiltin::from_str("bls12_381_g2_mul").unwrap(),
        };

        let input = hex!("
			00000000000000000000000000000000159da74f15e4c614b418997f81a1b8a3d9eb8dd80d94b5bad664bff271bb0f2d8f3c4ceb947dc6300d5003a2f7d7a829
			000000000000000000000000000000000cdd4d1d4666f385dd54052cf5c1966328403251bebb29f0d553a9a96b5ade350c8493270e9b5282d8a06f9fa8d7b1d9
			00000000000000000000000000000000189f8d3c94fdaa72cc67a7f93d35f91e22206ff9e97eed9601196c28d45b69c802ae92bcbf582754717b0355e08d37c0
			00000000000000000000000000000000054b0a282610f108fc7f6736b8c22c8778d082bf4b0d0abca5a228198eba6a868910dd5c5c440036968e977955054196
			b6a9408625b0ca8fcbfb21d34eec2d8e24e9a30d2d3b32d7a37d110b13afbfea
		");
        let expected = hex!("
			000000000000000000000000000000000b24adeb2ca184c9646cb39f45e0cf8711e10bf308ddae06519562b0af3b43be44c2fcb90622726f7446ed690551d30e
			00000000000000000000000000000000069467c3edc19416067f572c51740ba8e0e7380121ade98e38ce26d907a2bf3a4e82af2bd195b6c3b7c9b29218880531
			000000000000000000000000000000000eb8c90d0727511be53ffcb6f3b144c07983ed4b76d31ab003e45b37c7bc1066910f5e29f5adad5757af979dd0d8351d
			0000000000000000000000000000000004760f8d814189dcd893949797a3c4f56f2b60964bba3a4fc741e7ead05eb886787b2502fc64b20363eeba44e65d0ca0
		");

        let mut output = [0u8; 256];

        f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
            .expect("Builtin should not fail");
        assert_eq!(&output[..], &expected[..]);
    }

    #[test]
    fn bls12_381_g2_multiexp() {
        let f = Builtin {
            pricer: btreemap![0 => Pricing::Bls12ConstOperations(Bls12ConstOperations{price: 1})],
            native: EthereumBuiltin::from_str("bls12_381_g2_multiexp").unwrap(),
        };

        let input = hex!("
			00000000000000000000000000000000039b10ccd664da6f273ea134bb55ee48f09ba585a7e2bb95b5aec610631ac49810d5d616f67ba0147e6d1be476ea220e
			0000000000000000000000000000000000fbcdff4e48e07d1f73ec42fe7eb026f5c30407cfd2f22bbbfe5b2a09e8a7bb4884178cb6afd1c95f80e646929d3004
			0000000000000000000000000000000001ed3b0e71acb0adbf44643374edbf4405af87cfc0507db7e8978889c6c3afbe9754d1182e98ac3060d64994d31ef576
			000000000000000000000000000000001681a2bf65b83be5a2ca50430949b6e2a099977482e9405b593f34d2ed877a3f0d1bddc37d0cec4d59d7df74b2b8f2df
			b3c940fe79b6966489b527955de7599194a9ac69a6ff58b8d99e7b1084f0464e
			0000000000000000000000000000000018c0ada6351b70661f053365deae56910798bd2ace6e2bf6ba4192d1a229967f6af6ca1c9a8a11ebc0a232344ee0f6d6
			000000000000000000000000000000000cc70a587f4652039d8117b6103858adcd9728f6aebe230578389a62da0042b7623b1c0436734f463cfdd187d2090324
			0000000000000000000000000000000009f50bd7beedb23328818f9ffdafdb6da6a4dd80c5a9048ab8b154df3cad938ccede829f1156f769d9e149791e8e0cd9
			00000000000000000000000000000000079ba50d2511631b20b6d6f3841e616e9d11b68ec3368cd60129d9d4787ab56c4e9145a38927e51c9cd6271d493d9388
			4d0e25bf3f6fc9f4da25d21fdc71773f1947b7a8a775b8177f7eca990b05b71d
			0000000000000000000000000000000003632695b09dbf86163909d2bb25995b36ad1d137cf252860fd4bb6c95749e19eb0c1383e9d2f93f2791cb0cf6c8ed9d
			000000000000000000000000000000001688a855609b0bbff4452d146396558ff18777f329fd4f76a96859dabfc6a6f6977c2496280dbe3b1f8923990c1d6407
			000000000000000000000000000000000c8567fee05d05af279adc67179468a29d7520b067dbb348ee315a99504f70a206538b81a457cce855f4851ad48b7e80
			000000000000000000000000000000001238dcdfa80ea46e1500026ea5feadb421de4409f4992ffbf5ae59fa67fd82f38452642a50261b849e74b4a33eed70cc
			973f40c12c92b703d7b7848ef8b4466d40823aad3943a312b57432b91ff68be1
			000000000000000000000000000000000149704960cccf9d5ea414c73871e896b1d4cf0a946b0db72f5f2c5df98d2ec4f3adbbc14c78047961bc9620cb6cfb59
			00000000000000000000000000000000140c5d25e534fb1bfdc19ba4cecaabe619f6e0cd3d60b0f17dafd7bcd27b286d4f4477d00c5e1af22ee1a0c67fbf177c
			00000000000000000000000000000000029a1727041590b8459890de736df15c00d80ab007c3aee692ddcdf75790c9806d198e9f4502bec2f0a623491c3f877d
			0000000000000000000000000000000008a94c98baa9409151030d4fae2bd4a64c6f11ea3c99b9661fdaed226b9a7c2a7d609be34afda5d18b8911b6e015bf49
			4c51f97bcdda93904ae26991b471e9ea942e2b5b8ed26055da11c58bc7b5002a
			000000000000000000000000000000001156d478661337478ab0cbc877a99d9e4d9824a2b3f605d41404d6b557b3ffabbf42635b0bbcb854cf9ed8b8637561a8
			000000000000000000000000000000001147ed317d5642e699787a7b47e6795c9a8943a34a694007e44f8654ba96390cf19f010dcf695e22c21874022c6ce291
			000000000000000000000000000000000c6dccdf920fd5e7fae284115511952633744c6ad94120d9cae6acda8a7c23c48bd912cba6c38de5159587e1e6cad519
			000000000000000000000000000000001944227d462bc2e5dcc6f6db0f83dad411ba8895262836f975b2b91e06fd0e2138862162acc04e9e65050b34ccbd1a4e
			8964d5867927bc3e35a0b4c457482373969bff5edff8a781d65573e07fd87b89
			0000000000000000000000000000000019c31e3ab8cc9c920aa8f56371f133b6cb8d7b0b74b23c0c7201aca79e5ae69dc01f1f74d2492dcb081895b17d106b4e
			000000000000000000000000000000001789b0d371bd63077ccde3dbbebf3531368feb775bced187fb31cc6821481664600978e323ff21085b8c08e0f21daf72
			000000000000000000000000000000000009eacfe8f4a2a9bae6573424d07f42bd6af8a9d55f71476a7e3c7a4b2b898550c1e72ec13afd4eff22421a03af1d31
			000000000000000000000000000000000410bd4ea74dcfa33f2976aa1b571c67cbb596ab10f76a8aaf4548f1097e55b3373bff02683f806cb84e1e0e877819e2
			787c38b944eadbd03fd3187f450571740f6cd00e5b2e560165846eb800e5c944
			00000000000000000000000000000000147f09986691f2e57073378e8bfd58804241eed7934f6adfe6d0a6bac4da0b738495778a303e52113e1c80e698476d50
			000000000000000000000000000000000762348b84c92a8ca6de319cf1f8f11db296a71b90fe13e1e4bcd25903829c00a5d2ad4b1c8d98c37eaad7e042ab023d
			0000000000000000000000000000000011d1d94530d4a2daf0e902a5c3382cd135938557f94b04bccea5e16ea089c5e020e13524c854a316662bd68784fe31f3
			00000000000000000000000000000000070828522bec75b6a492fd9bca7b54dac6fbbf4f0bc3179d312bb65c647439e3868e4d5b21af5a64c93aeee8a9b7e46e
			aaee7ae2a237e8e53560c79e7baa9adf9c00a0ea4d6f514e7a6832eb15cef1e1
			000000000000000000000000000000000690a0869204c8dced5ba0ce13554b2703a3f18afb8fa8fa1c457d79c58fdc25471ae85bafad52e506fc1917fc3becff
			0000000000000000000000000000000010f7dbb16f8571ede1cec79e3f9ea03ae6468d7285984713f19607f5cab902b9a6b7cbcfd900be5c2e407cc093ea0e67
			00000000000000000000000000000000151caf87968433cb1f85fc1854c57049be22c26497a86bfbd66a2b3af121d894dba8004a17c6ff96a5843c2719fa32d1
			0000000000000000000000000000000011f0270f2b039409f70392879bcc2c67c836c100cf9883d3dc48d7adbcd52037d270539e863a951acd47ecaa1ca4db12
			dac6ed3ef45c1d7d3028f0f89e5458797996d3294b95bebe049b76c7d0db317c
			0000000000000000000000000000000017fae043c8fd4c520a90d4a6bd95f5b0484acc279b899e7b1d8f7f7831cc6ba37cd5965c4dc674768f5805842d433af3
			0000000000000000000000000000000008ddd7b41b8fa4d29fb931830f29b46f4015ec202d51cb969d7c832aafc0995c875cd45eff4a083e2d5ecb5ad185b64f
			0000000000000000000000000000000015d384ab7e52420b83a69827257cb52b00f0199ed2240a142812b46cf67e92b99942ac59fb9f9efd7dd822f5a36c799f
			00000000000000000000000000000000074b3a16a9cc4be9da0ac8e2e7003d9c1ec89244d2c33441b31af76716cce439f805843a9a44701203231efdca551d5b
			bb30985756c3ca075114c92f231575d6befafe4084517f1166a47376867bd108
			000000000000000000000000000000000e25365988664e8b6ade2e5a40da49c11ff1e084cc0f8dca51f0d0578555d39e3617c8cadb2abc2633b28c5895ab0a9e
			00000000000000000000000000000000169f5fd768152169c403475dee475576fd2cc3788179453b0039ff3cb1b7a5a0fff8f82d03f56e65cad579218486c3b6
			00000000000000000000000000000000087ccd7f92032febc1f75c7115111ede4acbb2e429cbccf3959524d0b79c449d431ff65485e1aecb442b53fec80ecb40
			00000000000000000000000000000000135d63f264360003b2eb28f126c6621a40088c6eb15acc4aea89d6068e9d5a47f842aa4b4300f5cda5cc5831edb81596
			fb730105809f64ea522983d6bbb62f7e2e8cbf702685e9be10e2ef71f8187672
			00000000000000000000000000000000159da74f15e4c614b418997f81a1b8a3d9eb8dd80d94b5bad664bff271bb0f2d8f3c4ceb947dc6300d5003a2f7d7a829
			000000000000000000000000000000000cdd4d1d4666f385dd54052cf5c1966328403251bebb29f0d553a9a96b5ade350c8493270e9b5282d8a06f9fa8d7b1d9
			00000000000000000000000000000000189f8d3c94fdaa72cc67a7f93d35f91e22206ff9e97eed9601196c28d45b69c802ae92bcbf582754717b0355e08d37c0
			00000000000000000000000000000000054b0a282610f108fc7f6736b8c22c8778d082bf4b0d0abca5a228198eba6a868910dd5c5c440036968e977955054196
			b6a9408625b0ca8fcbfb21d34eec2d8e24e9a30d2d3b32d7a37d110b13afbfea
			000000000000000000000000000000000f29b0d2b6e3466668e1328048e8dbc782c1111ab8cbe718c85d58ded992d97ca8ba20b9d048feb6ed0aa1b4139d02d3
			000000000000000000000000000000000d1f0dae940b99fbfc6e4a58480cac8c4e6b2fe33ce6f39c7ac1671046ce94d9e16cba2bb62c6749ef73d45bea21501a
			000000000000000000000000000000001902ccece1c0c763fd06934a76d1f2f056563ae6d8592bafd589cfebd6f057726fd908614ccd6518a21c66ecc2f78b66
			0000000000000000000000000000000017f6b113f8872c3187d20b0c765d73b850b54244a719cf461fb318796c0b8f310b5490959f9d9187f99c8ed3e25e42a9
			3b77283d0a7bb9e17a27e66851792fdd605cc0a339028b8985390fd024374c76
			000000000000000000000000000000000576b8cf1e69efdc277465c344cadf7f8cceffacbeca83821f3ff81717308b97f4ac046f1926e7c2eb42677d7afc257c
			000000000000000000000000000000000cc1524531e96f3c00e4250dd351aedb5a4c3184aff52ec8c13d470068f5967f3674fe173ee239933e67501a9decc668
			0000000000000000000000000000000001610cfcaea414c241b44cf6f3cc319dcb51d6b8de29c8a6869ff7c1ebb7b747d881e922b42e8fab96bde7cf23e8e4cd
			0000000000000000000000000000000017d4444dc8b6893b681cf10dac8169054f9d2f61d3dd5fd785ae7afa49d18ebbde9ce8dde5641adc6b38173173459836
			dd994eae929aee7428fdda2e44f8cb12b10b91c83b22abc8bbb561310b62257c
			000000000000000000000000000000000ca8f961f86ee6c46fc88fbbf721ba760186f13cd4cce743f19dc60a89fd985cb3feee34dcc4656735a326f515a729e4
			00000000000000000000000000000000174baf466b809b1155d524050f7ee58c7c5cf728c674e0ce549f5551047a4479ca15bdf69b403b03fa74eb1b26bbff6c
			0000000000000000000000000000000000e8c8b587c171b1b292779abfef57202ed29e7fe94ade9634ec5a2b3b4692a4f3c15468e3f6418b144674be70780d5b
			000000000000000000000000000000001865e99cf97d88bdf56dae32314eb32295c39a1e755cd7d1478bea8520b9ff21c39b683b92ae15568420c390c42b123b
			7010b134989c8368c7f831f9dd9f9a890e2c1435681107414f2e8637153bbf6a
			0000000000000000000000000000000017eccd446f10018219a1bd111b8786cf9febd49f9e7e754e82dd155ead59b819f0f20e42f4635d5044ec5d550d847623
			000000000000000000000000000000000403969d2b8f914ff2ea3bf902782642e2c6157bd2a343acf60ff9125b48b558d990a74c6d4d6398e7a3cc2a16037346
			000000000000000000000000000000000bd45f61f142bd78619fb520715320eb5e6ebafa8b078ce796ba62fe1a549d5fb9df57e92d8d2795988eb6ae18cf9d93
			00000000000000000000000000000000097db1314e064b8e670ec286958f17065bce644cf240ab1b1b220504560d36a0b43fc18453ff3a2bb315e219965f5bd3
			94c68bc8d91ac8c489ee87dbfc4b94c93c8bbd5fc04c27db8b02303f3a659054
			00000000000000000000000000000000018244ab39a716e252cbfb986c7958b371e29ea9190010d1f5e1cfdb6ce4822d4055c37cd411fc9a0c46d728f2c13ecf
			0000000000000000000000000000000001985d3c667c8d68c9adb92bdc7a8af959c17146544997d97116120a0f55366bd7ad7ffa28d93ee51222ff9222779675
			000000000000000000000000000000000c70fd4e3c8f2a451f83fb6c046431b38251b7bae44cf8d36df69a03e2d3ce6137498523fcf0bcf29b5d69e8f265e24d
			00000000000000000000000000000000047b9163a218f7654a72e0d7c651a2cf7fd95e9784a59e0bf119d081de6c0465d374a55fbc1eff9828c9fd29abf4c4bd
			b3682accc3939283b870357cf83683350baf73aa0d3d68bda82a0f6ae7e51746
		");
        let expected = hex!("
			00000000000000000000000000000000083ad744b34f6393bc983222b004657494232c5d9fbc978d76e2377a28a34c4528da5d91cbc0977dc953397a6d21eca2
			0000000000000000000000000000000015aec6526e151cf5b8403353517dfb9a162087a698b71f32b266d3c5c936a83975d5567c25b3a5994042ec1379c8e526
			000000000000000000000000000000000e3647185d1a20efad19f975729908840dc33909a583600f7915025f906aef9c022fd34e618170b11178aaa824ae36b3
			00000000000000000000000000000000159576d1d53f6cd12c39d651697e11798321f17cd287118d7ebeabf68281bc03109ee103ee8ef2ef93c71dd1dcbaf1e0
		");

        let mut output = [0u8; 256];

        f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
            .expect("Builtin should not fail");
        assert_eq!(&output[..], &expected[..]);
    }

    #[test]
    fn bls12_381_pairing() {
        let f = Builtin {
            pricer: btreemap![0 => 	Pricing::Bls12Pairing(Bls12PairingPricer{price: Bls12PairingPrice{base: 1, pair: 1}})],
            native: EthereumBuiltin::from_str("bls12_381_pairing").unwrap(),
        };

        let input = hex!("
			000000000000000000000000000000001830f52d9bff64a623c6f5259e2cd2c2a08ea17a8797aaf83174ea1e8c3bd3955c2af1d39bfa474815bfe60714b7cd80
			000000000000000000000000000000000874389c02d4cf1c61bc54c4c24def11dfbe7880bc998a95e70063009451ee8226fec4b278aade3a7cea55659459f1d5
			00000000000000000000000000000000197737f831d4dc7e708475f4ca7ca15284db2f3751fcaac0c17f517f1ddab35e1a37907d7b99b39d6c8d9001cd50e79e
			000000000000000000000000000000000af1a3f6396f0c983e7c2d42d489a3ae5a3ff0a553d93154f73ac770cd0af7467aa0cef79f10bbd34621b3ec9583a834
			000000000000000000000000000000001918cb6e448ed69fb906145de3f11455ee0359d030e90d673ce050a360d796de33ccd6a941c49a1414aca1c26f9e699e
			0000000000000000000000000000000019a915154a13249d784093facc44520e7f3a18410ab2a3093e0b12657788e9419eec25729944f7945e732104939e7a9e
			000000000000000000000000000000001830f52d9bff64a623c6f5259e2cd2c2a08ea17a8797aaf83174ea1e8c3bd3955c2af1d39bfa474815bfe60714b7cd80
			00000000000000000000000000000000118cd94e36ab177de95f52f180fdbdc584b8d30436eb882980306fa0625f07a1f7ad3b4c38a921c53d14aa9a6ba5b8d6
			00000000000000000000000000000000197737f831d4dc7e708475f4ca7ca15284db2f3751fcaac0c17f517f1ddab35e1a37907d7b99b39d6c8d9001cd50e79e
			000000000000000000000000000000000af1a3f6396f0c983e7c2d42d489a3ae5a3ff0a553d93154f73ac770cd0af7467aa0cef79f10bbd34621b3ec9583a834
			000000000000000000000000000000001918cb6e448ed69fb906145de3f11455ee0359d030e90d673ce050a360d796de33ccd6a941c49a1414aca1c26f9e699e
			0000000000000000000000000000000019a915154a13249d784093facc44520e7f3a18410ab2a3093e0b12657788e9419eec25729944f7945e732104939e7a9e
		");
        let expected = hex!(
            "
			0000000000000000000000000000000000000000000000000000000000000001
		"
        );

        let mut output = [0u8; 32];

        f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
            .expect("Builtin should not fail");
        assert_eq!(&output[..], &expected[..]);
    }

    #[test]
    fn bls12_381_fp_to_g1() {
        let f = Builtin {
            pricer: btreemap![0 => 	Pricing::Bls12Pairing(Bls12PairingPricer{price: Bls12PairingPrice{base: 1, pair: 1}})],
            native: EthereumBuiltin::from_str("bls12_381_fp_to_g1").unwrap(),
        };

        let input = hex!("
			0000000000000000000000000000000017f66b472b36717ee0902d685c808bb5f190bbcb2c51d067f1cbec64669f10199a5868d7181dcec0498fcc71f5acaf79
		");
        let expected = hex!("
			00000000000000000000000000000000188dc9e5ddf48977f33aeb6e505518269bf67fb624fa86b79741d842e75a6fa1be0911c2caa9e55571b6e55a3c0c0b9e
			00000000000000000000000000000000193e8b7c7e78daf104a59d7b39401a65355fa874bd34e91688580941e99a863367efc68fe871e38e07423090e93919c9
		");

        let mut output = [0u8; 128];

        f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
            .expect("Builtin should not fail");
        assert_eq!(&output[..], &expected[..]);
    }

    #[test]
    fn bls12_381_fp2_to_g2() {
        let f = Builtin {
            pricer: btreemap![0 => 	Pricing::Bls12Pairing(Bls12PairingPricer{price: Bls12PairingPrice{base: 1, pair: 1}})],
            native: EthereumBuiltin::from_str("bls12_381_fp2_to_g2").unwrap(),
        };

        let input = hex!("
			000000000000000000000000000000000f470603a402bc134db1b389fd187460f9eb2dd001a2e99f730af386508c62f0e911d831a2562da84bce11d39f2ff13f
			000000000000000000000000000000000d8c45f4ab20642d0cba9764126e0818b7d731a6ba29ed234d9d6309a5e8ddfbd85193f1fa8b7cfeed3d31b23b904ee9
		");
        let expected = hex!("
			0000000000000000000000000000000012e74d5a0c005a86ca148e9eff8e34a00bfa8b6e6aadf633d65cd09bb29917e0ceb0d5c9d9650c162d7fe4aa27452685
			0000000000000000000000000000000005f09101a2088712619f9c096403b66855a12f9016c55aef6047372fba933f02d9d59db1a86df7be57978021e2457821
			00000000000000000000000000000000136975b37fe400d1d217a2b496c1552b39be4e9e71dd7ad482f5f0836d271d02959fdb698dda3d0530587fb86e0db1dd
			0000000000000000000000000000000000bad0aabd9309e92e2dd752f4dd73be07c0de2c5ddd57916b9ffa065d7440d03d44e7c042075cda694414a9fb639bb7
		");

        let mut output = [0u8; 256];

        f.execute(&input[..], &mut BytesRef::Fixed(&mut output[..]))
            .expect("Builtin should not fail");
        assert_eq!(&output[..], &expected[..]);
    }

    #[test]
    fn bls12_381_g1_multiexp_init_from_spec() {
        use ethjson::spec::builtin::{Bls12G1Multiexp, Pricing};

        let b = Builtin::try_from(JsonBuiltin {
            name: "bls12_381_g1_multiexp".to_owned(),
            pricing: btreemap![
                10000000 => PricingAt {
                    info: None,
                    price: Pricing::Bls12G1Multiexp(Bls12G1Multiexp{
                            base: 12000,
                    }),
                }
            ],
        })
        .unwrap();

        match b.native {
            EthereumBuiltin::Bls12G1MultiExp(..) => {}
            _ => {
                panic!("invalid precompile type");
            }
        }
    }

    #[test]
    fn bls12_381_g2_multiexp_init_from_spec() {
        use ethjson::spec::builtin::{Bls12G2Multiexp, Pricing};

        let b = Builtin::try_from(JsonBuiltin {
            name: "bls12_381_g2_multiexp".to_owned(),
            pricing: btreemap![
                10000000 => PricingAt {
                    info: None,
                    price: Pricing::Bls12G2Multiexp(Bls12G2Multiexp{
                            base: 55000,
                    }),
                }
            ],
        })
        .unwrap();

        match b.native {
            EthereumBuiltin::Bls12G2MultiExp(..) => {}
            _ => {
                panic!("invalid precompile type");
            }
        }
    }
}
