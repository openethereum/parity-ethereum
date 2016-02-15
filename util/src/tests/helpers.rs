use common::*;
use std::path::PathBuf;
use std::fs::{remove_dir_all};
use std::env;

pub struct RandomTempPath {
	path: PathBuf
}

impl RandomTempPath {
	pub fn create_dir() -> RandomTempPath {
		let mut dir = env::temp_dir();
		dir.push(H32::random().hex());
		fs::create_dir_all(dir.as_path()).unwrap();
		RandomTempPath {
			path: dir.clone()
		}
	}

	pub fn as_path(&self) -> &PathBuf {
		&self.path
	}
}

impl Drop for RandomTempPath {
	fn drop(&mut self) {
		if let Err(e) = remove_dir_all(self.as_path()) {
			panic!("failed to remove temp directory, probably something failed to destroyed ({})", e);
		}
	}
}
