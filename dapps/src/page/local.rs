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

use mime_guess;
use std::io::{Seek, Read, SeekFrom};
use std::fs;
use std::path::{Path, PathBuf};
use futures::future;
use page::handler::{self, PageCache};
use endpoint::{Endpoint, EndpointInfo, EndpointPath, Request, Response};
use hyper::mime::Mime;
use Embeddable;

#[derive(Debug, Clone)]
pub struct LocalPageEndpoint {
	path: PathBuf,
	mime: Option<Mime>,
	info: Option<EndpointInfo>,
	cache: PageCache,
	embeddable_on: Embeddable,
}

impl LocalPageEndpoint {
	pub fn new(path: PathBuf, info: EndpointInfo, cache: PageCache, embeddable_on: Embeddable) -> Self {
		LocalPageEndpoint {
			path: path,
			mime: None,
			info: Some(info),
			cache: cache,
			embeddable_on: embeddable_on,
		}
	}

	pub fn single_file(path: PathBuf, mime: Mime, cache: PageCache) -> Self {
		LocalPageEndpoint {
			path: path,
			mime: Some(mime),
			info: None,
			cache: cache,
			embeddable_on: None,
		}
	}

	pub fn path(&self) -> PathBuf {
		self.path.clone()
	}

	fn page_handler_with_mime(&self, path: EndpointPath, mime: &Mime) -> handler::PageHandler<LocalFile> {
		let app = LocalSingleFile { path: &self.path, mime: format!("{}", mime) };
		handler::PageHandler {
			file: handler::PageHandler::file(app, &None, path),
			cache: self.cache,
			safe_to_embed_on: self.embeddable_on.clone(),
		}
	}

	fn page_handler(&self, path: EndpointPath) -> handler::PageHandler<LocalFile> {
		let app = LocalDapp { path: &self.path };
		handler::PageHandler {
			file: handler::PageHandler::file(app, &None, path),
			cache: self.cache,
			safe_to_embed_on: self.embeddable_on.clone(),
		}
	}

	pub fn to_response(&self, path: EndpointPath) -> Response {
		Box::new(future::ok(if let Some(ref mime) = self.mime {
			self.page_handler_with_mime(path, mime).into()
		} else {
			self.page_handler(path).into()
		}))
	}
}

impl Endpoint for LocalPageEndpoint {
	fn info(&self) -> Option<&EndpointInfo> {
		self.info.as_ref()
	}

	fn respond(&self, path: EndpointPath, _req: Request) -> Response {
		self.to_response(path)
	}
}

struct LocalSingleFile<'a> {
	path: &'a Path,
	mime: String,
}

impl<'a> handler::Dapp for LocalSingleFile<'a> {
	type DappFile = LocalFile;

	fn file(&self, _path: &str) -> Option<Self::DappFile> {
		LocalFile::from_path(self.path, Some(&self.mime))
	}
}

struct LocalDapp<'a> {
	path: &'a Path,
}

impl<'a> handler::Dapp for LocalDapp<'a> {
	type DappFile = LocalFile;

	fn file(&self, file_path: &str) -> Option<Self::DappFile> {
		let mut path = self.path.to_owned();
		for part in file_path.split('/') {
			path.push(part);
		}
		LocalFile::from_path(&path, None)
	}
}

struct LocalFile {
	content_type: String,
	buffer: [u8; 4096],
	file: fs::File,
	len: u64,
	pos: u64,
}

impl LocalFile {
	fn from_path<P: AsRef<Path>>(path: P, mime: Option<&str>) -> Option<Self> {
		// Check if file exists
		fs::File::open(&path).ok().map(|file| {
			let content_type = mime.map(|mime| mime.to_owned())
				.unwrap_or_else(|| mime_guess::guess_mime_type(path).to_string());
			let len = file.metadata().ok().map_or(0, |meta| meta.len());
			LocalFile {
				content_type: content_type,
				buffer: [0; 4096],
				file: file,
				pos: 0,
				len: len,
			}
		})
	}
}

impl handler::DappFile for LocalFile {
	fn content_type(&self) -> &str {
		&self.content_type
	}

	fn is_drained(&self) -> bool {
		self.pos == self.len
	}

	fn next_chunk(&mut self) -> &[u8] {
		let _ = self.file.seek(SeekFrom::Start(self.pos));
		if let Ok(n) = self.file.read(&mut self.buffer) {
			&self.buffer[0..n]
		} else {
			&self.buffer[0..0]
		}
	}

	fn bytes_written(&mut self, bytes: usize) {
		self.pos += bytes as u64;
	}
}
