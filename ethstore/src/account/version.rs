use json;

#[derive(Debug, PartialEq, Clone)]
pub enum Version {
	V3,
}

impl From<json::Version> for Version {
	fn from(json: json::Version) -> Self {
		match json {
			json::Version::V3 => Version::V3,
		}
	}
}

impl Into<json::Version> for Version {
	fn into(self) -> json::Version {
		match self {
			Version::V3 => json::Version::V3,
		}
	}
}
