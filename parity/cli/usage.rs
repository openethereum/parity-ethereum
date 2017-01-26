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

macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

macro_rules! otry {
	($e: expr) => (
		match $e {
			Some(ref v) => v,
			None => {
				return None;
			}
		}
	)
}
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
		{
			$(
				$field_s:ident : $typ_s:ty, display $default_s:expr, or $from_config_s:expr,
			)*
		}
	) => {
		use toml;
		use std::{fs, io, process};
		use std::io::{Read, Write};
		use util::version;
		use docopt::{Docopt, Error as DocoptError};
		use helpers::replace_home;
		use rustc_serialize;

		#[derive(Debug)]
		pub enum ArgsError {
			Docopt(DocoptError),
			Parsing(Vec<toml::ParserError>),
			Decode(toml::DecodeError),
			Config(String, io::Error),
			UnknownFields(String),
		}

		impl ArgsError {
			pub fn exit(self) -> ! {
				match self {
					ArgsError::Docopt(e) => e.exit(),
					ArgsError::Parsing(errors) => {
						println_stderr!("There is an error in config file.");
						for e in &errors {
							println_stderr!("{}", e);
						}
						process::exit(2)
					},
					ArgsError::Decode(e) => {
						println_stderr!("You might have supplied invalid parameters in config file.");
						println_stderr!("{}", e);
						process::exit(2)
					},
					ArgsError::Config(path, e) => {
						println_stderr!("There was an error reading your config file at: {}", path);
						println_stderr!("{}", e);
						process::exit(2)
					},
					ArgsError::UnknownFields(fields) => {
						println_stderr!("You have some extra fields in your config file:");
						println_stderr!("{}", fields);
						process::exit(2)
					}
				}
			}
		}

		impl From<DocoptError> for ArgsError {
			fn from(e: DocoptError) -> Self { ArgsError::Docopt(e) }
		}

		impl From<toml::DecodeError> for ArgsError {
			fn from(e: toml::DecodeError) -> Self { ArgsError::Decode(e) }
		}

		#[derive(Debug, PartialEq)]
		pub struct Args {
			$(
				pub $field_a: $typ_a,
			)*

			$(
				pub $field: $typ,
			)*

			$(
				pub $field_s: $typ_s,
			)*
		}

		impl Default for Args {
			fn default() -> Self {
				Args {
					$(
						$field_a: Default::default(),
					)*

					$(
						$field: $default.into(),
					)*

					$(
						$field_s: Default::default(),
					)*
				}
			}
		}

		#[derive(Default, Debug, PartialEq, Clone, RustcDecodable)]
		struct RawArgs {
			$(
				$field_a: $typ_a,
			)*
			$(
				$field: Option<$typ>,
			)*
			$(
				$field_s: Option<$typ_s>,
			)*
		}

		impl Args {

			pub fn parse<S: AsRef<str>>(command: &[S]) -> Result<Self, ArgsError> {
				let raw_args = RawArgs::parse(command)?;

				// Skip loading config file if no_config flag is specified
				if raw_args.flag_no_config {
					return Ok(raw_args.into_args(Config::default()));
				}

				let config_file = raw_args.flag_config.clone().unwrap_or_else(|| raw_args.clone().into_args(Config::default()).flag_config);
				let config_file = replace_home(&::dir::default_data_path(), &config_file);
				let config = match (fs::File::open(&config_file), raw_args.flag_config.is_some()) {
					// Load config file
					(Ok(mut file), _) => {
						println_stderr!("Loading config file from {}", &config_file);
						let mut config = String::new();
						file.read_to_string(&mut config).map_err(|e| ArgsError::Config(config_file, e))?;
						Self::parse_config(&config)?
					},
					// Don't display error in case default config cannot be loaded.
					(Err(_), false) => Config::default(),
					// Config set from CLI (fail with error)
					(Err(e), true) => {
						return Err(ArgsError::Config(config_file, e));
					},
				};

				Ok(raw_args.into_args(config))
			}

			#[cfg(test)]
			pub fn parse_without_config<S: AsRef<str>>(command: &[S]) -> Result<Self, ArgsError> {
				Self::parse_with_config(command, Config::default())
			}

			#[cfg(test)]
			fn parse_with_config<S: AsRef<str>>(command: &[S], config: Config) -> Result<Self, ArgsError> {
				RawArgs::parse(command).map(|raw| raw.into_args(config)).map_err(ArgsError::Docopt)
			}

			fn parse_config(config: &str) -> Result<Config, ArgsError> {
				let mut value_parser = toml::Parser::new(&config);
				match value_parser.parse() {
					Some(value) => {
						let mut decoder = toml::Decoder::new(toml::Value::Table(value));
						let result = rustc_serialize::Decodable::decode(&mut decoder);

						match (result, decoder.toml) {
							(Err(e), _) => Err(e.into()),
							(_, Some(toml)) => Err(ArgsError::UnknownFields(toml::encode_str(&toml))),
							(Ok(config), None) => Ok(config),
						}
					},
					None => Err(ArgsError::Parsing(value_parser.errors)),
				}
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
				$(
					args.$field_s = self.$field_s.or_else(|| $from_config_s(&config)).unwrap_or(None);
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
					$(
						$field_s = $default_s,
					)*
				)
			}
		}
	};
}
