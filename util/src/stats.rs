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

//! Statistical functions.

use bigint::uint::*;

/// Discretised histogram.
#[derive(Debug, PartialEq)]
pub struct Histogram {
	/// Bounds of each bucket.
	pub bucket_bounds: Vec<U256>,
	/// Count within each bucket.
	pub counts: Vec<u64>
}

impl Histogram {
	/// Histogram if a sorted corpus is at least fills the buckets.
	pub fn new(corpus: &[U256], bucket_number: usize) -> Option<Histogram> {
		if corpus.len() < bucket_number { return None; }
		let corpus_end = corpus.last().expect("there are at least bucket_number elements; qed").clone();
		// If there are extremely few transactions, go from zero.
		let corpus_start = corpus.first().expect("there are at least bucket_number elements; qed").clone();
		let bucket_size = (corpus_end - corpus_start + 1.into()) / bucket_number.into();
		let mut bucket_end = corpus_start + bucket_size;

		let mut bucket_bounds = vec![corpus_start; bucket_number + 1];
		let mut counts = vec![0; bucket_number];
		let mut corpus_i = 0;
		// Go through the corpus adding to buckets.
		for bucket in 0..bucket_number {
			while corpus[corpus_i] < bucket_end {
				counts[bucket] += 1;
				corpus_i += 1;
			}
			bucket_bounds[bucket + 1] = bucket_end;
			bucket_end = bucket_end + bucket_size;
		}
		Some(Histogram { bucket_bounds: bucket_bounds, counts: counts })
	}
}


#[cfg(test)]
mod tests {
	use bigint::uint::U256;
	use super::Histogram;

	#[test]
	fn check_histogram() {
		let hist = Histogram::new(&vec_into![643,689,1408,2000,2296,2512,4250,4320,4842,4958,5804,6065,6098,6354,7002,7145,7845,8589,8593,8895], 5).unwrap();
		let correct_bounds: Vec<U256> = vec_into![643,2293,3943,5593,7243,8893];
		assert_eq!(Histogram { bucket_bounds: correct_bounds, counts: vec![4,2,4,6,3] }, hist);

		assert!(Histogram::new(&vec_into![1, 2], 5).is_none());
	}
}
