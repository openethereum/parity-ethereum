// Copyright 2015-2020 Parity Technologies (UK) Ltd.
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

use ethcore::client::ClientConfig;

pub fn client_db_config(client_config: &ClientConfig) -> super::sled::DatabaseConfig {
	let mut client_db_config = super::sled::DatabaseConfig::with_columns(ethcore_db::NUM_COLUMNS);
	client_db_config.memory_budget_mb = client_config.db_cache_size.map(|s| s as u64);
	client_db_config
}
