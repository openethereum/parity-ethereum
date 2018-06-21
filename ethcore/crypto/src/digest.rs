// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

use rcrypto::ripemd160;
use ring::digest::{self, Context, SHA256, SHA512};
use std::marker::PhantomData;
use std::ops::Deref;

/// The message digest.
pub struct Digest<T>(InnerDigest, PhantomData<T>);

enum InnerDigest {
	Ring(digest::Digest),
	Ripemd160([u8; 20]),
}

impl<T> Deref for Digest<T> {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		match self.0 {
			InnerDigest::Ring(ref d) => d.as_ref(),
			InnerDigest::Ripemd160(ref d) => &d[..]
		}
	}
}

/// Single-step sha256 digest computation.
pub fn sha256(data: &[u8]) -> Digest<Sha256> {
	Digest(InnerDigest::Ring(digest::digest(&SHA256, data)), PhantomData)
}

/// Single-step sha512 digest computation.
pub fn sha512(data: &[u8]) -> Digest<Sha512> {
	Digest(InnerDigest::Ring(digest::digest(&SHA512, data)), PhantomData)
}

/// Single-step ripemd160 digest computation.
pub fn ripemd160(data: &[u8]) -> Digest<Ripemd160> {
	let mut hasher = Hasher::ripemd160();
	hasher.update(data);
	hasher.finish()
}

pub enum Sha256 {}
pub enum Sha512 {}
pub enum Ripemd160 {}

/// Stateful digest computation.
pub struct Hasher<T>(Inner, PhantomData<T>);

enum Inner {
	Ring(Context),
	Ripemd160(ripemd160::Ripemd160)
}

impl Hasher<Sha256> {
	pub fn sha256() -> Hasher<Sha256> {
		Hasher(Inner::Ring(Context::new(&SHA256)), PhantomData)
	}
}

impl Hasher<Sha512> {
	pub fn sha512() -> Hasher<Sha512> {
		Hasher(Inner::Ring(Context::new(&SHA512)), PhantomData)
	}
}

impl Hasher<Ripemd160> {
	pub fn ripemd160() -> Hasher<Ripemd160> {
		Hasher(Inner::Ripemd160(ripemd160::Ripemd160::new()), PhantomData)
	}
}

impl<T> Hasher<T> {
	pub fn update(&mut self, data: &[u8]) {
		match self.0 {
			Inner::Ring(ref mut ctx) => ctx.update(data),
			Inner::Ripemd160(ref mut ctx) => {
				use rcrypto::digest::Digest;
				ctx.input(data)
			}
		}
	}

	pub fn finish(self) -> Digest<T> {
		match self.0 {
			Inner::Ring(ctx) => Digest(InnerDigest::Ring(ctx.finish()), PhantomData),
			Inner::Ripemd160(mut ctx) => {
				use rcrypto::digest::Digest;
				let mut d = [0; 20];
				ctx.result(&mut d);
				Digest(InnerDigest::Ripemd160(d), PhantomData)
			}
		}
	}
}
