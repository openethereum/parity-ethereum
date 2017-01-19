use std::{env, io};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use ::regex::Regex;

struct DocScanner {
	root: PathBuf
}

impl DocScanner {
	pub fn new(dirs: &[&str]) -> Self {
		let mut root: PathBuf = env::var_os("CARGO_MANIFEST_DIR").expect("env var must be set by cargo; qed").into();

		for dir in dirs {
			root.push(dir);
		}

		// This will panic if necessary, but the panic happens on compile time and on hardcoded input,
		// so it's actually desirable.
		assert!(root.is_dir(), format!("{} is not a valid directory", root.to_string_lossy()));

		DocScanner {
			root: root
		}
	}

	pub fn read_all_files(&self) -> String {
		let iter = self.root.read_dir().expect("already verified it's a dir; qed")
						.filter(Result::is_ok)
						.map(|e| e.expect("errors are filtered out; qed").path())
						.filter(|p| p.is_file());

		let mut dummy = String::new();

		lazy_static! {
			static ref RE: Regex = Regex::new(r"(?x)
				(?:\s*///[^\n]*)+      # doc comment lines
				[\n\s]*                # excess whitespace
				\#\[[^\]]+\]           # attributes
				[\n\s]*                # excess whitespace
				fn\s+[a-z][a-z0-9_]*\( # function signature start
			").unwrap();
		}

		for path in iter {
			let file_name = path.file_name().expect("already filtered to be files only; qed")
								.to_str().expect("can trust file names are valid UTF-8; qed");

			let (name, ext) = file_name.split_at(file_name.len().saturating_sub(3));

			// skip non-Rust files and the main mod.rs
			if ext != ".rs" || name == "mod" {
				continue;
			}

			let source = self.read_source(&path);

			dummy.push('\n');
			dummy += name;
			dummy.push('\n');
			for _ in 0..name.chars().count() {
				dummy.push('=');
			}
			dummy += "\n\n";

			for (start, end) in RE.find_iter(&source) {
				dummy += ">>>";
				dummy += &source[start..end];
				dummy += "<<<";
			}
		}

		dummy
	}

	pub fn read_source(&self, path: &Path) -> String {
		let mut f = File::open(path).expect("iterating over existing files from `read_dir`; rustc must have access to read to compile them; so trait files can be open; qed");
		let mut buf = Vec::with_capacity(f.metadata().expect("called on an open file; qed").len() as usize);

		io::copy(&mut f, &mut buf).expect("`Write` impl on `Vec<u8>` mustn't fail; qed");

		String::from_utf8(buf).expect("can trust that our own source code is valid UTF-8; qed")
	}
}

pub fn build() {
	let scanner = DocScanner::new(&["src", "v1", "traits"]);

	scanner.read_all_files();

	let parity_accounts = scanner.read_all_files();

	let mut f = File::create("test.txt").unwrap();

	f.write_all(parity_accounts.as_bytes()).expect("Must be able to write to the dummy file; qed");

	drop(f);
}
