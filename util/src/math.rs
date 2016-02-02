//! Common math functions.

/// Returns log2.
pub fn log2(x: usize) -> u32 {
	if x <= 1 {
		return 0;
	}

	let n = x.leading_zeros();
	::std::mem::size_of::<usize>() as u32 * 8 - n
}
