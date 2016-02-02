//! Utils common types and macros global reexport.

pub use standard::*;
pub use from_json::*;
pub use error::*;
pub use hash::*;
pub use uint::*;
pub use bytes::*;
pub use vector::*;
pub use sha3::*;

#[macro_export]
macro_rules! map {
	( $( $x:expr => $y:expr ),* ) => {
		vec![ $( ($x, $y) ),* ].into_iter().collect::<BTreeMap<_, _>>()
	}
}

#[macro_export]
macro_rules! mapx {
	( $( $x:expr => $y:expr ),* ) => {
		vec![ $( ( From::from($x), From::from($y) ) ),* ].into_iter().collect::<BTreeMap<_, _>>()
	}
}

#[macro_export]
macro_rules! x {
	( $x:expr ) => {
		From::from($x)
	}
}

#[macro_export]
macro_rules! xx {
	( $x:expr ) => {
		From::from(From::from($x))
	}
}

#[macro_export]
macro_rules! flush {
	($($arg:tt)*) => ($crate::flush(format!("{}", format_args!($($arg)*))));
}

#[macro_export]
macro_rules! flushln {
	($fmt:expr) => (flush!(concat!($fmt, "\n")));
	($fmt:expr, $($arg:tt)*) => (flush!(concat!($fmt, "\n"), $($arg)*));
}

/// TODO [Gav Wood] Please document me
pub fn flush(s: String) {
	::std::io::stdout().write(s.as_bytes()).unwrap();
	::std::io::stdout().flush().unwrap();
}

#[test]
fn test_flush() {
	flushln!("hello_world {:?}", 1);
}
