// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

extern crate tempdir;
use std::process::Command;
use tempdir::TempDir;
use std::fs::File;
use std::io::Write;

fn run(args: &[&str]) -> String {
	let output = Command::new("cargo")
		.args(&["run", "--"])
		.args(args)
		.output()
		.unwrap();
	assert!(output.status.success());
	String::from_utf8(output.stdout).unwrap()
}

#[test]
fn cli_cmd() {
	Command::new("cargo")
		.arg("build")
		.output()
		.unwrap();

	let dir = TempDir::new("test-vault").unwrap();

	let mut passwd = File::create(dir.path().join("test-password")).unwrap();
	writeln!(passwd, "password").unwrap();

	let mut passwd2 = File::create(dir.path().join("test-vault-addr")).unwrap();
	writeln!(passwd2, "password2").unwrap();

	let test_password_buf = dir.path().join("test-password");
	let test_password: &str = test_password_buf.to_str().unwrap();
	let dir_str: &str = dir.path().to_str().unwrap();
	let test_vault_addr_buf = dir.path().join("test-vault-addr");
	let test_vault_addr = test_vault_addr_buf.to_str().unwrap();

	run(&["create-vault", "test-vault", test_password, "--dir", dir_str]);

	let output = run(&["insert", "7d29fab185a33e2cd955812397354c472d2b84615b645aa135ff539f6b0d70d5",
			    test_vault_addr,
			    "--dir", dir_str,
			    "--vault", "test-vault",
			   "--vault-pwd", test_password]);
	let address = output.trim();

	let output = run(&["list",
			   "--dir", dir_str,
			   "--vault", "test-vault",
			   "--vault-pwd", test_password]);
	assert_eq!(output, " 0: 0xa8fa5dd30a87bb9e3288d604eb74949c515ab66e\n");

	let output = run(&["sign", &address[2..],
			   test_vault_addr,
			   "7d29fab185a33e2cd955812397354c472d2b84615b645aa135ff539f6b0d70d5",
			   "--dir", dir_str,
			   "--vault", "test-vault",
			   "--vault-pwd", test_password]);
	assert_eq!(output, "0x54ab6e5cf0c5cb40043fdca5d15d611a3a94285414a076dafecc8dc9c04183f413296a3defff61092c0bb478dc9887ec01070e1275234211208fb8f4be4a9b0101\n");


	let output = run(&["public", &address[2..], test_vault_addr,
			   "--dir", dir_str,
			   "--vault", "test-vault",
			   "--vault-pwd", test_password]);
	assert_eq!(output, "0x35f222d88b80151857a2877826d940104887376a94c1cbd2c8c7c192eb701df88a18a4ecb8b05b1466c5b3706042027b5e079fe3a3683e66d822b0e047aa3418\n");
}
