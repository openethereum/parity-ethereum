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
	($e:expr) => (
		match $e {
			Some(ref v) => v,
			None => {
				return None;
			}
		}
	)
}

macro_rules! return_if_parse_error {
	($e:expr) => (
		match $e {
			Err(clap_error @ ClapError { kind: ClapErrorKind::ValueValidation, .. }) => {
				return Err(clap_error);
			},

			// Otherwise, if $e is ClapErrorKind::ArgumentNotFound or Ok(),
			// then convert to Option
			_ => $e.ok()
		}
	)
}

macro_rules! if_option {
	(Option<$type:ty>, THEN {$($then:tt)*} ELSE {$($otherwise:tt)*}) => (
		$($then)*
	);
	($type:ty, THEN {$($then:tt)*} ELSE {$($otherwise:tt)*}) => (
		$($otherwise)*
	);
}

macro_rules! if_vec {
	(Vec<$type:ty>, THEN {$($then:tt)*} ELSE {$($otherwise:tt)*}) => (
		$($then)*
	);
	($type:ty, THEN {$($then:tt)*} ELSE {$($otherwise:tt)*}) => (
		$($otherwise)*
	);
}

macro_rules! if_option_vec {
	(Option<Vec<String>>, THEN {$then:expr} ELSE {$otherwise:expr}) => (
		$then
	);
	(Option<$type:ty>, THEN {$then:expr} ELSE {$otherwise:expr}) => (
		$otherwise
	);
}

macro_rules! inner_option_type {
	(Option<$type:ty>) => (
		$type
	)
}

macro_rules! inner_vec_type {
	(Vec<$type:ty>) => (
		$type
	)
}

macro_rules! inner_option_vec_type {
	(Option<Vec<String>>) => (
		String
	)
}

macro_rules! usage_with_ident {
	($name:expr, $usage:expr, $help:expr) => (
		if $usage.contains("<") {
			format!("<{}> {} '{}'",$name, $usage, $help)
		} else {
			format!("[{}] {} '{}'",$name, $usage, $help)
		}
	);
}

macro_rules! underscore_to_hyphen {
	($e:expr) => (
		str::replace($e, "_", "-")
	)
}

macro_rules! usage {
	(
		{
			$(
				CMD $subc:ident
				{
					$subc_help:expr,

					$(
						CMD $subc_subc:ident
						{
							$subc_subc_help:expr,
							$(
								FLAG $subc_subc_flag:ident : (bool) = false, $subc_subc_flag_usage:expr, $subc_subc_flag_help:expr,
							)*
							$(
								ARG $subc_subc_arg:ident : ($($subc_subc_arg_type_tt:tt)+) = $subc_subc_arg_default:expr, $subc_subc_arg_usage:expr, $subc_subc_arg_help:expr,
							)*
						}
					)*

					$(
						FLAG $subc_flag:ident : (bool) = false, $subc_flag_usage:expr, $subc_flag_help:expr,
					)*
					$(
						ARG $subc_arg:ident : ($($subc_arg_type_tt:tt)+) = $subc_arg_default:expr, $subc_arg_usage:expr, $subc_arg_help:expr,
					)*
				}
			)*
		}
		{
			$(
			[$group_name:expr]
				$(
					FLAG $flag:ident : (bool) = false, or $flag_from_config:expr, $flag_usage:expr, $flag_help:expr,
				)*
				$(
					ARG $arg:ident : ($($arg_type_tt:tt)+) = $arg_default:expr, or $arg_from_config:expr, $arg_usage:expr, $arg_help:expr,
				)*
			)*
		}
	) => {
		use toml;
		use std::{fs, io, process};
		use std::io::{Read, Write};
		use util::version;
		use clap::{Arg, App, SubCommand, AppSettings, Error as ClapError, ErrorKind as ClapErrorKind};
		use helpers::replace_home;
		use std::ffi::OsStr;
		use std::collections::HashMap;

		#[cfg(test)]
		use regex::Regex;

		#[derive(Debug)]
		pub enum ArgsError {
			Clap(ClapError),
			Decode(toml::de::Error),
			Config(String, io::Error),
		}

		impl ArgsError {
			pub fn exit(self) -> ! {
				match self {
					ArgsError::Clap(e) => e.exit(),
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
				}
			}
		}

		impl From<ClapError> for ArgsError {
			fn from(e: ClapError) -> Self {
				ArgsError::Clap(e)
			}
		}

		impl From<toml::de::Error> for ArgsError {
			fn from(e: toml::de::Error) -> Self {
				ArgsError::Decode(e)
			}
		}

		#[derive(Debug, PartialEq)]
		pub struct Args {
			$(
				pub $subc: bool,

				$(
					pub $subc_subc: bool,
					$(
						pub $subc_subc_flag: bool,
					)*
					$(
						pub $subc_subc_arg: $($subc_subc_arg_type_tt)+,
					)*
				)*

				$(
					pub $subc_flag: bool,
				)*
				$(
					pub $subc_arg: $($subc_arg_type_tt)+,
				)*
			)*

			$(
				$(
					pub $flag: bool,
				)*
				$(
					pub $arg: $($arg_type_tt)+,
				)*
			)*
		}

		impl Default for Args {
			fn default() -> Self {
				Args {
					$(
						$subc: Default::default(),
						$(
							$subc_subc: Default::default(),
							$(
								$subc_subc_flag: Default::default(),
							)*
							$(
								$subc_subc_arg: Default::default(),
							)*
						)*

						$(
							$subc_flag: Default::default(),
						)*
						$(
							$subc_arg: Default::default(),
						)*
					)*

					$(
						$(
							$flag: Default::default(),
						)*
						$(
							$arg: Default::default(),
						)*
					)*
				}
			}
		}

		#[derive(Default, Debug, PartialEq, Clone, Deserialize)]
		struct RawArgs {
			$(
				$subc: bool,

				$(
					$subc_subc: bool,
					$(
						$subc_subc_flag: bool,
					)*
					$(
						$subc_subc_arg: if_option!(
							$($subc_subc_arg_type_tt)+,
							THEN { $($subc_subc_arg_type_tt)+ }
							ELSE { Option<$($subc_subc_arg_type_tt)+> }
						),
					)*
				)*

				$(
					$subc_flag: bool,
				)*
				$(
					$subc_arg: if_option!(
						$($subc_arg_type_tt)+,
						THEN { $($subc_arg_type_tt)+ }
						ELSE { Option<$($subc_arg_type_tt)+> }
					),
				)*

			)*
			$(
				$(
					$flag: bool,
				)*

				$(
					$arg: if_option!(
						$($arg_type_tt)+,
						THEN { $($arg_type_tt)+ }
						ELSE { Option<$($arg_type_tt)+> }
					),
				)*
			)*
		}

		impl Args {

			pub fn parse<S: AsRef<str>>(command: &[S]) -> Result<Self, ArgsError> {
				let raw_args = RawArgs::parse(command)?;

				// Skip loading config file if no_config flag is specified
				if raw_args.flag_no_config {
					return Ok(raw_args.into_args(Config::default()));
				}

				let config_file = raw_args.arg_config.clone().unwrap_or_else(|| raw_args.clone().into_args(Config::default()).arg_config);
				let config_file = replace_home(&::dir::default_data_path(), &config_file);
				match (fs::File::open(&config_file), raw_args.arg_config.clone()) {
					// Load config file
					(Ok(mut file), _) => {
						println_stderr!("Loading config file from {}", &config_file);
						let mut config = String::new();
						file.read_to_string(&mut config).map_err(|e| ArgsError::Config(config_file, e))?;
						Ok(raw_args.into_args(Self::parse_config(&config)?))
					},
					// Don't display error in case default config cannot be loaded.
					(Err(_), None) => Ok(raw_args.into_args(Config::default())),
					// Config set from CLI (fail with error)
					(Err(_), Some(ref config_arg)) => {
						match presets::preset_config_string(config_arg) {
							Ok(s) => Ok(raw_args.into_args(Self::parse_config(&s)?)),
							Err(e) => Err(ArgsError::Config(config_file, e))
						}
					},
				}
			}

			#[cfg(test)]
			pub fn parse_without_config<S: AsRef<str>>(command: &[S]) -> Result<Self, ArgsError> {
				Self::parse_with_config(command, Config::default())
			}

			#[cfg(test)]
			fn parse_with_config<S: AsRef<str>>(command: &[S], config: Config) -> Result<Self, ArgsError> {
				RawArgs::parse(command).map(|raw| raw.into_args(config)).map_err(ArgsError::Clap)
			}

			fn parse_config(config: &str) -> Result<Config, ArgsError> {
				Ok(toml::from_str(config)?)
			}

			pub fn print_version() -> String {
				format!(include_str!("./version.txt"), version())
			}

			#[allow(unused_mut)] // subc_subc_exist may be assigned true by the macro
			#[allow(unused_assignments)] // Rust issue #22630
			pub fn print_help() -> String {
				let mut help : String = include_str!("./usage_header.txt").to_owned();

				help.push_str("\n\n");

				// Subcommands
				help.push_str("parity [options]\n");
				$(
					{
						let mut subc_subc_exist = false;

						$(
							subc_subc_exist = true;
							let subc_subc_usages : Vec<&str> = vec![
								$(
									concat!("[",$subc_subc_flag_usage,"]"),
								)*
								$(
									$subc_subc_arg_usage,
								)*
							];

							if subc_subc_usages.is_empty() {
								help.push_str(&format!("parity [options] {} {}\n", underscore_to_hyphen!(&stringify!($subc)[4..]), underscore_to_hyphen!(&stringify!($subc_subc)[stringify!($subc).len()+1..])));
							} else {
								help.push_str(&format!("parity [options] {} {} {}\n", underscore_to_hyphen!(&stringify!($subc)[4..]), underscore_to_hyphen!(&stringify!($subc_subc)[stringify!($subc).len()+1..]), subc_subc_usages.join(" ")));
							}
						)*

						// Print the subcommand on its own only if it has no subsubcommands
						if !subc_subc_exist {
							let subc_usages : Vec<&str> = vec![
								$(
									concat!("[",$subc_flag_usage,"]"),
								)*
								$(
									$subc_arg_usage,
								)*
							];

							if subc_usages.is_empty() {
								help.push_str(&format!("parity [options] {}\n", underscore_to_hyphen!(&stringify!($subc)[4..])));
							} else {
								help.push_str(&format!("parity [options] {} {}\n", underscore_to_hyphen!(&stringify!($subc)[4..]), subc_usages.join(" ")));
							}
						}
					}
				)*

				// Arguments and flags
				$(
					help.push_str("\n");
					help.push_str($group_name); help.push_str(":\n");

					$(
						help.push_str(&format!("\t{}\n\t\t{}\n", $flag_usage, $flag_help));
					)*

					$(
						if_option!(
							$($arg_type_tt)+,
							THEN {
								if_option_vec!(
									$($arg_type_tt)+,
									THEN {
										help.push_str(&format!("\t{}\n\t\t{} (default: {:?})\n", $arg_usage, $arg_help, {let x : inner_option_type!($($arg_type_tt)+)> = $arg_default; x}))
									}
									ELSE {
										help.push_str(&format!("\t{}\n\t\t{}{}\n", $arg_usage, $arg_help, $arg_default.map(|x: inner_option_type!($($arg_type_tt)+)| format!(" (default: {})",x)).unwrap_or("".to_owned())))
									}
								)
							}
							ELSE {
								if_vec!(
									$($arg_type_tt)+,
									THEN {
										help.push_str(&format!("\t{}\n\t\t{} (default: {:?})\n", $arg_usage, $arg_help, {let x : $($arg_type_tt)+ = $arg_default; x}))
									}
									ELSE {
										help.push_str(&format!("\t{}\n\t\t{} (default: {})\n", $arg_usage, $arg_help, $arg_default))
									}
								)
							}
						);
					)*

				)*

				help
			}
		}

		impl RawArgs {
			fn into_args(self, config: Config) -> Args {
				let mut args = Args::default();
				$(
					args.$subc = self.$subc;

					$(
						args.$subc_subc = self.$subc_subc;
						$(
							args.$subc_subc_flag = self.$subc_subc_flag;
						)*
						$(
							args.$subc_subc_arg = if_option!(
								$($subc_subc_arg_type_tt)+,
								THEN { self.$subc_subc_arg.or($subc_subc_arg_default) }
								ELSE { self.$subc_subc_arg.unwrap_or($subc_subc_arg_default.into()) }
							);
						)*
					)*

					$(
						args.$subc_flag = self.$subc_flag;
					)*
					$(
						args.$subc_arg = if_option!(
							$($subc_arg_type_tt)+,
							THEN { self.$subc_arg.or($subc_arg_default) }
							ELSE { self.$subc_arg.unwrap_or($subc_arg_default.into()) }
						);
					)*
				)*

				$(
					$(
						args.$flag = self.$flag || $flag_from_config(&config).unwrap_or(false);
					)*
					$(
						args.$arg = if_option!(
							$($arg_type_tt)+,
							THEN { self.$arg.or_else(|| $arg_from_config(&config)).or_else(|| $arg_default.into()) }
							ELSE { self.$arg.or_else(|| $arg_from_config(&config)).unwrap_or_else(|| $arg_default.into()) }
						);
					)*
				)*
				args
			}

			#[allow(unused_variables)] // the submatches of arg-less subcommands aren't used
			pub fn parse<S: AsRef<str>>(command: &[S]) -> Result<Self, ClapError> {

				let usages = vec![
					$(
						$(
							usage_with_ident!(stringify!($arg), $arg_usage, $arg_help),
						)*
						$(
							usage_with_ident!(stringify!($flag), $flag_usage, $flag_help),
						)*
					)*
				];

				// Hash of subc|subc_subc => Vec<String>
				let mut subc_usages = HashMap::new();
				$(
					{
						let this_subc_usages = vec![
							$(
								usage_with_ident!(stringify!($subc_flag), $subc_flag_usage, $subc_flag_help),
							)*
							$(
								usage_with_ident!(stringify!($subc_arg), $subc_arg_usage, $subc_arg_help),
							)*
						];

						subc_usages.insert(stringify!($subc),this_subc_usages);

						$(
							{
								let this_subc_subc_usages = vec![
									$(
										usage_with_ident!(stringify!($subc_subc_flag), $subc_subc_flag_usage, $subc_subc_flag_help),
									)*
									$(
										usage_with_ident!(stringify!($subc_subc_arg), $subc_subc_arg_usage, $subc_subc_arg_help),
									)*
								];

								subc_usages.insert(stringify!($subc_subc), this_subc_subc_usages);
							}
						)*
					}
				)*

				let matches = App::new("Parity")
				    	.global_setting(AppSettings::VersionlessSubcommands)
						.global_setting(AppSettings::DisableHelpSubcommand)
						.help(Args::print_help().as_ref())
						.args(&usages.iter().map(|u| Arg::from_usage(u).use_delimiter(false).allow_hyphen_values(true)).collect::<Vec<Arg>>())
						$(
							.subcommand(
								SubCommand::with_name(&underscore_to_hyphen!(&stringify!($subc)[4..]))
								.about($subc_help)
								.args(&subc_usages.get(stringify!($subc)).unwrap().iter().map(|u| Arg::from_usage(u).use_delimiter(false).allow_hyphen_values(true)).collect::<Vec<Arg>>())
								$(
									.setting(AppSettings::SubcommandRequired) // prevent from running `parity account`
									.subcommand(
										SubCommand::with_name(&underscore_to_hyphen!(&stringify!($subc_subc)[stringify!($subc).len()+1..]))
										.about($subc_subc_help)
										.args(&subc_usages.get(stringify!($subc_subc)).unwrap().iter().map(|u| Arg::from_usage(u).use_delimiter(false).allow_hyphen_values(true)).collect::<Vec<Arg>>())
									)
								)*
							)
						)*
						.get_matches_from_safe(command.iter().map(|x| OsStr::new(x.as_ref())))?;

				let mut raw_args : RawArgs = Default::default();
				$(
					$(
						raw_args.$flag = matches.is_present(stringify!($flag));
					)*
					$(
						raw_args.$arg = return_if_parse_error!(if_option!(
							$($arg_type_tt)+,
							THEN {
								if_option_vec!(
									$($arg_type_tt)+,
									THEN { values_t!(matches, stringify!($arg), inner_option_vec_type!($($arg_type_tt)+)) }
									ELSE { value_t!(matches, stringify!($arg), inner_option_type!($($arg_type_tt)+)) }
								)
							}
							ELSE {
								if_vec!(
									$($arg_type_tt)+,
									THEN { values_t!(matches, stringify!($arg), inner_vec_type!($($arg_type_tt)+)) }
									ELSE { value_t!(matches, stringify!($arg), $($arg_type_tt)+) }
								)
							}
						));
					)*
				)*

				// Subcommands
				$(
					if let Some(submatches) = matches.subcommand_matches(&underscore_to_hyphen!(&stringify!($subc)[4..])) {
						raw_args.$subc = true;

						// Subcommand flags
						$(
							raw_args.$subc_flag = submatches.is_present(&stringify!($subc_flag));
						)*
						// Subcommand arguments
						$(
							raw_args.$subc_arg = return_if_parse_error!(if_option!(
										$($subc_arg_type_tt)+,
										THEN {
											if_option_vec!(
												$($subc_arg_type_tt)+,
												THEN { values_t!(submatches, stringify!($subc_arg), inner_option_vec_type!($($subc_arg_type_tt)+)) }
												ELSE { value_t!(submatches, stringify!($subc_arg), inner_option_type!($($subc_arg_type_tt)+)) }
											)
										}
										ELSE {
											if_vec!(
												$($subc_arg_type_tt)+,
												THEN { values_t!(submatches, stringify!($subc_arg), inner_vec_type!($($subc_arg_type_tt)+)) }
												ELSE { value_t!(submatches, stringify!($subc_arg), $($subc_arg_type_tt)+) }
											)
										}
							));
						)*

						// Sub-subcommands
						$(
							if let Some(subsubmatches) = submatches.subcommand_matches(&underscore_to_hyphen!(&stringify!($subc_subc)[stringify!($subc).len()+1..])) {
								raw_args.$subc_subc = true;

								// Sub-subcommand flags
								$(
									raw_args.$subc_subc_flag = subsubmatches.is_present(&stringify!($subc_subc_flag));
								)*
								// Sub-subcommand arguments
								$(
									raw_args.$subc_subc_arg = return_if_parse_error!(if_option!(
										$($subc_subc_arg_type_tt)+,
										THEN {
											if_option_vec!(
												$($subc_subc_arg_type_tt)+,
												THEN { values_t!(subsubmatches, stringify!($subc_subc_arg), inner_option_vec_type!($($subc_subc_arg_type_tt)+)) }
												ELSE { value_t!(subsubmatches, stringify!($subc_subc_arg), inner_option_type!($($subc_subc_arg_type_tt)+)) }
											)
										}
										ELSE {
											if_vec!(
												$($subc_subc_arg_type_tt)+,
												THEN { values_t!(subsubmatches, stringify!($subc_subc_arg), inner_vec_type!($($subc_subc_arg_type_tt)+)) }
												ELSE { value_t!(subsubmatches, stringify!($subc_subc_arg), $($subc_subc_arg_type_tt)+) }
											)
										}
									));
								)*
							}
							else {
								raw_args.$subc_subc = false;
							}
						)*
					}
					else {
						raw_args.$subc = false;
					}
				)*

				Ok(raw_args)
			}

		}

		#[test]
		fn usages_valid() {
			let re = Regex::new(r"^(?:(-[a-zA-Z-]+, )?--[a-z-]+(=\[[a-zA-Z]+\](\.\.\.)?|=<[a-zA-Z]+>(\.\.\.)?)?)|(?:\[[a-zA-Z-]+\])(\.\.\.)?|(?:<[a-zA-Z-]+>)(\.\.\.)?$").unwrap();

			let usages = vec![
				$(
					$(
						$(
							$subc_subc_arg_usage,
						)*
					)*
					$(
						$subc_arg_usage,
					)*
				)*
				$(
					$(
						$flag_usage,
					)*
					$(
						$arg_usage,
					)*
				)*
			];

			for usage in &usages {
				assert!(re.is_match(usage));
			}
		}
	}
}
