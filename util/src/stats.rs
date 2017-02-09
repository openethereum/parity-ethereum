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

//! Statistical functions.

use bigint::prelude::*;

/// Discretised histogram.
#[derive(Debug, PartialEq)]
pub struct Histogram {
	/// Bounds of each bucket.
	pub bucket_bounds: Vec<U256>,
	/// Count within each bucket.
	pub counts: Vec<u64>
}

impl Histogram {
	/// Histogram of a sorted corpus if it at least spans the buckets. Bounds are left closed.
	pub fn new(corpus: &[U256], bucket_number: usize) -> Option<Histogram> {
		if corpus.len() < 1 { return None; }
		let corpus_end = corpus.last().expect("there is at least 1 element; qed").clone();
		let corpus_start = corpus.first().expect("there is at least 1 element; qed").clone();
		trace!(target: "stats", "Computing histogram from {} to {} with {} buckets.", corpus_start, corpus_end, bucket_number);
		// Bucket needs to be at least 1 wide.
		let bucket_size = {
			// Round up to get the entire corpus included.
			let raw_bucket_size = (corpus_end - corpus_start + bucket_number.into()) / bucket_number.into();
			if raw_bucket_size == 0.into() { 1.into() } else { raw_bucket_size }
		};
		let mut bucket_end = corpus_start + bucket_size;

		let mut bucket_bounds = vec![corpus_start; bucket_number + 1];
		let mut counts = vec![0; bucket_number];
		let mut corpus_i = 0;
		// Go through the corpus adding to buckets.
		for bucket in 0..bucket_number {
			while corpus.get(corpus_i).map_or(false, |v| v < &bucket_end) {
				// Initialized to size bucket_number above; iterates up to bucket_number; qed
				counts[bucket] += 1;
				corpus_i += 1;
			}
			// Initialized to size bucket_number + 1 above; iterates up to bucket_number; subscript is in range; qed
			bucket_bounds[bucket + 1] = bucket_end;
			bucket_end = bucket_end + bucket_size;
		}
		Some(Histogram { bucket_bounds: bucket_bounds, counts: counts })
	}
}


#[cfg(test)]
mod tests {
	use bigint::prelude::U256;
	use super::Histogram;

	#[test]
	fn check_histogram() {
		let hist = Histogram::new(slice_into![643,689,1408,2000,2296,2512,4250,4320,4842,4958,5804,6065,6098,6354,7002,7145,7845,8589,8593,8895], 5).unwrap();
		let correct_bounds: Vec<U256> = vec_into![643, 2294, 3945, 5596, 7247, 8898];
		assert_eq!(Histogram { bucket_bounds: correct_bounds, counts: vec![4,2,4,6,4] }, hist);
	}

	#[test]
	fn smaller_data_range_than_bucket_range() {
		assert_eq!(
			Histogram::new(slice_into![1, 2, 2], 3),
			Some(Histogram { bucket_bounds: vec_into![1, 2, 3, 4], counts: vec![1, 2, 0] })
		);
	}

	#[test]
	fn data_range_is_not_multiple_of_bucket_range() {
		assert_eq!(
			Histogram::new(slice_into![1, 2, 5], 2),
			Some(Histogram { bucket_bounds: vec_into![1, 4, 7], counts: vec![2, 1] })
		);
	}

	#[test]
	fn data_range_is_multiple_of_bucket_range() {
		assert_eq!(
			Histogram::new(slice_into![1, 2, 6], 2),
			Some(Histogram { bucket_bounds: vec_into![1, 4, 7], counts: vec![2, 1] })
		);
	}

	#[test]
	fn none_when_too_few_data() {
		assert!(Histogram::new(slice_into![], 1).is_none());
	}
}
