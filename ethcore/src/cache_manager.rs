use std::collections::{VecDeque, HashSet};
use std::hash::Hash;

const COLLECTION_QUEUE_SIZE: usize = 8;

pub struct CacheManager<T> where T: Eq + Hash {
	pref_cache_size: usize,
	max_cache_size: usize,
	bytes_per_cache_entry: usize,
	cache_usage: VecDeque<HashSet<T>>
}

impl<T> CacheManager<T> where T: Eq + Hash {
	pub fn new(pref_cache_size: usize, max_cache_size: usize, bytes_per_cache_entry: usize) -> Self {
		CacheManager {
			pref_cache_size: pref_cache_size,
			max_cache_size: max_cache_size,
			bytes_per_cache_entry: bytes_per_cache_entry,
			cache_usage: (0..COLLECTION_QUEUE_SIZE).into_iter().map(|_| Default::default()).collect(),
		}
	}

	pub fn note_used(&mut self, id: T) {
		if !self.cache_usage[0].contains(&id) {
			if let Some(c) = self.cache_usage.iter_mut().skip(1).find(|e| e.contains(&id)) {
				c.remove(&id);
			}
			self.cache_usage[0].insert(id);
		}
	}

	pub fn collect_carbage<C, F>(&mut self, current_size: C, notify_unused: F) where C: Fn() -> usize, F: Fn(T) {
		if current_size() < self.pref_cache_size {
			self.rotate_cache_if_needed();
			return;
		}

		for i in 0..COLLECTION_QUEUE_SIZE {
			for id in self.cache_usage.pop_back().unwrap().into_iter() {
				notify_unused(id)
			}
			self.cache_usage.push_front(Default::default());
			if current_size() < self.max_cache_size {
				break;
			}
		}
	}

	fn rotate_cache_if_needed(&mut self) {
		if self.cache_usage[0].len() * self.bytes_per_cache_entry > self.pref_cache_size / COLLECTION_QUEUE_SIZE {
			let cache = self.cache_usage.pop_back().unwrap();
			self.cache_usage.push_front(cache);
		}
	}
}
