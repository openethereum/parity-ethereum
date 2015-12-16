use std::collections::HashMap;
use std::hash::Hash;
use heapsize::HeapSizeOf;

/// Should be used to squeeze collections to certain size in bytes
pub trait Squeeze {
	fn squeeze(&mut self, size: usize);
}

impl<K, T> Squeeze for HashMap<K, T> where K: Eq + Hash + Clone + HeapSizeOf, T: HeapSizeOf {
	fn squeeze(&mut self, size: usize) {
		if self.len() == 0 {
			return
		}
		
		let size_of_entry = self.heap_size_of_children() / self.capacity();
		let mut shrinked_size = size_of_entry * self.len();

		while self.len() > 0 || shrinked_size > size {
			// could be optimized
			let key = self.keys().next().unwrap().clone();
			self.remove(&key);
			shrinked_size -= size_of_entry;
		}

		self.shrink_to_fit();

		// if we havent shrinked enough, squeeze again
		if self.heap_size_of_children() > size {
			self.squeeze(size);
		}
	}
}
