//! Vector extensions.

/// Returns len of prefix shared with elem
/// 
/// ```rust
///	extern crate ethcore_util as util;
///	use util::vector::SharedPrefix;
///	
///	fn main () {
///		let a = vec![1,2,3,3,5];
///		let b = vec![1,2,3];
///		assert_eq!(a.shared_prefix_len(&b), 3);
///	}
/// ```
pub trait SharedPrefix <T> {
	/// Get common prefix length
	fn shared_prefix_len(&self, elem: &[T]) -> usize;
}

impl <T> SharedPrefix<T> for Vec<T> where T: Eq {
	fn shared_prefix_len(&self, elem: &[T]) -> usize {
		use std::cmp;
		let len = cmp::min(self.len(), elem.len());
		(0..len).take_while(|&i| self[i] == elem[i]).count()
	}
}

#[cfg(test)]
mod test {
	use vector::SharedPrefix;

	#[test]
	fn test_shared_prefix() {
		let a = vec![1,2,3,4,5,6];
		let b = vec![4,2,3,4,5,6];
		assert_eq!(a.shared_prefix_len(&b), 0);
	}

	#[test]
	fn test_shared_prefix2() {
		let a = vec![1,2,3,3,5];
		let b = vec![1,2,3];
		assert_eq!(a.shared_prefix_len(&b), 3);
	}
	
	#[test]
	fn test_shared_prefix3() {
		let a = vec![1,2,3,4,5,6];
		let b = vec![1,2,3,4,5,6];
		assert_eq!(a.shared_prefix_len(&b), 6);
	}
}
