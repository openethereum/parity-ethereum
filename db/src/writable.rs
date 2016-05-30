// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Readable trait mod

use cache::{Key, Cache, CacheUpdatePolicy};
use traits::DBTransaction;
use util::rlp::{encode, Encodable};
use std::ops::Deref;
use std::hash::Hash;
use std::collections::HashMap;
use util;

/// Should be used to write value into database.
pub trait Writable {
	/// Writes the value into the database.
	fn write<T, R>(&self, key: &Key<T, Target = R>, value: &T) where T: Encodable, R: Deref<Target = [u8]>;

	/// Writes the value into the database and updates the cache.
	fn write_with_cache<K, T, R>(&self, cache: &mut Cache<K, T>, key: K, value: T, policy: CacheUpdatePolicy)
	where K: Key<T, Target = R> + Hash + Eq,
		T: Encodable,
		R: Deref<Target = [u8]>
	{
		self.write(&key, &value);
		match policy {
			CacheUpdatePolicy::Overwrite => {
				cache.insert(key, value);
			},
			CacheUpdatePolicy::Remove => {
				cache.remove(&key);
			}
		}
	}

	/// Writes the values into the database and updates the cache.
	fn extend_with_cache<K, T, R>(&self, cache: &mut Cache<K, T>, values: HashMap<K, T>, policy: CacheUpdatePolicy)
		where K: Key<T, Target = R> + Hash + Eq,
			T: Encodable,
			R: Deref<Target = [u8]>
	{
		match policy {
			CacheUpdatePolicy::Overwrite => {
				for (key, value) in values.into_iter() {
					self.write(&key, &value);
					cache.insert(key, value);
				}
			},
			CacheUpdatePolicy::Remove => {
				for (key, value) in &values {
					self.write(key, value);
					cache.remove(key);
				}
			},
		}
	}
}

impl Writable for util::DBTransaction {
	fn write<T, R>(&self, key: &Key<T, Target = R>, value: &T) where T: Encodable, R: Deref<Target = [u8]> {
		let result = self.put(&key.key(), &encode(value));
		if let Err(err) = result {
			panic!("db put failed, key: {:?}, err: {:?}", &key.key() as &[u8], err);
		}
	}
}

impl Writable for DBTransaction {
	fn write<T, R>(&self, key: &Key<T, Target = R>, value: &T) where T: Encodable, R: Deref<Target = [u8]> {
		self.put(&key.key(), &encode(value));
	}
}
