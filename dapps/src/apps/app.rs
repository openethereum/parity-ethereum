// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct App {
	pub id: Option<String>,
	pub name: String,
	pub description: String,
	pub version: String,
	pub author: String,
	#[serde(rename="iconUrl")]
	pub icon_url: String,
	#[serde(rename="localUrl")]
	pub local_url: Option<String>,
	#[serde(rename="allowJsEval")]
	pub allow_js_eval: Option<bool>,
}

impl App {
	pub fn with_id(&self, id: &str) -> Self {
		let mut app = self.clone();
		app.id = Some(id.into());
		app
	}
}
