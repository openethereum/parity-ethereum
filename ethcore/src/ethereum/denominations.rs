use util::*;

#[inline]
/// 1 Ether in Wei
pub fn ether() -> U256 { U256::exp10(18) }

#[inline]
/// 1 Finney in Wei
pub fn finney() -> U256 { U256::exp10(15) }

#[inline]
/// 1 Szabo in Wei
pub fn szabo() -> U256 { U256::exp10(12) }

#[inline]
/// 1 Shannon in Wei
pub fn shannon() -> U256 { U256::exp10(9) }

#[inline]
/// 1 Wei in Wei
pub fn wei() -> U256 { U256::exp10(0) }

