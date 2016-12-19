// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

//! Test implementation of fetch client.

use std::io::Write;
use std::{fs, thread};
use std::path::{Path, PathBuf};
use futures::{self, Future};
use fetch::{Fetch, Error as FetchError, Response};

/// Test implementation of fetcher. Will always return the same file.
#[derive(Default)]
pub struct TestFetch;

impl Fetch for TestFetch {
	type Result = futures::BoxFuture<Response, FetchError>;
	type FileResult = futures::BoxFuture<PathBuf, FetchError>;

	fn fetch(&self, _url: &str) -> Self::Result {
		unimplemented!()
	}

	fn fetch_to_file(&self, _url: &str, path: &Path) -> Self::FileResult {
		let path = path.to_path_buf();
		let (tx, rx) = futures::oneshot();
		thread::spawn(move || {
			let mut file = fs::File::create(&path).unwrap();
			file.write_all(b"Some content").unwrap();

			tx.complete(path);
		});

		rx.map_err(|_| unimplemented!()).boxed()
	}
}
