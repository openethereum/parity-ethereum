// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
use std::path::PathBuf;
use page::handler;
use endpoint::{Endpoint, EndpointInfo, EndpointPath, Handler};

pub struct LocalPageEndpoint {
	path: PathBuf,
	info: EndpointInfo,
}

impl LocalPageEndpoint {
	pub fn new(path: PathBuf, info: EndpointInfo) -> Self {
		LocalPageEndpoint {
			path: path,
			info: info,
		}
	}
}

impl Endpoint for LocalPageEndpoint {
	fn info(&self) -> Option<&EndpointInfo> {
		Some(&self.info)
	}

	fn to_handler(&self, path: EndpointPath) -> Box<Handler> {
		Box::new(handler::PageHandler {
			app: LocalDapp::new(self.path.clone()),
			prefix: None,
			path: path,
			file: None,
			safe_to_embed: false,
		})
	}
}

struct LocalDapp {
	path: PathBuf,
}

impl LocalDapp {
	fn new(path: PathBuf) -> Self {
		LocalDapp {
			path: path
		}
	}
}

impl handler::Dapp for LocalDapp {
	type DappFile = LocalFile;

	fn file(&self, file_path: &str) -> Option<Self::DappFile> {
		let mut path = self.path.clone();
		for part in file_path.split('/') {
			path.push(part);
		}
		// Check if file exists
		fs::File::open(path.clone()).ok().map(|file| {
			let content_type = mime_guess::guess_mime_type(path);
			let len = file.metadata().ok().map_or(0, |meta| meta.len());
			LocalFile {
				content_type: content_type.to_string(),
				buffer: [0; 4096],
				file: file,
				pos: 0,
				len: len,
			}
		})
	}
}

struct LocalFile {
	content_type: String,
	buffer: [u8; 4096],
	file: fs::File,
	len: u64,
	pos: u64,
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
