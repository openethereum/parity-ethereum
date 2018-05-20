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

extern crate serde_json;
extern crate serde_ignored;
extern crate ethjson;

use std::collections::BTreeSet;
use std::{fs, env, process};
use ethjson::spec::Spec;

fn quit(s: &str) -> ! {
	println!("{}", s);
	process::exit(1);
}

fn main() {
	let mut args = env::args();
	if args.len() != 2 {
		quit("You need to specify chainspec.json\n\
		\n\
		./chainspec <chainspec.json>");
	}

	let path = args.nth(1).expect("args.len() == 2; qed");
	let file = match fs::File::open(&path) {
		Ok(file) => file,
		Err(_) => quit(&format!("{} could not be opened", path)),
	};

	let mut unused = BTreeSet::new();
	let mut deserializer = serde_json::Deserializer::from_reader(file);

	let spec: Result<Spec, _> = serde_ignored::deserialize(&mut deserializer, |field| {
		unused.insert(field.to_string());
	});

	if let Err(err) = spec {
		quit(&format!("{} {}", path, err.to_string()));
	}

	if !unused.is_empty() {
		let err = unused.into_iter()
			.map(|field| format!("{} unexpected field `{}`", path, field))
			.collect::<Vec<_>>()
			.join("\n");
		quit(&err);
	}

	println!("{} is valid", path);
}
