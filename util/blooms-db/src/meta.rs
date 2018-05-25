use std::io::{Write, Read};
use std::path::Path;
use std::{fs, io};

use byteorder::{WriteBytesExt, ReadBytesExt, LittleEndian};

#[derive(Debug)]
pub struct Meta {
	/// Database version.
	pub version: u64,
	/// Pending file hash.
	pub pending_hash: [u8; 32],
}

pub fn read_meta<P>(path: P) -> io::Result<Meta> where P: AsRef<Path> {
	let mut file = fs::OpenOptions::new()
		.read(true)
		.open(path)?;

	let version = file.read_u64::<LittleEndian>()?;
	let mut pending_hash = [0u8; 32];
	file.read_exact(&mut pending_hash)?;

	let meta = Meta {
		version,
		pending_hash,
	};

	Ok(meta)
}

pub fn save_meta<P>(path: P, meta: &Meta) -> io::Result<()> where P: AsRef<Path> {
	let mut file = fs::OpenOptions::new()
		.write(true)
		.create(true)
		.open(path)?;

	file.write_u64::<LittleEndian>(meta.version)?;
	file.write_all(&meta.pending_hash)?;
	file.flush()
}
