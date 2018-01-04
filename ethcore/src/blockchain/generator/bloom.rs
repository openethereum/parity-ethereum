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

use ethereum_types::H2048;

pub trait WithBloom {
	fn with_bloom(self, bloom: H2048) -> Self where Self: Sized;
}

pub struct Bloom<'a, I> where I: 'a {
	pub iter: &'a mut I,
	pub bloom: H2048,
}

impl<'a, I> Iterator for Bloom<'a, I> where I: Iterator, <I as Iterator>::Item: WithBloom {
	type Item = <I as Iterator>::Item;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|item| item.with_bloom(self.bloom.clone()))
	}
}
