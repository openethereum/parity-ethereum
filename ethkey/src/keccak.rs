use tiny_keccak::Keccak;

pub trait Keccak256<T> {
	fn keccak256(&self) -> T where T: Sized;
}

impl Keccak256<[u8; 32]> for [u8] {
	fn keccak256(&self) -> [u8; 32] {
		let mut keccak = Keccak::new_keccak256();
		let mut result = [0u8; 32];
		keccak.update(self);
		keccak.finalize(&mut result);
		result
	}
}
