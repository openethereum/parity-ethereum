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

use mime_guess;
use std::{fs, fmt};
use std::path::{Path, PathBuf};
use futures::{future};
use futures_cpupool::CpuPool;
use page::handler::{self, PageCache};
use endpoint::{Endpoint, EndpointInfo, EndpointPath, Request, Response};
use hyper::mime::Mime;
use Embeddable;

#[derive(Clone)]
pub struct Dapp {
	pool: CpuPool,
	path: PathBuf,
	mime: Option<Mime>,
	info: Option<EndpointInfo>,
	cache: PageCache,
	embeddable_on: Embeddable,
}

impl fmt::Debug for Dapp {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt.debug_struct("Dapp")
			.field("path", &self.path)
			.field("mime", &self.mime)
			.field("info", &self.info)
			.field("cache", &self.cache)
			.field("embeddable_on", &self.embeddable_on)
			.finish()
	}
}

impl Dapp {
	pub fn new(pool: CpuPool, path: PathBuf, info: EndpointInfo, cache: PageCache, embeddable_on: Embeddable) -> Self {
		Dapp {
			pool,
			path,
			mime: None,
			info: Some(info),
			cache,
			embeddable_on,
		}
	}

	pub fn single_file(pool: CpuPool, path: PathBuf, mime: Mime, cache: PageCache) -> Self {
		Dapp {
			pool,
			path,
			mime: Some(mime),
			info: None,
			cache,
			embeddable_on: None,
		}
	}

	pub fn path(&self) -> PathBuf {
		self.path.clone()
	}

	fn get_file(&self, path: &EndpointPath) -> Option<LocalFile> {
		if let Some(ref mime) = self.mime {
			return LocalFile::from_path(&self.path, mime.to_owned());
		}

		let mut file_path = self.path.to_owned();

		if path.has_no_params() {
			file_path.push("index.html");
		} else {
			for part in &path.app_params {
				file_path.push(part);
			}
		}

		let mime = mime_guess::guess_mime_type(&file_path);
		LocalFile::from_path(&file_path, mime)
	}

	pub fn to_response(&self, path: &EndpointPath) -> Response {
		let (reader, response) = handler::PageHandler {
			file: self.get_file(path),
			cache: self.cache,
			safe_to_embed_on: self.embeddable_on.clone(),
			allow_js_eval: self.info.as_ref().and_then(|x| x.allow_js_eval).unwrap_or(false),
		}.into_response();

		self.pool.spawn(reader).forget();

		Box::new(future::ok(response))
	}
}

impl Endpoint for Dapp {
	fn info(&self) -> Option<&EndpointInfo> {
		self.info.as_ref()
	}

	fn respond(&self, path: EndpointPath, _req: Request) -> Response {
		self.to_response(&path)
	}
}

struct LocalFile {
	content_type: Mime,
	file: fs::File,
}

impl LocalFile {
	fn from_path<P: AsRef<Path>>(path: P, content_type: Mime) -> Option<Self> {
		trace!(target: "dapps", "Local file: {:?}", path.as_ref());
		// Check if file exists
		fs::File::open(&path).ok().map(|file| {
			LocalFile {
				content_type,
				file,
			}
		})
	}
}

impl handler::DappFile for LocalFile {
	type Reader = fs::File;

	fn content_type(&self) -> &Mime {
		&self.content_type
	}

	fn into_reader(self) -> Self::Reader {
		self.file
	}
}
