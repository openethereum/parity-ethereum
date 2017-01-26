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

use page::{handler, PageCache};
use std::sync::Arc;
use endpoint::{Endpoint, EndpointInfo, EndpointPath, Handler};
use parity_dapps::{WebApp, File, Info};

pub struct PageEndpoint<T : WebApp + 'static> {
	/// Content of the files
	pub app: Arc<T>,
	/// Prefix to strip from the path (when `None` deducted from `app_id`)
	pub prefix: Option<String>,
	/// Safe to be loaded in frame by other origin. (use wisely!)
	safe_to_embed_on: Option<(String, u16)>,
	info: EndpointInfo,
}

impl<T: WebApp + 'static> PageEndpoint<T> {
	/// Creates new `PageEndpoint` for builtin (compile time) Dapp.
	pub fn new(app: T) -> Self {
		let info = app.info();
		PageEndpoint {
			app: Arc::new(app),
			prefix: None,
			safe_to_embed_on: None,
			info: EndpointInfo::from(info),
		}
	}

	/// Create new `PageEndpoint` and specify prefix that should be removed before looking for a file.
	/// It's used only for special endpoints (i.e. `/parity-utils/`)
	/// So `/parity-utils/inject.js` will be resolved to `/inject.js` is prefix is set.
	pub fn with_prefix(app: T, prefix: String) -> Self {
		let info = app.info();
		PageEndpoint {
			app: Arc::new(app),
			prefix: Some(prefix),
			safe_to_embed_on: None,
			info: EndpointInfo::from(info),
		}
	}

	/// Creates new `PageEndpoint` which can be safely used in iframe
	/// even from different origin. It might be dangerous (clickjacking).
	/// Use wisely!
	pub fn new_safe_to_embed(app: T, address: Option<(String, u16)>) -> Self {
		let info = app.info();
		PageEndpoint {
			app: Arc::new(app),
			prefix: None,
			safe_to_embed_on: address,
			info: EndpointInfo::from(info),
		}
	}
}

impl<T: WebApp> Endpoint for PageEndpoint<T> {

	fn info(&self) -> Option<&EndpointInfo> {
		Some(&self.info)
	}

	fn to_handler(&self, path: EndpointPath) -> Box<Handler> {
		Box::new(handler::PageHandler {
			app: BuiltinDapp::new(self.app.clone()),
			prefix: self.prefix.clone(),
			path: path,
			file: handler::ServedFile::new(self.safe_to_embed_on.clone()),
			cache: PageCache::Disabled,
			safe_to_embed_on: self.safe_to_embed_on.clone(),
		})
	}
}

impl From<Info> for EndpointInfo {
	fn from(info: Info) -> Self {
		EndpointInfo {
			name: info.name.into(),
			description: info.description.into(),
			author: info.author.into(),
			icon_url: info.icon_url.into(),
			version: info.version.into(),
		}
	}
}

struct BuiltinDapp<T: WebApp + 'static> {
	app: Arc<T>,
}

impl<T: WebApp + 'static> BuiltinDapp<T> {
	fn new(app: Arc<T>) -> Self {
		BuiltinDapp {
			app: app,
		}
	}
}

impl<T: WebApp + 'static> handler::Dapp for BuiltinDapp<T> {
	type DappFile = BuiltinDappFile<T>;

	fn file(&self, path: &str) -> Option<Self::DappFile> {
		self.app.file(path).map(|_| {
			BuiltinDappFile {
				app: self.app.clone(),
				path: path.into(),
				write_pos: 0,
			}
		})
	}
}

struct BuiltinDappFile<T: WebApp + 'static> {
	app: Arc<T>,
	path: String,
	write_pos: usize,
}

impl<T: WebApp + 'static> BuiltinDappFile<T> {
	fn file(&self) -> &File {
		self.app.file(&self.path).expect("Check is done when structure is created.")
	}
}

impl<T: WebApp + 'static> handler::DappFile for BuiltinDappFile<T> {
	fn content_type(&self) -> &str {
		self.file().content_type
	}

	fn is_drained(&self) -> bool {
		self.write_pos == self.file().content.len()
	}

	fn next_chunk(&mut self) -> &[u8] {
		&self.file().content[self.write_pos..]
	}

	fn bytes_written(&mut self, bytes: usize) {
		self.write_pos += bytes;
	}
}
