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

//! Encryptor for private transactions tests.

use encryptor::{Encryptor, NoopEncryptor};
use rand::{Rng, OsRng};
use std::sync::Arc;
use ethereum_types::H128;
use ethcore::account_provider::AccountProvider;

const INIT_VEC_LEN: usize = 16;

fn initialization_vector() -> H128 {
	let mut result = [0u8; INIT_VEC_LEN];
	let mut rng = OsRng::new().unwrap();
	rng.fill_bytes(&mut result);
	H128::from_slice(&result)
}

#[test]
fn dummy_encryptor_works() {
	let encryptor = NoopEncryptor::default();
	let ap = Arc::new(AccountProvider::transient_provider());

	let plain_data = vec![42];
	let iv = initialization_vector();
	let cypher = encryptor.encrypt(&Default::default(), ap.clone(), &iv, &plain_data).unwrap();
	let _decrypted_data = encryptor.decrypt(&Default::default(), ap.clone(), &cypher).unwrap();
}
