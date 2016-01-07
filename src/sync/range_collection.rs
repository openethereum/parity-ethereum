use std::ops::{Add, Sub, Range};

pub trait ToUsize {
	fn to_usize(&self) -> usize;
}

pub trait FromUsize {
	fn from_usize(s: usize) -> Self;
}

pub trait RangeCollection<K, V> {
	fn have_item(&self, key: &K) -> bool;
	fn find_item(&self, key: &K) -> Option<&V>;
	fn get_tail(&mut self, key: &K) -> Range<K>;
	fn remove_head(&mut self, start: &K);
	fn remove_tail(&mut self, start: &K);
	fn insert_item(&mut self, key: K, value: V);
	fn range_iter<'c>(&'c self) -> RangeIterator<'c, K, V>;
}

pub struct RangeIterator<'c, K:'c, V:'c> {
	range: usize,
	collection: &'c Vec<(K, Vec<V>)>
}

impl<'c, K:'c, V:'c> Iterator for RangeIterator<'c, K, V> where K: Add<Output = K> + FromUsize + ToUsize + Copy {
    type Item = (K, &'c [V]);
    // The 'Iterator' trait only requires the 'next' method to be defined. The
    // return type is 'Option<T>', 'None' is returned when the 'Iterator' is
    // over, otherwise the next value is returned wrapped in 'Some'
    fn next(&mut self) -> Option<(K, &'c [V])> {
		if self.range > 0 {
			self.range -= 1;
		}
		else {
			return None;
		}
		match self.collection.get(self.range) {
			Some(&(ref k, ref vec)) => {
				Some((*k, &vec))
			},
			None => None
		}
    }
}

impl<K, V> RangeCollection<K, V> for Vec<(K, Vec<V>)> where K: Ord + PartialEq + Add<Output = K> + Sub<Output = K> + Copy + FromUsize + ToUsize {
	fn range_iter<'c>(&'c self) -> RangeIterator<'c, K, V> {
		RangeIterator {
			range: self.len(),
			collection: self
		}
	}

	fn have_item(&self, key: &K) -> bool {
		match self.binary_search_by(|&(k, _)| k.cmp(key).reverse()) {
			Ok(_) => true,
			Err(index) => match self.get(index) {
				Some(&(ref k, ref v)) => k <= key && (*k + FromUsize::from_usize(v.len())) > *key,
				_ => false
			},
		}
	}

	fn find_item(&self, key: &K) -> Option<&V> {
		match self.binary_search_by(|&(k, _)| k.cmp(key).reverse()) {
			Ok(index) => self.get(index).unwrap().1.get(0),
			Err(index) => match self.get(index) {
				Some(&(ref k, ref v)) if k <= key && (*k + FromUsize::from_usize(v.len())) > *key => v.get((*key - *k).to_usize()),
				_ => None
			},
		}
	}

	/// Get a range of elements from start till the end of the range
	fn get_tail(&mut self, key: &K) -> Range<K> {
		let kv = *key;
		match self.binary_search_by(|&(k, _)| k.cmp(key).reverse()) {
			Ok(index) => kv..(kv + FromUsize::from_usize(self[index].1.len())),
			Err(index) => {
				match self.get_mut(index) {
					Some(&mut (ref k, ref mut v)) if k <= key && (*k + FromUsize::from_usize(v.len())) > *key => {
						kv..(*k + FromUsize::from_usize(v.len()))
					}
					_ => kv..kv
				}
			},
		}
	}
	/// Remove element key and following elements in the same range
	fn remove_tail(&mut self, key: &K) {
		match self.binary_search_by(|&(k, _)| k.cmp(key).reverse()) {
			Ok(index) => { self.remove(index); },
			Err(index) =>{
				let mut empty = false;
				match self.get_mut(index) {
					Some(&mut (ref k, ref mut v)) if k <= key && (*k + FromUsize::from_usize(v.len())) > *key => {
						v.truncate((*key - *k).to_usize());
						empty = v.is_empty();
					}
					_ => {}
				}
				if empty {
					self.remove(index);
				}
			},
		}
	}

	/// Remove range elements up to key
	fn remove_head(&mut self, key: &K) {
		if *key == FromUsize::from_usize(0) {
			return
		}

		let prev = *key - FromUsize::from_usize(1);
		match self.binary_search_by(|&(k, _)| k.cmp(key).reverse()) {
			Ok(_) => { }, //start of range, do nothing.
			Err(index) => {
				let mut empty = false;
				match self.get_mut(index) {
					Some(&mut (ref mut k, ref mut v)) if *k <= prev && (*k + FromUsize::from_usize(v.len())) > prev => {
						let tail = v.split_off((*key - *k).to_usize());
						empty = tail.is_empty();
						let removed = ::std::mem::replace(v, tail);
						let new_k = *k + FromUsize::from_usize(removed.len());
						::std::mem::replace(k, new_k);
					}
					_ => {}
				}
				if empty {
					self.remove(index);
				}
			},
		}
	}

	fn insert_item(&mut self, key: K, value: V) {
		assert!(!self.have_item(&key));

		let lower = match self.binary_search_by(|&(k, _)| k.cmp(&key).reverse()) {
			Ok(index) => index,
			Err(index) => index,
		};

		let mut to_remove: Option<usize> = None;
		if lower < self.len() && self[lower].0 + FromUsize::from_usize(self[lower].1.len()) == key {
				// extend into existing chunk
				self[lower].1.push(value);
		}
		else {
			// insert a new chunk
			let range: Vec<V> = vec![value];
			self.insert(lower, (key, range));
		};
		if lower > 0 {
			let next = lower - 1;
			if next < self.len()
			{
				{
					let (mut next, mut inserted) = self.split_at_mut(lower);
					let mut next = next.last_mut().unwrap();
					let mut inserted = inserted.first_mut().unwrap();
					if next.0 == key + FromUsize::from_usize(1)
					{
						inserted.1.append(&mut next.1);
						to_remove = Some(lower - 1);
					}
				}

				if let Some(r) = to_remove {
					self.remove(r);
				}
			}
		}
	}
}

#[test]
fn test_range() {
	use std::cmp::{Ordering};

	let mut ranges: Vec<(u64, Vec<char>)> = Vec::new();
	assert_eq!(ranges.range_iter().next(), None);
	assert_eq!(ranges.find_item(&1), None);
	assert!(!ranges.have_item(&1));
	assert_eq!(ranges.get_tail(&0), 0..0);

	ranges.insert_item(17, 'q');
	assert_eq!(ranges.range_iter().cmp(vec![(17, &['q'][..])]),  Ordering::Equal);
	assert_eq!(ranges.find_item(&17), Some(&'q'));
	assert!(ranges.have_item(&17));
	assert_eq!(ranges.get_tail(&17), 17..18);

	ranges.insert_item(18, 'r');
	assert_eq!(ranges.range_iter().cmp(vec![(17, &['q', 'r'][..])]),  Ordering::Equal);
	assert_eq!(ranges.find_item(&18), Some(&'r'));
	assert!(ranges.have_item(&18));
	assert_eq!(ranges.get_tail(&17), 17..19);

	ranges.insert_item(16, 'p');
	assert_eq!(ranges.range_iter().cmp(vec![(16, &['p', 'q', 'r'][..])]),  Ordering::Equal);
	assert_eq!(ranges.find_item(&16), Some(&'p'));
	assert_eq!(ranges.find_item(&17), Some(&'q'));
	assert_eq!(ranges.find_item(&18), Some(&'r'));
	assert!(ranges.have_item(&16));
	assert_eq!(ranges.get_tail(&17), 17..19);

	ranges.insert_item(2, 'b');
	assert_eq!(ranges.range_iter().cmp(vec![(2, &['b'][..]),  (16, &['p', 'q', 'r'][..])]),  Ordering::Equal);
	assert_eq!(ranges.find_item(&2), Some(&'b'));

	ranges.insert_item(3, 'c');
	ranges.insert_item(4, 'd');
	assert_eq!(ranges.get_tail(&3), 3..5);
	assert_eq!(ranges.range_iter().cmp(vec![(2, &['b', 'c', 'd'][..]),  (16, &['p', 'q', 'r'][..])]),  Ordering::Equal);

	let mut r = ranges.clone();
	r.remove_head(&1);
	assert_eq!(r.range_iter().cmp(vec![(2, &['b', 'c', 'd'][..]),  (16, &['p', 'q', 'r'][..])]),  Ordering::Equal);
	r.remove_head(&2);
	assert_eq!(r.range_iter().cmp(vec![(2, &['b', 'c', 'd'][..]),  (16, &['p', 'q', 'r'][..])]),  Ordering::Equal);
	r.remove_head(&3);
	assert_eq!(r.range_iter().cmp(vec![(3, &['c', 'd'][..]),  (16, &['p', 'q', 'r'][..])]),  Ordering::Equal);
	r.remove_head(&10);
	assert_eq!(r.range_iter().cmp(vec![(3, &['c', 'd'][..]),  (16, &['p', 'q', 'r'][..])]),  Ordering::Equal);
	r.remove_head(&5);
	assert_eq!(r.range_iter().cmp(vec![(16, &['p', 'q', 'r'][..])]),  Ordering::Equal);
	r.remove_head(&19);
	assert_eq!(r.range_iter().next(), None);

	let mut r = ranges.clone();
	r.remove_tail(&20);
	assert_eq!(r.range_iter().cmp(vec![(2, &['b', 'c', 'd'][..]),  (16, &['p', 'q', 'r'][..])]),  Ordering::Equal);
	r.remove_tail(&17);
	assert_eq!(r.range_iter().cmp(vec![(2, &['b', 'c', 'd'][..]),  (16, &['p'][..])]),  Ordering::Equal);
	r.remove_tail(&16);
	assert_eq!(r.range_iter().cmp(vec![(2, &['b', 'c', 'd'][..])]),  Ordering::Equal);
	r.remove_tail(&3);
	assert_eq!(r.range_iter().cmp(vec![(2, &['b'][..])]),  Ordering::Equal);
	r.remove_tail(&2);
	assert_eq!(r.range_iter().next(), None);
}

