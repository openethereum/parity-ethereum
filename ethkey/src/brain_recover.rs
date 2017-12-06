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

use std::collections::HashSet;

use edit_distance::edit_distance;
use parity_wordlist;

use super::{Address, Brain, Generator};


/// Tries to find a phrase for address, given the number
/// of expected words and a partial phrase.
///
/// Returns `None` if phrase couldn't be found.
pub fn brain_recover(
	address: &Address,
	known_phrase: &str,
	expected_words: usize,
) -> Option<String> {
	let it = PhrasesIterator::from_known_phrase(known_phrase, expected_words);
	for phrase in it {
		let keypair = Brain::new(phrase.clone()).generate().expect("Brain wallets are infallible; qed");
		trace!("Testing: {}, got: {:?}", phrase, keypair.address());
		if &keypair.address() == address {
			return Some(phrase);
		}
	}

	None
}

fn generate_substitutions(word: &str) -> Vec<&'static str> {
	let mut words = parity_wordlist::WORDS.iter().cloned()
		.map(|w| (edit_distance(w, word), w))
		.collect::<Vec<_>>();
	words.sort_by(|a, b| a.0.cmp(&b.0));

	words.into_iter()
		.map(|pair| pair.1)
		.collect()
}

/// Iterator over possible
pub struct PhrasesIterator {
	words: Vec<Vec<&'static str>>,
	combinations: u64,
	indexes: Vec<usize>,
	has_next: bool,
}

impl PhrasesIterator {
	pub fn from_known_phrase(known_phrase: &str, expected_words: usize) -> Self {
		let known_words = parity_wordlist::WORDS.iter().cloned().collect::<HashSet<_>>();
		let mut words = known_phrase.split(' ')
			.map(|word| match known_words.get(word) {
				None => {
					info!("Invalid word '{}', looking for potential substitutions.", word);
					let substitutions = generate_substitutions(word);
					info!("Closest words: {:?}", &substitutions[..10]);
					substitutions
				},
				Some(word) => vec![*word],
			})
		.collect::<Vec<_>>();

		// add missing words
		if words.len() < expected_words {
			let to_add = expected_words - words.len();
			info!("Number of words is insuficcient adding {} more.", to_add);
			for _ in 0..to_add {
				words.push(parity_wordlist::WORDS.iter().cloned().collect());
			}
		}

		// start searching
		PhrasesIterator::new(words)
	}

	pub fn new(words: Vec<Vec<&'static str>>) -> Self {
		let combinations = words.iter().fold(1u64, |acc, x| acc * x.len() as u64);
		let indexes = words.iter().map(|_| 0).collect();
		info!("Starting to test {} possible combinations.", combinations);

		PhrasesIterator {
			words,
			combinations,
			indexes,
			has_next: combinations > 0,
		}
	}

	pub fn combinations(&self) -> u64 {
		self.combinations
	}

	fn current(&self) -> String {
		let mut s = self.words[0][self.indexes[0]].to_owned();
		for i in 1..self.indexes.len() {
			s.push(' ');
			s.push_str(self.words[i][self.indexes[i]]);
		}
		s
	}

	fn next_index(&mut self) -> bool {
		let mut pos = self.indexes.len();
		while pos > 0 {
			pos -= 1;
			self.indexes[pos] += 1;
			if self.indexes[pos] >= self.words[pos].len() {
				self.indexes[pos] = 0;
			} else {
				return true;
			}
		}

		false
	}
}

impl Iterator for PhrasesIterator {
	type Item = String;

	fn next(&mut self) -> Option<String> {
		if !self.has_next {
			return None;
		}

		let phrase = self.current();
		self.has_next = self.next_index();
		Some(phrase)
	}
}

#[cfg(test)]
mod tests {
	use super::PhrasesIterator;


	#[test]
	fn should_generate_possible_combinations() {
		let mut it = PhrasesIterator::new(vec![
			vec!["1", "2", "3"],
			vec!["test"],
			vec!["a", "b", "c"],
		]);

		assert_eq!(it.combinations(), 9);
		assert_eq!(it.next(), Some("1 test a".to_owned()));
		assert_eq!(it.next(), Some("1 test b".to_owned()));
		assert_eq!(it.next(), Some("1 test c".to_owned()));
		assert_eq!(it.next(), Some("2 test a".to_owned()));
		assert_eq!(it.next(), Some("2 test b".to_owned()));
		assert_eq!(it.next(), Some("2 test c".to_owned()));
		assert_eq!(it.next(), Some("3 test a".to_owned()));
		assert_eq!(it.next(), Some("3 test b".to_owned()));
		assert_eq!(it.next(), Some("3 test c".to_owned()));
		assert_eq!(it.next(), None);
	}

}
