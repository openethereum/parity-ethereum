use std::ptr;

pub trait InsertSlice<T> {
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

pub trait SharedPreifx {
}
