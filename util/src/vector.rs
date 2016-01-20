//! vector util functions

use std::ptr;

/// TODO [debris] Please document me
pub trait InsertSlice<T> {
				/// TODO [debris] Please document me
    fn insert_slice(&mut self, index: usize, elements: &[T]);
}

/// based on `insert` function implementation from standard library
impl<T> InsertSlice<T> for Vec<T> {
    fn insert_slice(&mut self, index: usize, elements: &[T]) {
        let e_len = elements.len();
        if e_len == 0 {
            return;
        }

        let len = self.len();
        assert!(index <= len);

        // space for the new element
        self.reserve(e_len);

        unsafe {
            {
                let p = self.as_mut_ptr().offset(index as isize);
                let ep = elements.as_ptr().offset(0);
                // shift everything by e_len, to make space
                ptr::copy(p, p.offset(e_len as isize), len - index);
                // write new element
                ptr::copy(ep, p, e_len); 
            }
            self.set_len(len + e_len);
        }
    }
}

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
	/// TODO [debris] Please document me
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
