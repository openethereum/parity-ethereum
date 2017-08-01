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
