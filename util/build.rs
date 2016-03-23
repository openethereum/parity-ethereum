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

extern crate vergen;
extern crate rustc_version;

use vergen::*;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
	vergen(OutputFns::all()).unwrap();
	let out_dir = env::var("OUT_DIR").unwrap();
	let dest_path = Path::new(&out_dir).join("rustc_version.rs");
	let mut f = File::create(&dest_path).unwrap();
	f.write_all(format!("
		/// Returns compiler version.
		pub fn rustc_version() -> &'static str {{
			\"{}\"
		}}
	", rustc_version::version()).as_bytes()).unwrap();
}
