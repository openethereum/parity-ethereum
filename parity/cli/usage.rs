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

macro_rules! usage {
	(
		{
			$(
				$field_a:ident : $typ_a:ty,
			)*
		}
		{
			$(
				$field:ident : $typ:ty = $default:expr, or $from_config:expr,
			)*
		}
	) => {
		use util::version;
		use docopt::{Docopt, Error as DocoptError};

		#[derive(Debug, PartialEq)]
		pub struct Args {
			$(
				pub $field: $typ,
			)*
			$(
				pub $field_a: $typ_a,
			)*
		}

		impl Default for Args {
			fn default() -> Self {
				Args {
					$(
						$field: $default.into(),
					)*
					$(
						$field_a: Default::default(),
					)*
				}
			}
		}

		#[derive(Default, Debug, PartialEq, RustcDecodable)]
		struct RawArgs {
			$(
				$field_a: $typ_a,
			)*
			$(
				$field: Option<$typ>,
			)*
		}

		impl Args {

			pub fn parse<S: AsRef<str>>(command: &[S]) -> Result<Self, DocoptError> {
				Ok(try!(RawArgs::parse(command)).into_args(Default::default()))
			}

			fn parse_with_config<S: AsRef<str>>(command: &[S], config: Config) -> Result<Self, DocoptError> {
				Ok(try!(RawArgs::parse(command)).into_args(config))
			}

			pub fn print_version() -> String {
				format!(include_str!("./version.txt"), version())
			}
		}

		impl RawArgs {
			fn into_args(self, config: Config) -> Args {
				let mut args = Args::default();
				$(
					args.$field_a = self.$field_a;
				)*
				$(
					args.$field = self.$field.or_else(|| $from_config(&config)).unwrap_or_else(|| $default.into());
				)*
				args
			}

			pub fn parse<S: AsRef<str>>(command: &[S]) -> Result<Self, DocoptError> {
				Docopt::new(Self::usage()).and_then(|d| d.argv(command).decode())
			}

			fn usage() -> String {
				format!(
					include_str!("./usage.txt"),
					$(
						$field={ let v: $typ = $default.into(); v },
						// Uncomment this to debug
						// "named argument never used" error
						// $field = $default,
					)*
				)
			}
		}
	};
}
