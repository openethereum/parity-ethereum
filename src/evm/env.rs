use util::hash::*;

pub struct Env;

impl Env {
	pub fn new() -> Env {
		Env
	}

	pub fn sload(&self, _index: &H256) -> H256 {
		println!("sload!: {:?}", _index);
		//unimplemented!();
		H256::new()
	}

	pub fn sstore(&self, _index: &H256, _value: &H256) {
		println!("sstore!: {:?} , {:?}", _index, _value);
		//unimplemented!();
		
	}
}


