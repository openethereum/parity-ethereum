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

//! Panic utilities

use std::thread;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::default::Default;

/// Thread-safe closure for handling possible panics
pub trait OnPanicListener: Send + Sync + 'static {
	/// Invoke listener
	fn call(&mut self, arg: &str);
}

/// Forwards panics from child
pub trait ForwardPanic {
	/// Attach `on_panic` listener to `child` and rethrow all panics
	fn forward_from<S>(&self, child: &S) where S : MayPanic;
}

/// Trait indicating that the structure catches some of the panics (most probably from spawned threads)
/// and it's possbile to be notified when one of the threads panics.
pub trait MayPanic {
	/// `closure` will be invoked whenever panic in thread is caught
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener;
}

struct PanicGuard<'a> {
	handler: &'a PanicHandler,
}

impl<'a> Drop for PanicGuard<'a> {
	fn drop(&mut self) {
		if thread::panicking() {
			self.handler.notify_all("Panic!".to_owned());
		}
	}
}

/// Structure that allows to catch panics and notify listeners
pub struct PanicHandler {
	listeners: Mutex<Vec<Box<OnPanicListener>>>
}

impl Default for PanicHandler {
	fn default() -> Self {
		PanicHandler::new()
	}
}

impl PanicHandler {
	/// Creates new `PanicHandler` wrapped in `Arc`
	pub fn new_in_arc() -> Arc<Self> {
		Arc::new(Self::new())
	}

	/// Creates new `PanicHandler`
	pub fn new() -> Self {
		PanicHandler {
			listeners: Mutex::new(vec![])
		}
	}

	/// Invoke closure and catch any possible panics.
	/// In case of panic notifies all listeners about it.
	#[cfg_attr(feature="dev", allow(deprecated))]
	pub fn catch_panic<G, R>(&self, g: G) -> thread::Result<R> where G: FnOnce() -> R + Send + 'static {
		let _guard = PanicGuard { handler: self };
		let result = g();
		Ok(result)
	}

	/// Notifies all listeners in case there is a panic.
	/// You should use `catch_panic` instead of calling this method explicitly.
	pub fn notify_all(&self, r: String) {
		let mut listeners = self.listeners.lock().unwrap();
		for listener in listeners.deref_mut() {
			listener.call(&r);
		}
	}
}

impl MayPanic for PanicHandler {
	fn on_panic<F>(&self, closure: F) where F: OnPanicListener {
		self.listeners.lock().unwrap().push(Box::new(closure));
	}
}

impl ForwardPanic for Arc<PanicHandler> {
	fn forward_from<S>(&self, child: &S) where S : MayPanic {
		let p = self.clone();
		child.on_panic(move |t| p.notify_all(t));
	}
}

impl<F> OnPanicListener for F
	where F: FnMut(String) + Send + Sync + 'static {
	fn call(&mut self, arg: &str) {
		self(arg.to_owned())
	}
}

#[test]
#[ignore] // panic forwarding doesnt work on the same thread in beta
fn should_notify_listeners_about_panic () {
	use std::sync::RwLock;
	// given
	let invocations = Arc::new(RwLock::new(vec![]));
	let i = invocations.clone();
	let p = PanicHandler::new();
	p.on_panic(move |t| i.write().unwrap().push(t));

	// when
	p.catch_panic(|| panic!("Panic!")).unwrap_err();

	// then
	assert!(invocations.read().unwrap()[0] == "Panic!");
}

#[test]
#[ignore] // panic forwarding doesnt work on the same thread in beta
fn should_notify_listeners_about_panic_when_string_is_dynamic () {
	use std::sync::RwLock;
	// given
	let invocations = Arc::new(RwLock::new(vec![]));
	let i = invocations.clone();
	let p = PanicHandler::new();
	p.on_panic(move |t| i.write().unwrap().push(t));

	// when
	p.catch_panic(|| panic!("Panic: {}", 1)).unwrap_err();

	// then
	assert!(invocations.read().unwrap()[0] == "Panic: 1");
}

#[test]
fn should_notify_listeners_about_panic_in_other_thread () {
	use std::thread;
	use std::sync::RwLock;

	// given
	let invocations = Arc::new(RwLock::new(vec![]));
	let i = invocations.clone();
	let p = PanicHandler::new();
	p.on_panic(move |t| i.write().unwrap().push(t));

	// when
	let t = thread::spawn(move ||
		p.catch_panic(|| panic!("Panic!")).unwrap()
	);
	t.join().unwrap_err();

	// then
	assert!(invocations.read().unwrap()[0] == "Panic!");
}

#[test]
#[ignore] // panic forwarding doesnt work on the same thread in beta
fn should_forward_panics () {
use std::sync::RwLock;
	// given
	let invocations = Arc::new(RwLock::new(vec![]));
	let i = invocations.clone();
	let p = PanicHandler::new_in_arc();
	p.on_panic(move |t| i.write().unwrap().push(t));

	let p2 = PanicHandler::new();
	p.forward_from(&p2);

	// when
	p2.catch_panic(|| panic!("Panic!")).unwrap_err();

	// then
	assert!(invocations.read().unwrap()[0] == "Panic!");
}
