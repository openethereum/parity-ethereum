use serde_json::Error as SerdeError;

#[derive(Debug)]
pub enum Error {
	Serde(SerdeError),
}

impl From<SerdeError> for Error {
	fn from(err: SerdeError) -> Self {
		Error::Serde(err)
	}
}
