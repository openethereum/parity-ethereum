use serde_json::Error as SerdeError;

/// Spec deserialization errors.
#[derive(Debug)]
pub enum Error {
	/// Returned when spec deserialization from json fails.
	Serde(SerdeError),
}

impl From<SerdeError> for Error {
	fn from(err: SerdeError) -> Self {
		Error::Serde(err)
	}
}
