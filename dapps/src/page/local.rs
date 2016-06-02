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

use page::handler;
use std::path::PathBuf;
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

	fn file(&self, path: &str) -> Option<Self::DappFile> {
		unimplemented!()
	}
}

struct LocalFile {

}

impl handler::DappFile for LocalFile {
	fn content_type(&self) -> &str {
		"application/octetstream"
	}

	fn is_drained(&self) -> bool {
		false
	}

	fn next_chunk(&self) -> &[u8] {
		unimplemented!()
	}

	fn bytes_written(&mut self, bytes: usize) {
		unimplemented!()
	}
}
