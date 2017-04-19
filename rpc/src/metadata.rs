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

use jsonrpc_core;
use http;
use hyper;
use minihttp;
use HttpMetaExtractor;

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
		let origin = req.headers().get::<hyper::header::Origin>()
			.map(|origin| format!("{}://{}", origin.scheme, origin.host))
			.unwrap_or_else(|| "unknown".into());
		let dapps_origin = req.headers().get_raw("x-parity-origin")
			.and_then(|raw| raw.one())
			.map(|raw| String::from_utf8_lossy(raw).into_owned());
		self.extractor.read_metadata(origin, dapps_origin)
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
		let origin = req.header("origin")
			.unwrap_or_else(|| "unknown")
			.to_owned();
		let dapps_origin = req.header("x-parity-origin").map(|h| h.to_owned());

		self.extractor.read_metadata(origin, dapps_origin)
	}
}
