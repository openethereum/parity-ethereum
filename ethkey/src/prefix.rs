use super::{Random, Generator, KeyPair, Error};

/// Tries to find keypair with address starting with given prefix.
pub struct Prefix {
	prefix: Vec<u8>,
	iterations: usize,
}

impl Prefix {
	pub fn new(prefix: Vec<u8>, iterations: usize) -> Self {
		Prefix {
			prefix: prefix,
			iterations: iterations,
		}
	}
}

impl Generator for Prefix {
	fn generate(self) -> Result<KeyPair, Error> {
		for _ in 0..self.iterations {
			let keypair = try!(Random.generate());
			if keypair.address().starts_with(&self.prefix) {
				return Ok(keypair)
			}
		}

		Err(Error::Custom("Could not find keypair".into()))
	}
}

#[cfg(test)]
mod tests {
	use {Generator, Prefix};

	#[test]
	fn prefix_generator() {
		let prefix = vec![0xffu8];
		let keypair = Prefix::new(prefix.clone(), usize::max_value()).generate().unwrap();
		assert!(keypair.address().starts_with(&prefix));
	}
}
