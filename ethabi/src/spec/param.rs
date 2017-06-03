//! Function param.

use super::ParamType;

/// Function param.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Param {
	/// Param name.
	pub name: String,
	/// Param type.
	#[serde(rename="type")]
	pub kind: ParamType,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::Param;
	use spec::ParamType;

	#[test]
	fn param_deserialization() {
		let s = r#"{
			"name": "foo",
			"type": "address"
		}"#;
		
		let deserialized: Param = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, Param {
			name: "foo".to_owned(),
			kind: ParamType::Address,
		});
	}
}
