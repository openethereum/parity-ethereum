//! Event param specification.

use super::ParamType;

/// Event param specification.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct EventParam {
	/// Param name.
	pub name: String,
	/// Param type.
	#[serde(rename="type")]
	pub kind: ParamType,
	/// Indexed flag. If true, param is used to build block bloom.
	pub indexed: bool,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use super::EventParam;
	use spec::ParamType;

	#[test]
	fn event_param_deserialization() {
		let s = r#"{
			"name": "foo",
			"type": "address",
			"indexed": true
		}"#;
		
		let deserialized: EventParam = serde_json::from_str(s).unwrap();

		assert_eq!(deserialized, EventParam {
			name: "foo".to_owned(),
			kind: ParamType::Address,
			indexed: true,
		});
	}
}
