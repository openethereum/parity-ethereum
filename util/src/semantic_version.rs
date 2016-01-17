/// A version value with strict meaning. Use `to_u32` to convert to a simple integer.
/// 
/// # Example
/// ```
/// extern crate ethcore_util as util;
/// use util::semantic_version::*;
/// 
/// fn main() {
///   assert_eq!(SemanticVersion::new(1, 2, 3).as_u32(), 0x010203);
/// }
/// ```
pub struct SemanticVersion {
	/// Major version - API/feature removals & breaking changes.
	pub major: u8,
	/// Minor version - API/feature additions.
	pub minor: u8,
	/// Tiny version - bug fixes.
	pub tiny: u8,
}

impl SemanticVersion {
	/// Create a new object.
	pub fn new(major: u8, minor: u8, tiny: u8) -> SemanticVersion { SemanticVersion{major: major, minor: minor, tiny: tiny} }

	/// Convert to a `u32` representation.
	pub fn as_u32(&self) -> u32 { ((self.major as u32) << 16) + ((self.minor as u32) << 8) + self.tiny as u32 }
}

// TODO: implement Eq, Comparison and Debug/Display for SemanticVersion.
