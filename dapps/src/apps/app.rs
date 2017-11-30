// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use endpoint::EndpointInfo;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct App {
	pub id: String,
	pub name: String,
	pub description: String,
	pub version: String,
	pub author: String,
	#[serde(rename="iconUrl")]
	pub icon_url: String,
	#[serde(rename="localUrl")]
	pub local_url: Option<String>,
}

impl App {
	/// Creates `App` instance from `EndpointInfo` and `id`.
	pub fn from_info(id: &str, info: &EndpointInfo) -> Self {
		App {
			id: id.to_owned(),
			name: info.name.to_owned(),
			description: info.description.to_owned(),
			version: info.version.to_owned(),
			author: info.author.to_owned(),
			icon_url: info.icon_url.to_owned(),
			local_url: info.local_url.to_owned(),
		}
	}
}

impl Into<EndpointInfo> for App {
	fn into(self) -> EndpointInfo {
		EndpointInfo {
			name: self.name,
			description: self.description,
			version: self.version,
			author: self.author,
			icon_url: self.icon_url,
			local_url: self.local_url,
		}
	}
}
