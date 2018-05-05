// Copyright 2018 Parity Technologies (UK) Ltd.
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

use digest;
use ring::digest::{SHA256, SHA512};
use ring::hmac::{self, SigningContext};
use std::marker::PhantomData;
use std::ops::Deref;

/// HMAC signature.
pub struct Signature<T>(hmac::Signature, PhantomData<T>);

impl<T> Deref for Signature<T> {
	type Target = [u8];
	fn deref(&self) -> &Self::Target {
		self.0.as_ref()
	}
}

/// HMAC signing key.
pub struct SigKey<T>(hmac::SigningKey, PhantomData<T>);

impl SigKey<digest::Sha256> {
	pub fn sha256(key: &[u8]) -> SigKey<digest::Sha256> {
		SigKey(hmac::SigningKey::new(&SHA256, key), PhantomData)
	}
}

impl SigKey<digest::Sha512> {
	pub fn sha512(key: &[u8]) -> SigKey<digest::Sha512> {
		SigKey(hmac::SigningKey::new(&SHA512, key), PhantomData)
	}
}

/// Compute HMAC signature of `data`.
pub fn sign<T>(k: &SigKey<T>, data: &[u8]) -> Signature<T> {
	Signature(hmac::sign(&k.0, data), PhantomData)
}

/// Stateful HMAC computation.
pub struct Signer<T>(SigningContext, PhantomData<T>);

impl<T> Signer<T> {
	pub fn with(key: &SigKey<T>) -> Signer<T> {
		Signer(hmac::SigningContext::with_key(&key.0), PhantomData)
	}

	pub fn update(&mut self, data: &[u8]) {
		self.0.update(data)
	}

	pub fn sign(self) -> Signature<T> {
		Signature(self.0.sign(), PhantomData)
	}
}

/// HMAC signature verification key.
pub struct VerifyKey<T>(hmac::VerificationKey, PhantomData<T>);

impl VerifyKey<digest::Sha256> {
	pub fn sha256(key: &[u8]) -> VerifyKey<digest::Sha256> {
		VerifyKey(hmac::VerificationKey::new(&SHA256, key), PhantomData)
	}
}

impl VerifyKey<digest::Sha512> {
	pub fn sha512(key: &[u8]) -> VerifyKey<digest::Sha512> {
		VerifyKey(hmac::VerificationKey::new(&SHA512, key), PhantomData)
	}
}

/// Verify HMAC signature of `data`.
pub fn verify<T>(k: &VerifyKey<T>, data: &[u8], sig: &[u8]) -> bool {
	hmac::verify(&k.0, data, sig).is_ok()
}

