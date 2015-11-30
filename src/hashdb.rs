use hash::*;
use bytes::*;

pub trait HashDB {
	/// Look up a given hash into the bytes that hash to it, returning None if the 
	/// hash is not known.
	///
	/// # Examples
	/// ```rust
	/// extern crate ethcore_util;
	/// use ethcore_util::hashdb::*;
	/// use ethcore_util::memorydb::*;
	/// fn main() {
	///   let mut m = MemoryDB::new();
	///   let hello_bytes = "Hello world!".as_bytes();
	///   let hash = m.insert(hello_bytes);
	///   assert_eq!(m.lookup(&hash).unwrap(), hello_bytes);
	/// }
	/// ```
	fn lookup(&self, key: &H256) -> Option<&[u8]>;

	/// Check for the existance of a hash-key.
	///
	/// # Examples
	/// ```rust
	/// extern crate ethcore_util;
	/// use ethcore_util::hashdb::*;
	/// use ethcore_util::memorydb::*;
	/// use ethcore_util::sha3::*;
	/// fn main() {
	///   let mut m = MemoryDB::new();
	///   let hello_bytes = "Hello world!".as_bytes();
	///   assert!(!m.exists(&hello_bytes.sha3()));
	///   let key = m.insert(hello_bytes);
	///   assert!(m.exists(&key));
	///   m.kill(&key);
	///   assert!(!m.exists(&key));
	/// }
	/// ```
	fn exists(&self, key: &H256) -> bool;

	/// Insert a datum item into the DB and return the datum's hash for a later lookup. Insertions
	/// are counted and the equivalent number of `kill()`s must be performed before the data
	/// is considered dead.
	///
	/// # Examples
	/// ```rust
	/// extern crate ethcore_util;
	/// use ethcore_util::hashdb::*;
	/// use ethcore_util::memorydb::*;
	/// use ethcore_util::hash::*;
	/// fn main() {
	///   let mut m = MemoryDB::new();
	///   let key = m.insert("Hello world!".as_bytes());
	///   assert!(m.exists(&key));
	/// }
	/// ```
	fn insert(&mut self, value: &[u8]) -> H256;

	/// Like `insert()` , except you provide the key and the data is all moved.
	fn emplace(&mut self, key: H256, value: Bytes);

	/// Remove a datum previously inserted. Insertions can be "owed" such that the same number of `insert()`s may
	/// happen without the data being eventually being inserted into the DB.
	///
	/// # Examples
	/// ```rust
	/// extern crate ethcore_util;
	/// use ethcore_util::hashdb::*;
	/// use ethcore_util::memorydb::*;
	/// use ethcore_util::sha3::*;
	/// fn main() {
	///   let mut m = MemoryDB::new();
	///   let d = "Hello world!".as_bytes();
	///   let key = &d.sha3();
	///   m.kill(key);	// OK - we now owe an insertion.
	///   assert!(!m.exists(key));
	///   m.insert(d);	// OK - now it's "empty" again.
	///   assert!(!m.exists(key));
	///   m.insert(d);	// OK - now we've
	///   assert_eq!(m.lookup(key).unwrap(), d);
	/// }
	/// ```
	fn kill(&mut self, key: &H256);
}
