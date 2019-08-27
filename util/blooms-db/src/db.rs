// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::{error, io, fmt};
use std::path::{Path, PathBuf};
use ethbloom;
use crate::file::{File, FileIterator};

fn other_io_err<E>(e: E) -> io::Error where E: Into<Box<dyn error::Error + Send + Sync>> {
	io::Error::new(io::ErrorKind::Other, e)
}

/// Bloom positions in database files.
#[derive(Debug)]
struct Positions {
	top: u64,
	mid: u64,
	bot: u64
}

impl Positions {
	fn from_index(index: u64) -> Self {
		Positions {
			top: index >> 8,
			mid: index >> 4,
			bot: index,
		}
	}
}

struct DatabaseFilesIterator<'a> {
	pub top: FileIterator<'a>,
	pub mid: FileIterator<'a>,
	pub bot: FileIterator<'a>,
}

/// Blooms database files.
struct DatabaseFiles {
	/// Top level bloom file
	///
	/// Every bloom represents 16 blooms on mid level
	top: File,
	/// Mid level bloom file
	///
	/// Every bloom represents 16 blooms on bot level
	mid: File,
	/// Bot level bloom file
	///
	/// Every bloom is an ethereum header bloom
	bot: File,
}

impl DatabaseFiles {
	/// Open the blooms db files
	pub fn open(path: &Path) -> io::Result<DatabaseFiles> {
		Ok(DatabaseFiles {
			top: File::open(path.join("top.bdb"))?,
			mid: File::open(path.join("mid.bdb"))?,
			bot: File::open(path.join("bot.bdb"))?,
		})
	}

	pub fn accrue_bloom(&mut self, pos: Positions, bloom: ethbloom::BloomRef) -> io::Result<()> {
		self.top.accrue_bloom::<ethbloom::BloomRef>(pos.top, bloom)?;
		self.mid.accrue_bloom::<ethbloom::BloomRef>(pos.mid, bloom)?;
		self.bot.replace_bloom::<ethbloom::BloomRef>(pos.bot, bloom)?;
		Ok(())
	}

	pub fn iterator_from(&mut self, pos: Positions) -> io::Result<DatabaseFilesIterator> {
		Ok(DatabaseFilesIterator {
			top: self.top.iterator_from(pos.top)?,
			mid: self.mid.iterator_from(pos.mid)?,
			bot: self.bot.iterator_from(pos.bot)?,
		})
	}

	fn flush(&mut self) -> io::Result<()> {
		self.top.flush()?;
		self.mid.flush()?;
		self.bot.flush()?;
		Ok(())
	}
}

impl Drop for DatabaseFiles {
	/// Flush the database files on drop
	fn drop(&mut self) {
		self.flush().ok();
	}
}

/// Blooms database.
pub struct Database {
	/// Database files
	db_files: Option<DatabaseFiles>,
	/// Database path
	path: PathBuf,
}

impl Database {
	/// Opens blooms database.
	pub fn open<P>(path: P) -> io::Result<Database> where P: AsRef<Path> {
		let path: PathBuf = path.as_ref().to_path_buf();
		let database = Database {
			db_files: Some(DatabaseFiles::open(&path)?),
			path: path,
		};

		Ok(database)
	}

	/// Close the inner-files
	pub fn close(&mut self) -> io::Result<()> {
		self.db_files = None;
		Ok(())
	}

	/// Reopens the database at the same location.
	pub fn reopen(&mut self) -> io::Result<()> {
		self.db_files = Some(DatabaseFiles::open(&self.path)?);
		Ok(())
	}

	/// Insert consecutive blooms into database starting at the given positon.
	pub fn insert_blooms<'a, I, B>(&mut self, from: u64, blooms: I) -> io::Result<()>
	where ethbloom::BloomRef<'a>: From<B>, I: Iterator<Item = B> {
		match self.db_files {
			Some(ref mut db_files) => {
				for (index, bloom) in (from..).into_iter().zip(blooms.map(Into::into)) {
					let pos = Positions::from_index(index);

					// Constant forks may lead to increased ratio of false positives in bloom filters
					// since we do not rebuild top or mid level, but we should not be worried about that
					// because most of the time events at block n(a) occur also on block n(b) or n+1(b)
					db_files.accrue_bloom(pos, bloom)?;
				}
				db_files.flush()?;
				Ok(())
			},
			None => Err(other_io_err("Database is closed")),
		}
	}

	/// Returns an iterator yielding all indexes containing given bloom.
	pub fn iterate_matching<'a, 'b, B, I, II>(&'a mut self, from: u64, to: u64, blooms: II) -> io::Result<DatabaseIterator<'a, II>>
	where ethbloom::BloomRef<'b>: From<B>, 'b: 'a, II: IntoIterator<Item = B, IntoIter = I> + Copy, I: Iterator<Item = B> {
		match self.db_files {
			Some(ref mut db_files) => {
				let index = from / 256 * 256;
				let pos = Positions::from_index(index);
				let files_iter = db_files.iterator_from(pos)?;

				let iter = DatabaseIterator {
					top: files_iter.top,
					mid: files_iter.mid,
					bot: files_iter.bot,
					state: IteratorState::Top,
					from,
					to,
					index,
					blooms,
				};

				Ok(iter)
			},
			None => Err(other_io_err("Database is closed")),
		}
	}
}

fn contains_any<'a, I, B>(bloom: ethbloom::Bloom, mut iterator: I) -> bool
where ethbloom::BloomRef<'a>: From<B>, I: Iterator<Item = B> {
	iterator.any(|item| bloom.contains_bloom(item))
}

/// Blooms database iterator
pub struct DatabaseIterator<'a, I> {
	top: FileIterator<'a>,
	mid: FileIterator<'a>,
	bot: FileIterator<'a>,
	state: IteratorState,
	from: u64,
	to: u64,
	index: u64,
	blooms: I,
}

impl<'a, I> fmt::Debug for DatabaseIterator<'a, I> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("DatabaseIterator")
			.field("state", &self.state)
			.field("from", &self.from)
			.field("to", &self.to)
			.field("index", &self.index)
			.field("blooms", &"...")
			.field("top", &"...")
			.field("mid", &"...")
			.field("bot", &"...")
			.finish()
	}
}

/// Database iterator state.
#[derive(Debug)]
enum IteratorState {
	/// Iterator should read top level bloom
	Top,
	/// Iterator should read mid level bloom `x` more times
	Mid(usize),
	/// Iterator should read mid level bloom `mid` more times
	/// and bot level `mix * 16 + bot` times
	Bot { mid: usize, bot: usize },
}

impl<'a, 'b, B, I, II> Iterator for DatabaseIterator<'a, II>
where ethbloom::BloomRef<'b>: From<B>, 'b: 'a, II: IntoIterator<Item = B, IntoIter = I> + Copy, I: Iterator<Item = B> {
	type Item = io::Result<u64>;

	fn next(&mut self) -> Option<Self::Item> {
		macro_rules! try_o {
			($expr: expr) => {
				match $expr {
					Err(err) => return Some(Err(err)),
					Ok(ok) => ok,
				}
			}
		}

		macro_rules! next_bloom {
			($iter: expr) => {
				try_o!($iter.next()?)
			}
		}

		loop {
			if self.index > self.to {
				return None;
			}

			self.state = match self.state {
				IteratorState::Top => {
					if contains_any(next_bloom!(self.top), self.blooms.into_iter()) {
						IteratorState::Mid(16)
					} else {
						self.index += 256;
						try_o!(self.mid.advance(16));
						try_o!(self.bot.advance(256));
						IteratorState::Top
					}
				},
				IteratorState::Mid(left) => {
					if left == 0 {
						IteratorState::Top
					} else if contains_any(next_bloom!(self.mid), self.blooms.into_iter()) && self.index + 16 >= self.from {
						IteratorState::Bot { mid: left - 1, bot: 16 }
					} else {
						self.index += 16;
						try_o!(self.bot.advance(16));
						IteratorState::Mid(left - 1)
					}
				},
				IteratorState::Bot { mid, bot } => {
					if bot == 0 {
						IteratorState::Mid(mid)
					} else if contains_any(next_bloom!(self.bot), self.blooms.into_iter()) && self.index >= self.from {
						let result = self.index;
						self.index += 1;
						self.state = IteratorState::Bot { mid, bot: bot - 1 };
						return Some(Ok(result));
					} else {
						self.index += 1;
						IteratorState::Bot { mid, bot: bot - 1 }
					}
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use ethbloom::Bloom;
	use tempdir::TempDir;
	use super::Database;

	#[test]
	fn test_database() {
		let tempdir = TempDir::new("").unwrap();
		let mut database = Database::open(tempdir.path()).unwrap();
		database.insert_blooms(0, vec![
			Bloom::from_low_u64_be(0),
			Bloom::from_low_u64_be(0x01),
			Bloom::from_low_u64_be(0x10),
			Bloom::from_low_u64_be(0x11),
		].iter()).unwrap();

		let matches = database.iterate_matching(0, 3, Some(&Bloom::zero())).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![0, 1, 2, 3]);

		let matches = database.iterate_matching(0, 4, Some(&Bloom::zero())).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![0, 1, 2, 3]);

		let matches = database.iterate_matching(1, 3, Some(&Bloom::zero())).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![1, 2, 3]);

		let matches = database.iterate_matching(1, 2, Some(&Bloom::zero())).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![1, 2]);

		let matches = database.iterate_matching(0, 3, Some(&Bloom::from_low_u64_be(0x01))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![1, 3]);

		let matches = database.iterate_matching(0, 3, Some(&Bloom::from_low_u64_be(0x10))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![2, 3]);

		let matches = database.iterate_matching(2, 2, Some(&Bloom::from_low_u64_be(0x10))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![2]);
	}

	#[test]
	fn test_database2() {
		let tempdir = TempDir::new("").unwrap();
		let mut database = Database::open(tempdir.path()).unwrap();
		database.insert_blooms(254, vec![
			Bloom::from_low_u64_be(0x100),
			Bloom::from_low_u64_be(0x01),
			Bloom::from_low_u64_be(0x10),
			Bloom::from_low_u64_be(0x11),
		].iter()).unwrap();

		let matches = database.iterate_matching(0, 257, Some(&Bloom::from_low_u64_be(0x01))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![255, 257]);

		let matches = database.iterate_matching(0, 258, Some(&Bloom::from_low_u64_be(0x100))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![254]);

		let matches = database.iterate_matching(0, 256, Some(&Bloom::from_low_u64_be(0x01))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![255]);

		let matches = database.iterate_matching(255, 255, Some(&Bloom::from_low_u64_be(0x01))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![255]);

		let matches = database.iterate_matching(256, 256, Some(&Bloom::from_low_u64_be(0x10))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![256]);

		let matches = database.iterate_matching(256, 257, Some(&Bloom::from_low_u64_be(0x10))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
		assert_eq!(matches, vec![256, 257]);
	}

	#[test]
	fn test_db_close() {
		let tempdir = TempDir::new("").unwrap();
		let blooms = vec![
			Bloom::from_low_u64_be(0x100),
			Bloom::from_low_u64_be(0x01),
			Bloom::from_low_u64_be(0x10),
			Bloom::from_low_u64_be(0x11),
		];
		let mut database = Database::open(tempdir.path()).unwrap();

		// Close the DB and ensure inserting blooms errors
		database.close().unwrap();
		assert!(database.insert_blooms(254, blooms.iter()).is_err());

		// Reopen it and ensure inserting blooms is OK
		database.reopen().unwrap();
		assert!(database.insert_blooms(254, blooms.iter()).is_ok());
	}
}
