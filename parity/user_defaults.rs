// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::de::from_reader;
use serde_json::ser::to_string;
use journaldb::Algorithm;
use ethcore::client::{Mode as ClientMode};

#[derive(Clone)]
pub struct Seconds(Duration);

impl Seconds {
	pub fn value(&self) -> u64 {
		self.0.as_secs()
	}
}

impl From<u64> for Seconds {
	fn from(s: u64) -> Seconds {
		Seconds(Duration::from_secs(s))
	}
}

impl From<Duration> for Seconds {
	fn from(d: Duration) -> Seconds {
		Seconds(d)
	}
}

impl Into<Duration> for Seconds {
	fn into(self) -> Duration {
		self.0
	}
}

impl Serialize for Seconds {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_u64(self.value())
	}
}

impl<'de> Deserialize<'de> for Seconds {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let secs = u64::deserialize(deserializer)?;
		Ok(Seconds::from(secs))
	}
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "mode")]
pub enum Mode {
	Active,
	Passive {
		#[serde(rename = "mode.timeout")]
		timeout: Seconds,
		#[serde(rename = "mode.alarm")]
		alarm: Seconds,
	},
	Dark {
		#[serde(rename = "mode.timeout")]
		timeout: Seconds,
	},
	Offline,
}

impl Into<ClientMode> for Mode {
	fn into(self) -> ClientMode {
		match self {
			Mode::Active => ClientMode::Active,
			Mode::Passive { timeout, alarm } => ClientMode::Passive(timeout.into(), alarm.into()),
			Mode::Dark { timeout } => ClientMode::Dark(timeout.into()),
			Mode::Offline => ClientMode::Off,
		}
	}
}

impl From<ClientMode> for Mode {
	fn from(mode: ClientMode) -> Mode {
		match mode {
			ClientMode::Active => Mode::Active,
			ClientMode::Passive(timeout, alarm) => Mode::Passive { timeout: timeout.into(), alarm: alarm.into() },
			ClientMode::Dark(timeout) => Mode::Dark { timeout: timeout.into() },
			ClientMode::Off => Mode::Offline,
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct UserDefaults {
	pub is_first_launch: bool,
	#[serde(with = "algorithm_serde")]
	pub pruning: Algorithm,
	pub tracing: bool,
	pub fat_db: bool,
	#[serde(flatten)]
	mode: Mode,
}

impl UserDefaults {
	pub fn mode(&self) -> ClientMode {
		self.mode.clone().into()
	}

	pub fn set_mode(&mut self, mode: ClientMode) {
		self.mode = mode.into();
	}
}

mod algorithm_serde {
	use serde::{Deserialize, Deserializer, Serialize, Serializer};
	use serde::de::Error;
	use journaldb::Algorithm;

	pub fn serialize<S>(algorithm: &Algorithm, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		algorithm.as_str().serialize(serializer)
	}

	pub fn deserialize<'de, D>(deserializer: D) -> Result<Algorithm, D::Error>
	where D: Deserializer<'de> {
		let pruning = String::deserialize(deserializer)?;
		pruning.parse().map_err(|_| Error::custom("invalid pruning method"))
	}
}

impl Default for UserDefaults {
	fn default() -> Self {
		UserDefaults {
			is_first_launch: true,
			pruning: Algorithm::OverlayRecent,
			tracing: false,
			fat_db: false,
			mode: Mode::Active,
		}
	}
}

impl UserDefaults {
	pub fn load<P>(path: P) -> Result<Self, String> where P: AsRef<Path> {
		match File::open(path) {
			Ok(file) => match from_reader(file) {
				Ok(defaults) => Ok(defaults),
				Err(e) => {
					warn!("Error loading user defaults file: {:?}", e);
					Ok(UserDefaults::default())
				},
			},
			_ => Ok(UserDefaults::default()),
		}
	}

	pub fn save<P>(&self, path: P) -> Result<(), String> where P: AsRef<Path> {
		let mut file: File = File::create(path).map_err(|_| "Cannot create user defaults file".to_owned())?;
		file.write_all(to_string(&self).unwrap().as_bytes()).map_err(|_| "Failed to save user defaults".to_owned())
	}
}
