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
use rustc_hex::FromHex;
use fetch::{Client as FetchClient, Fetch};
use hash_fetch::urlhint::{URLHintContract, URLHint, URLHintResult};
use parity_reactor::Remote;

use hyper;
use hyper::status::StatusCode;

use {Embeddable, SyncStatus, random_filename};
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

pub struct ContentFetcher<F: Fetch = FetchClient, R: URLHint + 'static = URLHintContract> {
	cache_path: PathBuf,
	resolver: R,
	cache: Arc<Mutex<ContentCache>>,
	sync: Arc<SyncStatus>,
	embeddable_on: Embeddable,
	remote: Remote,
	fetch: F,
	only_content: bool,
}

impl<R: URLHint + 'static, F: Fetch> Drop for ContentFetcher<F, R> {
	fn drop(&mut self) {
		// Clear cache path
		let _ = fs::remove_dir_all(&self.cache_path);
	}
}

impl<R: URLHint + 'static, F: Fetch> ContentFetcher<F, R> {

	pub fn new(
		resolver: R,
		sync_status: Arc<SyncStatus>,
		remote: Remote,
		fetch: F,
	) -> Self {
		let mut cache_path = env::temp_dir();
		cache_path.push(random_filename());

		ContentFetcher {
			cache_path: cache_path,
			resolver: resolver,
			sync: sync_status,
			cache: Arc::new(Mutex::new(ContentCache::default())),
			embeddable_on: None,
			remote: remote,
			fetch: fetch,
			only_content: true,
		}
	}

	pub fn allow_dapps(mut self, dapps: bool) -> Self {
		self.only_content = !dapps;
		self
	}

	pub fn embeddable_on(mut self, embeddable_on: Embeddable) -> Self {
		self.embeddable_on = embeddable_on;
		self
	}

	fn still_syncing(embeddable: Embeddable) -> Box<Handler> {
		Box::new(ContentHandler::error(
			StatusCode::ServiceUnavailable,
			"Sync In Progress",
			"Your node is still syncing. We cannot resolve any content before it's fully synced.",
			Some("<a href=\"javascript:window.location.reload()\">Refresh</a>"),
			embeddable,
		))
	}

	fn dapps_disabled(address: Embeddable) -> Box<Handler> {
		Box::new(ContentHandler::error(
			StatusCode::ServiceUnavailable,
			"Network Dapps Not Available",
			"This interface doesn't support network dapps for security reasons.",
			None,
			address,
		))
	}

	#[cfg(test)]
	fn set_status(&self, content_id: &str, status: ContentStatus) {
		self.cache.lock().insert(content_id.to_owned(), status);
	}

	// resolve contract call synchronously.
	// TODO: port to futures-based hyper and make it all async.
	fn resolve(&self, content_id: Vec<u8>) -> Option<URLHintResult> {
		use futures::Future;

		self.resolver.resolve(content_id)
			.wait()
			.unwrap_or_else(|e| { warn!("Error resolving content-id: {}", e); None })
	}
}

impl<R: URLHint + 'static, F: Fetch> Fetcher for ContentFetcher<F, R> {
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
			// if there is content or we are syncing return true
			self.sync.is_major_importing() || self.resolve(content_id).is_some()
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
					let content = self.resolve(content_hex);

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
						Some(URLHintResult::Dapp(_)) if self.only_content => {
							(None, Self::dapps_disabled(self.embeddable_on.clone()))
						},
						Some(URLHintResult::Dapp(dapp)) => {
							let handler = ContentFetcherHandler::new(
								dapp.url(),
								path,
								control,
								installers::Dapp::new(
									content_id.clone(),
									self.cache_path.clone(),
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
									self.cache_path.clone(),
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
	use futures::{future, Future, BoxFuture};
	use hash_fetch::urlhint::{URLHint, URLHintResult};
	use parity_reactor::Remote;

	use apps::cache::ContentStatus;
	use endpoint::EndpointInfo;
	use page::LocalPageEndpoint;
	use super::{ContentFetcher, Fetcher};
	use {SyncStatus};

	#[derive(Clone)]
	struct FakeResolver;
	impl URLHint for FakeResolver {
		fn resolve(&self, _id: Bytes) -> BoxFuture<Option<URLHintResult>, String> {
			future::ok(None).boxed()
		}
	}

	#[derive(Debug)]
	struct FakeSync(bool);
	impl SyncStatus for FakeSync {
		fn is_major_importing(&self) -> bool { self.0 }
		fn peers(&self) -> (usize, usize) { (0, 5) }
	}

	#[test]
	fn should_true_if_contains_the_app() {
		// given
		let path = env::temp_dir();
		let fetcher = ContentFetcher::new(FakeResolver, Arc::new(FakeSync(false)), Remote::new_sync(), Client::new().unwrap())
			.allow_dapps(true);
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
