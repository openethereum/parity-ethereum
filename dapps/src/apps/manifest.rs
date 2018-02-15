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

use serde_json;
pub use apps::App as Manifest;

pub const MANIFEST_FILENAME: &'static str = "manifest.json";

pub fn deserialize_manifest(manifest: String) -> Result<Manifest, String> {
	let mut manifest = serde_json::from_str::<Manifest>(&manifest).map_err(|e| format!("{:?}", e))?;
	if manifest.id.is_none() {
		return Err("App 'id' is missing.".into());
	}
	manifest.allow_js_eval = Some(manifest.allow_js_eval.unwrap_or(false));

	Ok(manifest)
}

pub fn serialize_manifest(manifest: &Manifest) -> Result<String, String> {
	serde_json::to_string_pretty(manifest).map_err(|e| format!("{:?}", e))
}
