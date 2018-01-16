// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use ethereum_types::U256;
use vm::ReturnData;

const MAX_RETURN_WASTE_BYTES: usize = 16384;

pub trait Memory {
	/// Retrieve current size of the memory
	fn size(&self) -> usize;
	/// Resize (shrink or expand) the memory to specified size (fills 0)
	fn resize(&mut self, new_size: usize);
	/// Resize the memory only if its smaller
	fn expand(&mut self, new_size: usize);
	/// Write single byte to memory
	fn write_byte(&mut self, offset: U256, value: U256);
	/// Write a word to memory. Does not resize memory!
	fn write(&mut self, offset: U256, value: U256);
	/// Read a word from memory
	fn read(&self, offset: U256) -> U256;
	/// Write slice of bytes to memory. Does not resize memory!
	fn write_slice(&mut self, offset: U256, &[u8]);
	/// Retrieve part of the memory between offset and offset + size
	fn read_slice(&self, offset: U256, size: U256) -> &[u8];
	/// Retrieve writeable part of memory
	fn writeable_slice(&mut self, offset: U256, size: U256) -> &mut[u8];
	fn dump(&self);
	/// Convert memory into return data.
	fn into_return_data(self, offset: U256, size: U256) -> ReturnData;
}

/// Checks whether offset and size is valid memory range
pub fn is_valid_range(off: usize, size: usize)  -> bool {
	// When size is zero we haven't actually expanded the memory
	let overflow = off.overflowing_add(size).1;
	size > 0 && !overflow
}

impl Memory for Vec<u8> {
	fn dump(&self) {
		println!("MemoryDump:");
		for i in self.iter() {
			println!("{:02x} ", i);
		}
		println!("");
	}

	fn size(&self) -> usize {
		self.len()
	}

	fn read_slice(&self, init_off_u: U256, init_size_u: U256) -> &[u8] {
		let off = init_off_u.low_u64() as usize;
		let size = init_size_u.low_u64() as usize;
		if !is_valid_range(off, size) {
			&self[0..0]
		} else {
			&self[off..off+size]
		}
	}

	fn read(&self, offset: U256) -> U256 {
		let off = offset.low_u64() as usize;
		U256::from(&self[off..off+32])
	}

	fn writeable_slice(&mut self, offset: U256, size: U256) -> &mut [u8] {
		let off = offset.low_u64() as usize;
		let s = size.low_u64() as usize;
		if !is_valid_range(off, s) {
			&mut self[0..0]
		} else {
			&mut self[off..off+s]
		}
	}

	fn write_slice(&mut self, offset: U256, slice: &[u8]) {
		if !slice.is_empty() {
			let off = offset.low_u64() as usize;
			self[off..off+slice.len()].copy_from_slice(slice);
		}
	}

	fn write(&mut self, offset: U256, value: U256) {
		let off = offset.low_u64() as usize;
		value.to_big_endian(&mut self[off..off+32]);
	}

	fn write_byte(&mut self, offset: U256, value: U256) {
		let off = offset.low_u64() as usize;
		let val = value.low_u64() as u64;
		self[off] = val as u8;
	}

	fn resize(&mut self, new_size: usize) {
		self.resize(new_size, 0);
	}

	fn expand(&mut self, size: usize) {
		if size > self.len() {
			Memory::resize(self, size)
		}
	}

	fn into_return_data(mut self, offset: U256, size: U256) -> ReturnData {
		let mut offset = offset.low_u64() as usize;
		let size = size.low_u64() as usize;
		if !is_valid_range(offset, size) {
			return ReturnData::empty()
		}
		if self.len() - size > MAX_RETURN_WASTE_BYTES {
			{ let _ =  self.drain(..offset); }
			self.truncate(size);
			self.shrink_to_fit();
			offset = 0;
		}
		ReturnData::new(self, offset, size)
	}
}

#[cfg(test)]
mod tests {
	use ethereum_types::U256;
	use super::Memory;

	#[test]
	fn test_memory_read_and_write() {
		// given
		let mem: &mut Memory = &mut vec![];
		mem.resize(0x80 + 32);

		// when
		mem.write(U256::from(0x80), U256::from(0xabcdef));

		// then
		assert_eq!(mem.read(U256::from(0x80)), U256::from(0xabcdef));
	}

	#[test]
	fn test_memory_read_and_write_byte() {
		// given
		let mem: &mut Memory = &mut vec![];
		mem.resize(32);

		// when
		mem.write_byte(U256::from(0x1d), U256::from(0xab));
		mem.write_byte(U256::from(0x1e), U256::from(0xcd));
		mem.write_byte(U256::from(0x1f), U256::from(0xef));

		// then
		assert_eq!(mem.read(U256::from(0x00)), U256::from(0xabcdef));
	}

	#[test]
	fn test_memory_read_slice_and_write_slice() {
		let mem: &mut Memory = &mut vec![];
		mem.resize(32);

		{
			let slice = "abcdefghijklmnopqrstuvwxyz012345".as_bytes();
			mem.write_slice(U256::from(0), slice);

			assert_eq!(mem.read_slice(U256::from(0), U256::from(32)), slice);
		}

		// write again
		{
			let slice = "67890".as_bytes();
			mem.write_slice(U256::from(0x1), slice);

			assert_eq!(mem.read_slice(U256::from(0), U256::from(7)), "a67890g".as_bytes());
		}

		// write empty slice out of bounds
		{
			let slice = [];
			mem.write_slice(U256::from(0x1000), &slice);
			assert_eq!(mem.size(), 32);
		}
	}
}
