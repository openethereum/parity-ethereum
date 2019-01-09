// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use std::io::{Error, ErrorKind};

pub fn preset_config_string(arg: &str) -> Result<&'static str, Error> {
    match arg.to_lowercase().as_ref() {
        "dev" => Ok(include_str!("./config.dev.toml")),
        "mining" => Ok(include_str!("./config.mining.toml")),
        "non-standard-ports" => Ok(include_str!("./config.non-standard-ports.toml")),
        "insecure" => Ok(include_str!("./config.insecure.toml")),
        "dev-insecure" => Ok(include_str!("./config.dev-insecure.toml")),
        _ => Err(Error::new(ErrorKind::InvalidInput, "Config doesn't match any presets [dev, mining, non-standard-ports, insecure, dev-insecure]"))
    }
}
