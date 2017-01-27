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

//! Fetchable Dapps support.
//! Manages downloaded (cached) Dapps and downloads them when necessary.
//! Uses `URLHint` to resolve addresses into Dapps bundle file location.

mod installers;

use std::{fs, env};
use std::path::PathBuf;
use std::sync::Arc;
use rustc_serialize::hex::FromHex;
use fetch::{Client as FetchClient, Fetch};
use hash_fetch::urlhint::{URLHintContract, URLHint, URLHintResult};
use parity_reactor::Remote;

use hyper;
use hyper::status::StatusCode;

use {SyncStatus, random_filename};
use util::Mutex;
use page::LocalPageEndpoint;
use handlers::{ContentHandler, ContentFetcherHandler};
use endpoint::{Endpoint, EndpointPath, Handler};
use apps::cache::{ContentCache, ContentStatus};

/// Limit of cached dapps/content
const MAX_CACHED_DAPPS: usize = 20;

pub trait Fetcher: Send + Sync + 'static {
	fn contains(&self, content_id: &str) -> bool;

	fn to_async_handler(&self, path: EndpointPath, control: hyper::Control) -> Box<Handler>;
}

pub struct ContentFetcher<F: Fetch = FetchClient, R: URLHint + Send + Sync + 'static = URLHintContract> {
	dapps_path: PathBuf,
	resolver: R,
	cache: Arc<Mutex<ContentCache>>,
	sync: Arc<SyncStatus>,
	embeddable_on: Option<(String, u16)>,
	remote: Remote,
	fetch: F,
}

impl<R: URLHint + Send + Sync + 'static, F: Fetch> Drop for ContentFetcher<F, R> {
	fn drop(&mut self) {
		// Clear cache path
		let _ = fs::remove_dir_all(&self.dapps_path);
	}
}

impl<R: URLHint + Send + Sync + 'static, F: Fetch> ContentFetcher<F, R> {

	pub fn new(resolver: R, sync_status: Arc<SyncStatus>, embeddable_on: Option<(String, u16)>, remote: Remote, fetch: F) -> Self {
		let mut dapps_path = env::temp_dir();
		dapps_path.push(random_filename());

		ContentFetcher {
			dapps_path: dapps_path,
			resolver: resolver,
			sync: sync_status,
			cache: Arc::new(Mutex::new(ContentCache::default())),
			embeddable_on: embeddable_on,
			remote: remote,
			fetch: fetch,
		}
	}

	fn still_syncing(address: Option<(String, u16)>) -> Box<Handler> {
		Box::new(ContentHandler::error(
			StatusCode::ServiceUnavailable,
			"Sync In Progress",
			"Your node is still syncing. We cannot resolve any content before it's fully synced.",
			Some("<a href=\"javascript:window.location.reload()\">Refresh</a>"),
			address,
		))
	}

	#[cfg(test)]
	fn set_status(&self, content_id: &str, status: ContentStatus) {
		self.cache.lock().insert(content_id.to_owned(), status);
	}
}

impl<R: URLHint + Send + Sync + 'static, F: Fetch> Fetcher for ContentFetcher<F, R> {
	fn contains(&self, content_id: &str) -> bool {
		{
			let mut cache = self.cache.lock();
			// Check if we already have the app
			if cache.get(content_id).is_some() {
				return true;
			}
		}
		// fallback to resolver
		if let Ok(content_id) = content_id.from_hex() {
			// else try to resolve the app_id
			let has_content = self.resolver.resolve(content_id).is_some();
			// if there is content or we are syncing return true
			has_content || self.sync.is_major_importing()
		} else {
			false
		}
	}

	fn to_async_handler(&self, path: EndpointPath, control: hyper::Control) -> Box<Handler> {
		let mut cache = self.cache.lock();
		let content_id = path.app_id.clone();

		let (new_status, handler) = {
			let status = cache.get(&content_id);
			match status {
				// Just serve the content
				Some(&mut ContentStatus::Ready(ref endpoint)) => {
					(None, endpoint.to_async_handler(path, control))
				},
				// Content is already being fetched
				Some(&mut ContentStatus::Fetching(ref fetch_control)) if !fetch_control.is_deadline_reached() => {
					trace!(target: "dapps", "Content fetching in progress. Waiting...");
					(None, fetch_control.to_async_handler(path, control))
				},
				// We need to start fetching the content
				_ => {
					trace!(target: "dapps", "Content unavailable. Fetching... {:?}", content_id);
					let content_hex = content_id.from_hex().expect("to_handler is called only when `contains` returns true.");
					let content = self.resolver.resolve(content_hex);

					let cache = self.cache.clone();
					let id = content_id.clone();
					let on_done = move |result: Option<LocalPageEndpoint>| {
						let mut cache = cache.lock();
						match result {
							Some(endpoint) => cache.insert(id.clone(), ContentStatus::Ready(endpoint)),
							// In case of error
							None => cache.remove(&id),
						};
					};

					match content {
						// Don't serve dapps if we are still syncing (but serve content)
						Some(URLHintResult::Dapp(_)) if self.sync.is_major_importing() => {
							(None, Self::still_syncing(self.embeddable_on.clone()))
						},
						Some(URLHintResult::Dapp(dapp)) => {
							let handler = ContentFetcherHandler::new(
								dapp.url(),
								path,
								control,
								installers::Dapp::new(
									content_id.clone(),
									self.dapps_path.clone(),
									Box::new(on_done),
									self.embeddable_on.clone(),
								),
								self.embeddable_on.clone(),
								self.remote.clone(),
								self.fetch.clone(),
							);

							(Some(ContentStatus::Fetching(handler.fetch_control())), Box::new(handler) as Box<Handler>)
						},
						Some(URLHintResult::Content(content)) => {
							let handler = ContentFetcherHandler::new(
								content.url,
								path,
								control,
								installers::Content::new(
									content_id.clone(),
									content.mime,
									self.dapps_path.clone(),
									Box::new(on_done),
								),
								self.embeddable_on.clone(),
								self.remote.clone(),
								self.fetch.clone(),
							);

							(Some(ContentStatus::Fetching(handler.fetch_control())), Box::new(handler) as Box<Handler>)
						},
						None if self.sync.is_major_importing() => {
							(None, Self::still_syncing(self.embeddable_on.clone()))
						},
						None => {
							// This may happen when sync status changes in between
							// `contains` and `to_handler`
							(None, Box::new(ContentHandler::error(
								StatusCode::NotFound,
								"Resource Not Found",
								"Requested resource was not found.",
								None,
								self.embeddable_on.clone(),
							)) as Box<Handler>)
						},
					}
				},
			}
		};

		if let Some(status) = new_status {
			cache.clear_garbage(MAX_CACHED_DAPPS);
			cache.insert(content_id, status);
		}

		handler
	}
}

#[cfg(test)]
mod tests {
	use std::env;
	use std::sync::Arc;
	use util::Bytes;
	use fetch::{Fetch, Client};
	use hash_fetch::urlhint::{URLHint, URLHintResult};
	use parity_reactor::Remote;

	use apps::cache::ContentStatus;
	use endpoint::EndpointInfo;
	use page::LocalPageEndpoint;
	use super::{ContentFetcher, Fetcher};

	struct FakeResolver;
	impl URLHint for FakeResolver {
		fn resolve(&self, _id: Bytes) -> Option<URLHintResult> {
			None
		}
	}

	#[test]
	fn should_true_if_contains_the_app() {
		// given
		let path = env::temp_dir();
		let fetcher = ContentFetcher::new(FakeResolver, Arc::new(|| false), None, Remote::new_sync(), Client::new().unwrap());
		let handler = LocalPageEndpoint::new(path, EndpointInfo {
			name: "fake".into(),
			description: "".into(),
			version: "".into(),
			author: "".into(),
			icon_url: "".into(),
		}, Default::default(), None);

		// when
		fetcher.set_status("test", ContentStatus::Ready(handler));
		fetcher.set_status("test2", ContentStatus::Fetching(Default::default()));

		// then
		assert_eq!(fetcher.contains("test"), true);
		assert_eq!(fetcher.contains("test2"), true);
		assert_eq!(fetcher.contains("test3"), false);
	}
}
