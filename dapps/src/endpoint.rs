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

//! URL Endpoint traits

use std::collections::BTreeMap;

use futures::Future;
use hyper;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct EndpointPath {
	pub app_id: String,
	pub app_params: Vec<String>,
	pub query: Option<String>,
	pub host: String,
	pub port: u16,
	pub using_dapps_domains: bool,
}

impl EndpointPath {
	pub fn has_no_params(&self) -> bool {
		self.app_params.is_empty() || self.app_params.iter().all(|x| x.is_empty())
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct EndpointInfo {
	pub name: String,
	pub description: String,
	pub version: String,
	pub author: String,
	pub icon_url: String,
	pub local_url: Option<String>,
}

pub type Endpoints = BTreeMap<String, Box<Endpoint>>;
pub type Response = Box<Future<Item=hyper::Response, Error=hyper::Error> + Send>;
pub type Request = hyper::Request;

pub trait Endpoint : Send + Sync {
	fn info(&self) -> Option<&EndpointInfo> { None }

	fn respond(&self, path: EndpointPath, req: Request) -> Response;
}
