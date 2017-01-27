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
use page::handler::{self, PageCache, PageHandlerWaiting};
use endpoint::{Endpoint, EndpointInfo, EndpointPath, Handler};
use mime::Mime;

#[derive(Debug, Clone)]
pub struct LocalPageEndpoint {
	path: PathBuf,
	mime: Option<Mime>,
	info: Option<EndpointInfo>,
	cache: PageCache,
	embeddable_on: Option<(String, u16)>,
}

impl LocalPageEndpoint {
	pub fn new(path: PathBuf, info: EndpointInfo, cache: PageCache, embeddable_on: Option<(String, u16)>) -> Self {
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

	fn page_handler_with_mime(&self, path: EndpointPath, mime: &Mime) -> handler::PageHandler<LocalSingleFile> {
		handler::PageHandler {
			app: LocalSingleFile { path: self.path.clone(), mime: format!("{}", mime) },
			prefix: None,
			path: path,
			file: handler::ServedFile::new(None),
			safe_to_embed_on: self.embeddable_on.clone(),
			cache: self.cache,
		}
	}

	fn page_handler(&self, path: EndpointPath) -> handler::PageHandler<LocalDapp> {
		handler::PageHandler {
			app: LocalDapp { path: self.path.clone() },
			prefix: None,
			path: path,
			file: handler::ServedFile::new(None),
			safe_to_embed_on: self.embeddable_on.clone(),
			cache: self.cache,
		}
	}

	pub fn to_page_handler(&self, path: EndpointPath) -> Box<PageHandlerWaiting> {
		if let Some(ref mime) = self.mime {
			Box::new(self.page_handler_with_mime(path, mime))
		} else {
			Box::new(self.page_handler(path))
		}
	}
}

impl Endpoint for LocalPageEndpoint {
	fn info(&self) -> Option<&EndpointInfo> {
		self.info.as_ref()
	}

	fn to_handler(&self, path: EndpointPath) -> Box<Handler> {
		if let Some(ref mime) = self.mime {
			Box::new(self.page_handler_with_mime(path, mime))
		} else {
			Box::new(self.page_handler(path))
		}
	}
}

struct LocalSingleFile {
	path: PathBuf,
	mime: String,
}

impl handler::Dapp for LocalSingleFile {
	type DappFile = LocalFile;

	fn file(&self, _path: &str) -> Option<Self::DappFile> {
		LocalFile::from_path(&self.path, Some(&self.mime))
	}
}

struct LocalDapp {
	path: PathBuf,
}

impl handler::Dapp for LocalDapp {
	type DappFile = LocalFile;

	fn file(&self, file_path: &str) -> Option<Self::DappFile> {
		let mut path = self.path.clone();
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
