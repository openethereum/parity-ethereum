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
use std::any::Any;
use std::sync::{Arc, Mutex};

pub trait OnPanicListener<T>: Send + Sync + 'static {
	fn call(&mut self, arg: &T);
}

impl<F, T> OnPanicListener<T> for F
	where F: FnMut(&T) + Send + Sync + 'static {
	fn call(&mut self, arg: &T) {
		self(arg)
	}
}

pub trait ArgsConverter<T> : Send + Sync {
	fn convert(&self, t: &Box<Any + Send>) -> Option<T>;
}

pub trait MayPanic<T> {
	fn on_panic<F>(&self, closure: F)
		where F: OnPanicListener<T>;
}

pub trait PanicHandler<T, C: ArgsConverter<T>> : MayPanic<T>{
	fn with_converter(converter: C) -> Self;
	fn catch_panic<G, R>(&self, g: G) -> thread::Result<R>
		where G: FnOnce() -> R + Send + 'static;
	fn notify_all(&self, &T);
}

pub struct StringConverter;
impl ArgsConverter<String> for StringConverter {
	fn convert(&self, t: &Box<Any + Send>) -> Option<String> {
		let as_str = t.downcast_ref::<&'static str>().map(|t| t.clone().to_owned());
		let as_string = t.downcast_ref::<String>().cloned();

		as_str.or(as_string)
	}
}

pub struct BasePanicHandler<T, C>
	where C: ArgsConverter<T>, T: 'static {
	converter: C,
	listeners: Mutex<Vec<Box<OnPanicListener<T>>>>
}

impl<T, C> PanicHandler<T, C> for BasePanicHandler<T, C>
	where C: ArgsConverter<T>, T: 'static {

	fn with_converter(converter: C) -> Self {
		BasePanicHandler {
			converter: converter,
			listeners: Mutex::new(vec![])
		}
	}

	#[allow(deprecated)]
	// TODO [todr] catch_panic is deprecated but panic::recover has different bounds (not allowing mutex)
	fn catch_panic<G, R>(&self, g: G) -> thread::Result<R> where G: FnOnce() -> R + Send + 'static {
		let result = thread::catch_panic(g);

		if let Err(ref e) = result {
			let res = self.converter.convert(e);
			if let Some(r) = res {
				self.notify_all(&r);
			}
		}

		result
	}

	fn notify_all(&self, r: &T) {
		let mut listeners = self.listeners.lock().unwrap();
		for listener in listeners.deref_mut() {
			listener.call(r);
		}
	}
}

impl<T, C> MayPanic<T> for BasePanicHandler<T, C>
	where C: ArgsConverter<T>, T: 'static {
	fn on_panic<F>(&self, closure: F)
		where F: OnPanicListener<T> {
		self.listeners.lock().unwrap().push(Box::new(closure));
	}
}

pub struct StringPanicHandler {
	handler: BasePanicHandler<String, StringConverter>
}

impl StringPanicHandler {
	pub fn new_arc() -> Arc<StringPanicHandler> {
		Arc::new(Self::new())
	}

	pub fn new () -> Self {
		Self::with_converter(StringConverter)
	}
}

impl PanicHandler<String, StringConverter> for StringPanicHandler {

	fn with_converter(converter: StringConverter) -> Self {
		StringPanicHandler {
			handler: BasePanicHandler::with_converter(converter)
		}
	}

	fn catch_panic<G, R>(&self, g: G) -> thread::Result<R> where G: FnOnce() -> R + Send + 'static {
		self.handler.catch_panic(g)
	}

	fn notify_all(&self, r: &String) {
		self.handler.notify_all(r);
	}
}

impl MayPanic<String> for StringPanicHandler {
	fn on_panic<F>(&self, closure: F)
		where F: OnPanicListener<String> {
			self.handler.on_panic(closure)
		}
}

#[test]
fn should_notify_listeners_about_panic () {
	use std::sync::RwLock;
	// given
	let invocations = Arc::new(RwLock::new(vec![]));
	let i = invocations.clone();
	let p = StringPanicHandler::new();
	p.on_panic(move |t: &String| i.write().unwrap().push(t.clone()));

	// when
	p.catch_panic(|| panic!("Panic!")).unwrap_err();

	// then
	assert!(invocations.read().unwrap()[0] == "Panic!");
}

#[test]
fn should_notify_listeners_about_panic_when_string_is_dynamic () {
	use std::sync::RwLock;
	// given
	let invocations = Arc::new(RwLock::new(vec![]));
	let i = invocations.clone();
	let p = StringPanicHandler::new();
	p.on_panic(move |t: &String| i.write().unwrap().push(t.clone()));

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
	let p = StringPanicHandler::new();
	p.on_panic(move |t: &String| i.write().unwrap().push(t.clone()));

	// when
	let t = thread::spawn(move ||
		p.catch_panic(|| panic!("Panic!")).unwrap()
	);
	t.join().unwrap_err();

	// then
	assert!(invocations.read().unwrap()[0] == "Panic!");
}

#[test]
fn should_forward_panics () {
use std::sync::RwLock;
	// given
	let invocations = Arc::new(RwLock::new(vec![]));
	let i = invocations.clone();
	let p = StringPanicHandler::new();
	p.on_panic(move |t: &String| i.write().unwrap().push(t.clone()));

	let p2 = StringPanicHandler::new();
	p2.on_panic(move |t: &String| p.notify_all(t));

	// when
	p2.catch_panic(|| panic!("Panic!")).unwrap_err();

	// then
	assert!(invocations.read().unwrap()[0] == "Panic!");
}
