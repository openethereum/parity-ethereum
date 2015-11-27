use hash::*;
use bytes::Bytes;

pub trait HashDB {
	fn lookup(&self, key: &H256) -> Option<&Bytes>;
	fn exists(&self, key: &H256) -> bool;
	fn insert(&mut self, value: &[u8]) -> H256;
	fn kill(&mut self, key: &H256);
}
