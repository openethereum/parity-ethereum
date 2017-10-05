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

//! Fetchable Dapps support.

use std::fs;

use linked_hash_map::LinkedHashMap;
use page::local;
use handlers::FetchControl;

pub enum ContentStatus {
	Fetching(FetchControl),
	Ready(local::Dapp),
}

#[derive(Default)]
pub struct ContentCache {
	cache: LinkedHashMap<String, ContentStatus>,
}

impl ContentCache {
	pub fn insert(&mut self, content_id: String, status: ContentStatus) -> Option<ContentStatus> {
		self.cache.insert(content_id, status)
	}

	pub fn remove(&mut self, content_id: &str) -> Option<ContentStatus> {
		self.cache.remove(content_id)
	}

	pub fn get(&mut self, content_id: &str) -> Option<&mut ContentStatus> {
		self.cache.get_refresh(content_id)
	}

	pub fn clear_garbage(&mut self, expected_size: usize) -> Vec<(String, ContentStatus)> {
		let len = self.cache.len();

		if len <= expected_size {
			return Vec::new();
		}

		let mut removed = Vec::with_capacity(len - expected_size);

		while self.cache.len() > expected_size {
			let entry = self.cache.pop_front().expect("expected_size bounded at 0, len is greater; qed");

			match entry.1 {
				ContentStatus::Fetching(ref fetch) => {
					trace!(target: "dapps", "Aborting {} because of limit.", entry.0);
					// Mark as aborted
					fetch.abort()
				},
				ContentStatus::Ready(ref endpoint) => {
					trace!(target: "dapps", "Removing {} because of limit.", entry.0);
					// Remove path (dir or file)
					let res = fs::remove_dir_all(&endpoint.path()).or_else(|_| fs::remove_file(&endpoint.path()));
					if let Err(e) = res {
						warn!(target: "dapps", "Unable to remove dapp/content from cache: {:?}", e);
					}
				}
			}

			removed.push(entry);
		}
		removed
	}

	#[cfg(test)]
	pub fn len(&self) -> usize {
		self.cache.len()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn only_keys(data: Vec<(String, ContentStatus)>) -> Vec<String> {
		data.into_iter().map(|x| x.0).collect()
	}

	#[test]
	fn should_remove_least_recently_used() {
		// given
		let mut cache = ContentCache::default();
		cache.insert("a".into(), ContentStatus::Fetching(Default::default()));
		cache.insert("b".into(), ContentStatus::Fetching(Default::default()));
		cache.insert("c".into(), ContentStatus::Fetching(Default::default()));

		// when
		let res = cache.clear_garbage(2);

		// then
		assert_eq!(cache.len(), 2);
		assert_eq!(only_keys(res), vec!["a"]);
	}

	#[test]
	fn should_update_lru_if_accessed() {
		// given
		let mut cache = ContentCache::default();
		cache.insert("a".into(), ContentStatus::Fetching(Default::default()));
		cache.insert("b".into(), ContentStatus::Fetching(Default::default()));
		cache.insert("c".into(), ContentStatus::Fetching(Default::default()));

		// when
		cache.get("a");
		let res = cache.clear_garbage(2);

		// then
		assert_eq!(cache.len(), 2);
		assert_eq!(only_keys(res), vec!["b"]);
	}

}
