use super::ParamType;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct EventParam {
	pub name: String,
	#[serde(rename="type")]
	pub kind: ParamType,
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
