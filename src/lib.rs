#[macro_use] extern crate log;

/// Silly function to return 69.
///
/// # Example
///
/// ```
/// assert_eq!(ethcore::sixtynine(), 69);
/// ```
pub fn sixtynine() -> i32 {
	debug!("Hello world!");
	69
}

