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

//! Test implementation of fetch client.

use std::io::Write;
use std::{env, fs, thread};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use fetch::{Fetch, FetchError, FetchResult};

/// Test implementation of fetcher. Will always return the same file.
#[derive(Default)]
pub struct TestFetch;

impl Fetch for TestFetch {
	fn request_async(&mut self, _url: &str, _abort: Arc<AtomicBool>, on_done: Box<Fn(FetchResult) + Send>) -> Result<(), FetchError> {
		thread::spawn(move || {
			let mut path = env::temp_dir();
			path.push(Self::random_filename());

			let mut file = fs::File::create(&path).unwrap();
			file.write_all(b"Some content").unwrap();

			on_done(Ok(path));
		});
		Ok(())
	}
}


