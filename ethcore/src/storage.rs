trait Storage {
	fn request_bytes(&mut self, key: &[u8]) -> Option<Vec<u8>>;
}
