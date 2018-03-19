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
