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


#[cfg(feature = "with-syntex")]
mod inner {
	extern crate syntex;
	extern crate quasi_codegen;

	use std::env;
	use std::path::Path;

	pub fn main() {
		let out_dir = env::var_os("OUT_DIR").unwrap();
		let mut registry = syntex::Registry::new();
		quasi_codegen::register(&mut registry);

		let src = Path::new("src/lib.rs.in");
		let dst = Path::new(&out_dir).join("lib.rs");

		registry.expand("", &src, &dst).unwrap();
	}
}

#[cfg(not(feature = "with-syntex"))]
mod inner {
	pub fn main() {}
}

fn main() {
	inner::main();
}
