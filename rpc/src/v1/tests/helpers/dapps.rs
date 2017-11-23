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

//! Test implementation of dapps service.

use v1::types::LocalDapp;
use v1::helpers::dapps::DappsService;

/// Test implementation of dapps service. Will always return the same list of dapps.
#[derive(Default, Clone)]
pub struct TestDappsService;

impl DappsService for TestDappsService {
	fn list_dapps(&self) -> Vec<LocalDapp> {
		vec![LocalDapp {
			id: "skeleton".into(),
			name: "Skeleton".into(),
			description: "A skeleton dapp".into(),
			version: "0.1".into(),
			author: "Parity Technologies Ltd".into(),
			icon_url: "title.png".into(),
			local_url: None,
		}]
	}

	fn refresh_local_dapps(&self) -> bool {
		true
	}
}
