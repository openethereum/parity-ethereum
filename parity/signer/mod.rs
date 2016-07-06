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

use std::sync::Arc;
use rpc_apis;
use util::panics::PanicHandler;

pub struct Configuration {
	pub enabled: bool,
	pub port: u16,
	pub signer_path: String,
}

pub struct Dependencies {
	pub panic_handler: Arc<PanicHandler>,
	pub apis: Arc<rpc_apis::Dependencies>,
}

#[cfg(feature = "ethcore-signer")]
mod on;
#[cfg(feature = "ethcore-signer")]
pub use self::on::*;

#[cfg(not(feature = "ethcore-signer"))]
mod off;
#[cfg(not(feature = "ethcore-signer"))]
pub use self::off::*;
