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

use std::io::{BufReader, Read, BufRead};
use bloomchain::Bloom;
use super::FromHex;

pub fn for_each_bloom<F>(bytes: &[u8], mut f: F) where F: FnMut(usize, Bloom) {
	let mut reader = BufReader::new(bytes);
	let mut line = String::new();
	while reader.read_line(&mut line).unwrap() > 0 {
		{
			let mut number_bytes = vec![];
			let mut bloom_bytes = [0; 512];

			let mut line_reader = BufReader::new(line.as_ref() as &[u8]);
			line_reader.read_until(b' ', &mut number_bytes).unwrap();
			line_reader.consume(2);
			line_reader.read_exact(&mut bloom_bytes).unwrap();

			let number = String::from_utf8(number_bytes).map(|s| s[..s.len() -1].to_owned()).unwrap().parse::<usize>().unwrap();
			let bloom = Bloom::from_hex(&String::from_utf8(bloom_bytes.to_vec()).unwrap());
			f(number, bloom);
		}
		line.clear();
	}
}
