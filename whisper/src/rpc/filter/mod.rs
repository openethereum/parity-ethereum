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

//! Abstraction over filters which works

use bigint::hash::H512;
use ethkey::Public;
use parking_lot::RwLock;

use message::{Message, Topic};
use rpc::types::Message as FilterItem;

pub struct Filter {
	push: Box<Fn(FilterItem)>,
	full_topics: Vec<Vec<u8>>,
	abridged_topics: Vec<Topic>,
	from: Option<Public>,
	bloom: H512,
}

