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

use std::io;
use futures::future;
use futures_cpupool::CpuPool;
use hyper::mime::{self, Mime};
use itertools::Itertools;
use parity_dapps::{WebApp, Info};

use endpoint::{Endpoint, EndpointInfo, EndpointPath, Request, Response};
use page::{handler, PageCache};
use Embeddable;

pub struct Dapp<T: WebApp + 'static> {
	/// futures cpu pool
	pool: CpuPool,
	/// Content of the files
	app: T,
	/// Safe to be loaded in frame by other origin. (use wisely!)
	safe_to_embed_on: Embeddable,
	info: EndpointInfo,
	fallback_to_index_html: bool,
}

impl<T: WebApp + 'static> Dapp<T> {
	/// Creates new `Dapp` for builtin (compile time) Dapp.
	pub fn new(pool: CpuPool, app: T) -> Self {
		let info = app.info();
		Dapp {
			pool,
			app,
			safe_to_embed_on: None,
			info: EndpointInfo::from(info),
			fallback_to_index_html: false,
		}
	}

	/// Creates a new `Dapp` for builtin (compile time) Dapp.
	/// Instead of returning 404 this endpoint will always server index.html.
	pub fn with_fallback_to_index(pool: CpuPool, app: T) -> Self {
		let info = app.info();
		Dapp {
			pool,
			app,
			safe_to_embed_on: None,
			info: EndpointInfo::from(info),
			fallback_to_index_html: true,
		}
	}

	/// Creates new `Dapp` which can be safely used in iframe
	/// even from different origin. It might be dangerous (clickjacking).
	/// Use wisely!
	pub fn new_safe_to_embed(pool: CpuPool, app: T, address: Embeddable) -> Self {
		let info = app.info();
		Dapp {
			pool,
			app,
			safe_to_embed_on: address,
			info: EndpointInfo::from(info),
			fallback_to_index_html: false,
		}
	}
}

impl<T: WebApp> Endpoint for Dapp<T> {
	fn info(&self) -> Option<&EndpointInfo> {
		Some(&self.info)
	}

	fn respond(&self, path: EndpointPath, _req: Request) -> Response {
		trace!(target: "dapps", "Builtin file path: {:?}", path);
		let file_path = if path.has_no_params() {
			"index.html".to_owned()
		} else {
			path.app_params.into_iter().filter(|x| !x.is_empty()).join("/")
		};
		trace!(target: "dapps", "Builtin file: {:?}", file_path);

		let file = {
			let file = |path| self.app.file(path).map(|file| {
				let content_type = match file.content_type.parse() {
					Ok(mime) => mime,
					Err(_) => {
						warn!(target: "dapps", "invalid MIME type: {}", file.content_type);
						mime::TEXT_HTML
					},
				};
				BuiltinFile {
					content_type,
					content: io::Cursor::new(file.content),
				}
			});
			let res = file(&file_path);
			if self.fallback_to_index_html {
				res.or_else(|| file("index.html"))
			} else {
				res
			}
		};

		let (reader, response) = handler::PageHandler {
			file,
			cache: PageCache::Disabled,
			safe_to_embed_on: self.safe_to_embed_on.clone(),
		}.into_response();

		self.pool.spawn(reader).forget();

		Box::new(future::ok(response))
	}
}

impl From<Info> for EndpointInfo {
	fn from(info: Info) -> Self {
		EndpointInfo {
			name: info.name.into(),
			description: info.description.into(),
			author: info.author.into(),
			icon_url: info.icon_url.into(),
			local_url: info.local_url.into(),
			version: info.version.into(),
		}
	}
}


struct BuiltinFile {
	content_type: Mime,
	content: io::Cursor<&'static [u8]>,
}

impl handler::DappFile for BuiltinFile {
	type Reader = io::Cursor<&'static [u8]>;

	fn content_type(&self) -> &Mime {
		&self.content_type
	}

	fn into_reader(self) -> Self::Reader {
		self.content
	}
}
