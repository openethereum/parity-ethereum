//! Diff misc.

use common::*;

#[derive(Debug,Clone,PartialEq,Eq)]
/// Diff type for specifying a change (or not).
pub enum Diff<T> where T: Eq {
	/// Both sides are the same.
	Same,
	/// Left (pre, source) side doesn't include value, right side (post, destination) does.
	Born(T),
	/// Both sides include data; it chaged value between them.
	Changed(T, T),
	/// Left (pre, source) side does include value, right side (post, destination) does not.
	Died(T),
}

impl<T> Diff<T> where T: Eq {
	/// Construct new object with given `pre` and `post`.
	pub fn new(pre: T, post: T) -> Self { if pre == post { Diff::Same } else { Diff::Changed(pre, post) } }

	/// Get the before value, if there is one.
	pub fn pre(&self) -> Option<&T> { match *self { Diff::Died(ref x) | Diff::Changed(ref x, _) => Some(x), _ => None } }

	/// Get the after value, if there is one.
	pub fn post(&self) -> Option<&T> { match *self { Diff::Born(ref x) | Diff::Changed(_, ref x) => Some(x), _ => None } }

	/// Determine whether there was a change or not.
	pub fn is_same(&self) -> bool { match *self { Diff::Same => true, _ => false }}
}

#[derive(PartialEq,Eq,Clone,Copy)]
/// Boolean type for clean/dirty status.
pub enum Filth {
	/// Data has not been changed.
	Clean,
	/// Data has been changed.
	Dirty,
}
