//! Helper macros for running the `JSON tests`

/// Declares a test:
///
/// declare_test!(test_name, "path/to/folder/with/tests");
///
/// Declares a test but skip the named test files inside the folder (no extension):
///
/// declare_test!(skip => ["a-test-file", "other-test-file"], test_name, "path/to/folder/with/tests");
///
/// NOTE: a skipped test is considered a passing test as far as `cargo test` is concerned. Normally
/// one test corresponds to a folder full of test files, each of which may contain many tests.
#[macro_export]
macro_rules! declare_test {
	(skip => $arr: expr, $id: ident, $name: expr) => {
		#[cfg(test)]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name, $arr);
		}
	};
	(ignore => $id: ident, $name: expr) => {
		#[cfg(test)]
		#[ignore]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name, []);
		}
	};
	(heavy => $id: ident, $name: expr) => {
		#[cfg(test)]
		#[cfg(feature = "test-heavy")]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name, []);
		}
	};
	($id: ident, $name: expr) => {
		#[cfg(test)]
		#[test]
		#[allow(non_snake_case)]
		fn $id() {
			test!($name, []);
		}
	}
}

#[cfg(test)]
macro_rules! test {
	($name: expr, $skip: expr) => {
		$crate::json_tests::test_common::run_test_path(
			std::path::Path::new(concat!("res/ethereum/tests/", $name)),
			&$skip,
			do_json_test,
			&mut |_, _| ()
		);
	}
}

/// Similar to `print!` but flushes stdout in order to ensure the output is emitted immediately.
#[macro_export]
macro_rules! flushed_write {
	($arg:expr) => ($crate::json_tests::macros::write_and_flush($arg.into()));
	($($arg:tt)*) => ($crate::json_tests::macros::write_and_flush(format!("{}", format_args!($($arg)*))));
}

/// Similar to `println!` but flushes stdout in order to ensure the output is emitted immediately.
#[macro_export]
macro_rules! flushed_writeln {
	($fmt:expr) => (flushed_write!(concat!($fmt, "\n")));
	($fmt:expr, $($arg:tt)*) => (flushed_write!(concat!($fmt, "\n"), $($arg)*));
}

/// Write to stdout and flush (ignores errors)
#[doc(hidden)]
pub fn write_and_flush(s: String) {
	if let Err(err) = std::io::Write::write_all(&mut std::io::stdout(), s.as_bytes()) {
		error!(target: "json_tests", "io::Write::write_all to stdout failed because of: {:?}", err);
	}
	if let Err(err) = std::io::Write::flush(&mut std::io::stdout()) {
		error!(target: "json_tests", "io::Write::flush stdout failed because of: {:?}", err);
	}
}
