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

//! Statistical functions and helpers.

use std::iter::FromIterator;
use std::ops::{Add, Sub, Deref, Div};

#[macro_use]
extern crate log;

/// Sorted corpus of data.
#[derive(Debug, Clone, PartialEq)]
pub struct Corpus<T>(Vec<T>);

impl<T: Ord> From<Vec<T>> for Corpus<T> {
	fn from(mut data: Vec<T>) -> Self {
		data.sort();
		Corpus(data)
	}
}

impl<T: Ord> FromIterator<T> for Corpus<T> {
	fn from_iter<I: IntoIterator<Item=T>>(iterable: I) -> Self {
		iterable.into_iter().collect::<Vec<_>>().into()
	}
}

impl<T> Deref for Corpus<T> {
	type Target = [T];

	fn deref(&self) -> &[T] { &self.0[..] }
}

impl<T: Ord> Corpus<T> {
	/// Get given percentile (approximated).
	pub fn percentile(&self, val: usize) -> Option<&T> {
		let len = self.0.len();
		let x = val * len / 100;
		let x = ::std::cmp::min(x, len);
		if x == 0 {
			return None;
		}

		self.0.get(x - 1)
	}

	/// Get the median element, if it exists.
	pub fn median(&self) -> Option<&T> {
		self.0.get(self.0.len() / 2)
	}

	/// Whether the corpus is empty.
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Number of elements in the corpus.
	pub fn len(&self) -> usize {
		self.0.len()
	}
}

impl<T: Ord + Copy + ::std::fmt::Display> Corpus<T>
	where T: Add<Output=T> + Sub<Output=T> + Div<Output=T> + From<usize>
{
	/// Create a histogram of this corpus if it at least spans the buckets. Bounds are left closed.
	/// Excludes outliers.
	pub fn histogram(&self, bucket_number: usize) -> Option<Histogram<T>> {
		// TODO: get outliers properly.
		let upto = self.len() - self.len() / 40;
		Histogram::create(&self.0[..upto], bucket_number)
	}
}

/// Discretised histogram.
#[derive(Debug, PartialEq)]
pub struct Histogram<T> {
	/// Bounds of each bucket.
	pub bucket_bounds: Vec<T>,
	/// Count within each bucket.
	pub counts: Vec<usize>,
}

impl<T: Ord + Copy + ::std::fmt::Display> Histogram<T>
	where T: Add<Output=T> + Sub<Output=T> + Div<Output=T> + From<usize>
{
	// Histogram of a sorted corpus if it at least spans the buckets. Bounds are left closed.
	fn create(corpus: &[T], bucket_number: usize) -> Option<Histogram<T>> {
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
	use super::*;

	#[test]
	fn check_corpus() {
		let corpus = Corpus::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
		assert_eq!(corpus.percentile(0), None);
		assert_eq!(corpus.percentile(1), None);
		assert_eq!(corpus.percentile(101), Some(&10));
		assert_eq!(corpus.percentile(100), Some(&10));
		assert_eq!(corpus.percentile(50), Some(&5));
		assert_eq!(corpus.percentile(60), Some(&6));
		assert_eq!(corpus.median(), Some(&6));
	}

	#[test]
	fn check_histogram() {
		let hist = Histogram::create(&[643,689,1408,2000,2296,2512,4250,4320,4842,4958,5804,6065,6098,6354,7002,7145,7845,8589,8593,8895], 5).unwrap();
		let correct_bounds: Vec<usize> = vec![643, 2294, 3945, 5596, 7247, 8898];
		assert_eq!(Histogram { bucket_bounds: correct_bounds, counts: vec![4,2,4,6,4] }, hist);
	}

	#[test]
	fn smaller_data_range_than_bucket_range() {
		assert_eq!(
			Histogram::create(&[1, 2, 2], 3),
			Some(Histogram { bucket_bounds: vec![1, 2, 3, 4], counts: vec![1, 2, 0] })
		);
	}

	#[test]
	fn data_range_is_not_multiple_of_bucket_range() {
		assert_eq!(
			Histogram::create(&[1, 2, 5], 2),
			Some(Histogram { bucket_bounds: vec![1, 4, 7], counts: vec![2, 1] })
		);
	}

	#[test]
	fn data_range_is_multiple_of_bucket_range() {
		assert_eq!(
			Histogram::create(&[1, 2, 6], 2),
			Some(Histogram { bucket_bounds: vec![1, 4, 7], counts: vec![2, 1] })
		);
	}

	#[test]
	fn none_when_too_few_data() {
		assert!(Histogram::<usize>::create(&[], 1).is_none());
	}
}
