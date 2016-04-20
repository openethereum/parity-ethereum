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

extern crate syntex;
extern crate ethcore_ipc_codegen as codegen;
extern crate serde_codegen;

use std::env;
use std::path::Path;

pub fn main() {
	let out_dir = env::var_os("OUT_DIR").unwrap();

	// ipc pass
	{
		let src = Path::new("nested.rs.in");
		let dst = Path::new(&out_dir).join("nested_ipc.rs");
		let mut registry = syntex::Registry::new();
		codegen::register(&mut registry);
		registry.expand("", &src, &dst).unwrap();
	}

	// serde pass
	{
		let src = Path::new(&out_dir).join("nested_ipc.rs");
		let dst = Path::new(&out_dir).join("nested_cg.rs");
		let mut registry = syntex::Registry::new();
		serde_codegen::register(&mut registry);
		registry.expand("", &src, &dst).unwrap();
	}

	// ipc pass
	{
		let src = Path::new("service.rs.in");
		let dst = Path::new(&out_dir).join("service_ipc.rs");
		let mut registry = syntex::Registry::new();
		codegen::register(&mut registry);
		registry.expand("", &src, &dst).unwrap();
	}

	// serde pass
	{
		let src = Path::new(&out_dir).join("service_ipc.rs");
		let dst = Path::new(&out_dir).join("service_cg.rs");
		let mut registry = syntex::Registry::new();
		serde_codegen::register(&mut registry);
		registry.expand("", &src, &dst).unwrap();
	}


	// ipc pass
	{
		let src = Path::new("binary.rs.in");
		let dst = Path::new(&out_dir).join("binary.rs");
		let mut registry = syntex::Registry::new();
		codegen::register(&mut registry);
		registry.expand("", &src, &dst).unwrap();
	}
}
