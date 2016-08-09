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

use rand::{Rng, OsRng};
use itertools::Itertools;

pub trait Random {
	fn random() -> Self where Self: Sized;
}

impl Random for [u8; 16] {
	fn random() -> Self {
		let mut result = [0u8; 16];
		let mut rng = OsRng::new().unwrap();
		rng.fill_bytes(&mut result);
		result
	}
}

impl Random for [u8; 32] {
	fn random() -> Self {
		let mut result = [0u8; 32];
		let mut rng = OsRng::new().unwrap();
		rng.fill_bytes(&mut result);
		result
	}
}

/// Generate a string which is a random phrase of a number of lowercase words.
///
/// `words` is the number of words, chosen from a dictionary of 7,530. An value of
/// 12 gives 155 bits of entropy (almost saturating address space); 20 gives 258 bits
/// which is enough to saturate 32-byte key space
pub fn random_phrase(words: usize) -> String {
	lazy_static! {
		static ref WORDS: Vec<String> = String::from_utf8_lossy(include_bytes!("../res/wordlist.txt"))
			.split("\n")
			.map(|s| s.to_owned())
			.collect();
	}
	let mut rng = OsRng::new().unwrap();
	(0..words).map(|_| rng.choose(&WORDS).unwrap()).join(" ")
}

#[test]
fn should_produce_right_number_of_words() {
	let p = random_phrase(10);
	assert_eq!(p.split(" ").count(), 10);
}