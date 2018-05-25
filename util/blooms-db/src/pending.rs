use std::io::{Seek, SeekFrom, Write, Read};
use std::path::Path;
use std::{fs, io};

use byteorder::{WriteBytesExt, ReadBytesExt, LittleEndian};
use ethbloom;
use tiny_keccak::Keccak;

/// File with blooms which are not flushed to the database yet.
pub struct Pending {
	file: fs::File,
}

impl Pending {
	/// Opens pending changes file. Creates new file if pending changes do not exist.
	pub fn open<P>(path: P) -> io::Result<Pending> where P: AsRef<Path> {
		let file = fs::OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.append(true)
			.open(path)?;

		let pending = Pending {
			file,
		};

		Ok(pending)
	}

	/// Pushes pending changes to a file.
	pub fn append<'a, B>(&mut self, index: u64, bloom: B) -> io::Result<()> where ethbloom::BloomRef<'a>: From<B> {
		self.file.write_u64::<LittleEndian>(index)?;
		self.file.write_all(ethbloom::BloomRef::from(bloom).data())
	}

	/// Flushes changes to underlying file.
	pub fn flush(&mut self) -> io::Result<()> {
		self.file.sync_all()
	}

	/// Clears underlying file.
	pub fn clear(&mut self) -> io::Result<()> {
		self.file.seek(SeekFrom::Start(0))?;
		self.file.set_len(0)?;
		self.file.sync_all()
	}

	/// Returns an iterator over blooms in the file.
	pub fn iterator(&self) -> io::Result<PendingIterator> {
		let mut file_ref = &self.file;
		file_ref.seek(SeekFrom::Start(0))?;

		let iter = PendingIterator {
			file: file_ref,
		};

		Ok(iter)
	}

	/// Returns file hash.
	pub fn hash(&self) -> io::Result<[u8; 32]> {
		let mut file_ref = &self.file;
		file_ref.seek(SeekFrom::Start(0))?;
		let mut keccak = Keccak::new_keccak256();
		let mut buffer = [0u8; 256 + 8];
		loop {
			match file_ref.read_exact(&mut buffer) {
				Ok(_) => {
					keccak.update(&mut buffer);
				},
				Err(ref err) if err.kind() == io::ErrorKind::UnexpectedEof => {
					let mut result = [0u8; 32];
					keccak.finalize(&mut result);
					return Ok(result);
				},
				Err(err) => return Err(err),
			}
		}
	}
}

/// Iterator over blooms in the file.
pub struct PendingIterator<'a> {
	file: &'a fs::File,
}

impl<'a> Iterator for PendingIterator<'a> {
	type Item = io::Result<(u64, ethbloom::Bloom)>;

	fn next(&mut self) -> Option<Self::Item> {
		let index = match self.file.read_u64::<LittleEndian>() {
			Ok(index) => index,
			Err(ref err) if err.kind() == io::ErrorKind::UnexpectedEof => return None,
			Err(err) => return Some(Err(err)),
		};

		let mut bloom = ethbloom::Bloom::default();
		match self.file.read_exact(&mut bloom) {
			Ok(_) => Some(Ok((index, bloom))),
			Err(err) => Some(Err(err)),
		}
	}
}

#[cfg(test)]
mod tests {
	use ethbloom::Bloom;
	use tempdir::TempDir;
	use super::Pending;

	#[test]
	fn test_pending() {
		let tempdir = TempDir::new("").unwrap();
		let mut pending = Pending::open(tempdir.path().join("pending")).unwrap();

		// append elements
		pending.append(0, &Bloom::from(0)).unwrap();
		pending.append(1, &Bloom::from(1)).unwrap();
		pending.append(2, &Bloom::from(2)).unwrap();
		pending.append(3, &Bloom::from(3)).unwrap();
		pending.append(2, &Bloom::from(4)).unwrap();

		// flush
		pending.flush().unwrap();

		// validate all elements
		let elements = pending.iterator().unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(elements, vec![
			(0, 0.into()),
			(1, 1.into()),
			(2, 2.into()),
			(3, 3.into()),
			(2, 4.into())
		]);

		// move iterator
		let first = pending.iterator().unwrap().next().unwrap().unwrap();
		assert_eq!(first, (0, 0.into()));

		// validate that after moving an iterator the element is still appended to the of the file
		pending.append(4, &Bloom::from(5)).unwrap();
		pending.flush().unwrap();
		let elements2 = pending.iterator().unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(elements2, vec![
			(0, 0.into()),
			(1, 1.into()),
			(2, 2.into()),
			(3, 3.into()),
			(2, 4.into()),
			(4, 5.into())
		]);

		pending.clear().unwrap();
		let elements3 = pending.iterator().unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert!(elements3.is_empty());
	}
}
