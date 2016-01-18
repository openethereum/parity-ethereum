extern {
	pub fn sha3_256(out: *mut u8, outlen: usize, input: *const u8, inputlen: usize) -> i32;
	pub fn sha3_512(out: *mut u8, outlen: usize, input: *const u8, inputlen: usize) -> i32;
}
