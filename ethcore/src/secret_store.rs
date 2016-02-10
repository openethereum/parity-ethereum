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

//! SecretStore
//! module for managing key files, decrypting and encrypting arbitrary data

use common::*;

enum CryptoCipherType {
	// aes-128-ctr with 128-bit initialisation vector(iv)
	Aes128Ctr(U128)
}

enum KeyFileKdf {
	Pbkdf2(KdfPbkdf2Params),
	Scrypt(KdfScryptParams)
}

struct KeyFileCrypto {
	cipher: CryptoCipherType,
	Kdf: KeyFileKdf,
}

enum KeyFileVersion {
	V1, V2, V3
}

enum Pbkdf2CryptoFunction {
	HMacSha256
}

#[allow(non_snake_case)]
// Kdf of type `Pbkdf2`
// https://en.wikipedia.org/wiki/PBKDF2
struct KdfPbkdf2Params {
	// desired length of the derived key, in octets
	dkLen: u32,
	// cryptographic salt
	salt: H256,
	// number of iterations for derived key
	c: u32,
	// pseudo-random 2-parameters function
	prf: Pbkdf2CryptoFunction
}

#[allow(non_snake_case)]
// Kdf of type `Scrypt`
// https://en.wikipedia.org/wiki/Scrypt
struct KdfScryptParams {
	// desired length of the derived key, in octets
	dkLen: u32,
	// parallelization
	p: u32,
	// cpu cost
	n: u32,
	// TODO: comment
	r: u32,
}

type Uuid = String;

enum Kdf {
	Pbkdf2(KdfPbkdf2Params),
	Scrypt(KdfScryptParams)
}

struct KeyFileContent {
	version: KeyFileVersion,
	crypto: KeyFileCrypto,
	id: Uuid
}
