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

//! Transport-specific metadata extractors.

use jsonrpc_core;
use http;
use hyper;
use minihttp;

/// HTTP RPC server impl-independent metadata extractor
pub trait HttpMetaExtractor: Send + Sync + 'static {
	/// Type of Metadata
	type Metadata: jsonrpc_core::Metadata;
	/// Extracts metadata from given params.
	fn read_metadata(&self, origin: Option<String>, user_agent: Option<String>, dapps_origin: Option<String>) -> Self::Metadata;
}

pub struct HyperMetaExtractor<T> {
	extractor: T,
}

impl<T> HyperMetaExtractor<T> {
	pub fn new(extractor: T) -> Self {
		HyperMetaExtractor {
			extractor: extractor,
		}
	}
}

impl<M, T> http::MetaExtractor<M> for HyperMetaExtractor<T> where
	T: HttpMetaExtractor<Metadata = M>,
	M: jsonrpc_core::Metadata,
{
	fn read_metadata(&self, req: &hyper::server::Request<hyper::net::HttpStream>) -> M {
		let as_string = |header: Option<&http::request_response::header::Raw>| header
			.and_then(|raw| raw.one())
			.map(|raw| String::from_utf8_lossy(raw).into_owned());

		let origin = as_string(req.headers().get_raw("origin"));
		let user_agent = as_string(req.headers().get_raw("user-agent"));
		let dapps_origin = as_string(req.headers().get_raw("x-parity-origin"));
		self.extractor.read_metadata(origin, user_agent, dapps_origin)
	}
}

pub struct MiniMetaExtractor<T> {
	extractor: T,
}

impl<T> MiniMetaExtractor<T> {
	pub fn new(extractor: T) -> Self {
		MiniMetaExtractor {
			extractor: extractor,
		}
	}
}

impl<M, T> minihttp::MetaExtractor<M> for MiniMetaExtractor<T> where
	T: HttpMetaExtractor<Metadata = M>,
	M: jsonrpc_core::Metadata,
{
	fn read_metadata(&self, req: &minihttp::Req) -> M {
		let origin = req.header("origin").map(|h| h.to_owned());
		let user_agent = req.header("user-agent").map(|h| h.to_owned());
		let dapps_origin = req.header("x-parity-origin").map(|h| h.to_owned());

		self.extractor.read_metadata(origin, user_agent, dapps_origin)
	}
}
