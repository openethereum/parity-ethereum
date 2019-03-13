// Copyright 2019 Parity Technologies (UK) Ltd.
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

use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::sync::Arc;

use parity_ethereum::RunningClient;
use pyo3::exceptions::{RuntimeError, ValueError};
use pyo3::prelude::{
	pyclass, pyfunction, pymethods, pymodule, ObjectProtocol, PyModule, PyObject, PyResult, Python,
};
use pyo3::types::PyList;
use {
	parity_config_destroy, parity_config_from_cli, parity_destroy, parity_rpc_worker,
	parity_set_logger, parity_start, parity_unsubscribe_ws, parity_ws_worker, Callback,
	ParityParams,
};

/// Handle to a Parity config created with parity_config_from_cli
#[pyclass]
struct ConfigHandle {
	inner: *mut c_void,
}

impl Drop for ConfigHandle {
	fn drop(&mut self) {
		if self.inner != ptr::null_mut() {
			unsafe {
				parity_config_destroy(self.inner);
			}
		}
	}
}

/// Handle to a Parity instance created with parity_start
#[pyclass]
struct ParityHandle {
	inner: *mut c_void,
}

impl Drop for ParityHandle {
	fn drop(&mut self) {
		if self.inner != ptr::null_mut() {
			unsafe {
				parity_destroy(self.inner);
			}
		}
	}
}

/// Handle to a subscription created with parity_ws_worker
#[pyclass]
struct SubscriptionHandle {
	inner: *const c_void,
}

#[pymethods]
impl SubscriptionHandle {
	fn unsubscribe(&mut self) {
		if self.inner != ptr::null_mut() {
			unsafe {
				parity_unsubscribe_ws(self.inner);
			}

			self.inner = ptr::null_mut();
		}
	}
}

impl Drop for SubscriptionHandle {
	fn drop(&mut self) {
		self.unsubscribe()
	}
}

/// A Python callable which accepts a String value representing a RPC result or subscription event
struct PythonCallback(PyObject);

impl PythonCallback {
	fn new(callback: PyObject) -> Self {
		Self(callback)
	}
}

impl Callback for PythonCallback {
	fn call(&self, msg: &str) {
		// Assume this is called from Rust, acquire GIL
		let gil = Python::acquire_gil();
		let py = gil.python();

		// Run our callback, and re-raise any exceptions that occur
		if let Err(e) = self.0.call1(py, msg) {
			e.restore(py);
		}
	}
}

/// Generate a Parity config given a list of CLI options
#[pyfunction]
fn config_from_cli(_py: Python<'_>, cli: &PyList) -> PyResult<ConfigHandle> {
	let cli_len = cli.len();

	let opts_strings = cli
		.into_iter()
		.map(|o| o.extract())
		.collect::<PyResult<Vec<String>>>()?;

	let opts = opts_strings
		.iter()
		.map(|s: &String| s.as_ptr() as *const i8)
		.collect::<Vec<_>>();
	let opts_lens = opts_strings
		.iter()
		.map(|s| s.as_bytes().len())
		.collect::<Vec<_>>();

	let mut out = ptr::null_mut();
	unsafe {
		match parity_config_from_cli(opts.as_ptr(), opts_lens.as_ptr(), cli_len, &mut out) {
			0 => Ok(ConfigHandle { inner: out }),
			_ => Err(ValueError::py_err("failed to create config object")),
		}
	}
}

/// Create a running Parity instance given a config
#[pyfunction]
unsafe fn build(
	_py: Python<'_>,
	config: &mut ConfigHandle,
	logger_mode: &str,
	logger_file: &str,
) -> PyResult<ParityHandle> {
	let mut params = ParityParams {
		configuration: config.inner,
		..mem::zeroed()
	};

	parity_set_logger(
		logger_mode.as_ptr(),
		logger_mode.as_bytes().len(),
		logger_file.as_ptr(),
		logger_file.as_bytes().len(),
		&mut params.logger,
	);

	let mut out = ptr::null_mut();
	let ret = match parity_start(&params, &mut out) {
		0 => Ok(ParityHandle { inner: out }),
		_ => Err(RuntimeError::py_err("failed to start Parity")),
	};

	// Ensure we don't double-free config
	config.inner = ptr::null_mut();
	ret
}

/// Perform a RPC query against a running Parity instance, invoking callback on completion
#[pyfunction]
fn rpc_query(
	_py: Python<'_>,
	parity: &ParityHandle,
	rpc: &str,
	timeout_ms: u64,
	callback: PyObject,
) -> PyResult<()> {
	if parity.inner.is_null() {
		return Err(ValueError::py_err(
			"Attempt to query RPC when ParityClient is not running",
		));
	}

	let client = unsafe { &*(parity.inner as *const RunningClient) };
	let callback = Arc::new(PythonCallback::new(callback));
	parity_rpc_worker(client, rpc, callback, timeout_ms);
	Ok(())
}

/// Subscribe to a websocket event, invoking callback when event received
#[pyfunction]
fn subscribe_ws(
	_py: Python<'_>,
	parity: &ParityHandle,
	rpc: &str,
	callback: PyObject,
) -> PyResult<SubscriptionHandle> {
	if parity.inner.is_null() {
		return Err(ValueError::py_err(
			"Attempt to subscribe to WebSocket when ParityClient is not running",
		));
	}

	let client = unsafe { &*(parity.inner as *const RunningClient) };
	let callback = Arc::new(PythonCallback::new(callback));

	Ok(SubscriptionHandle {
		inner: parity_ws_worker(client, rpc, callback),
	})
}

/// Python extension module exposing Parity functionality
#[pymodule]
fn _parity(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
	m.add_wrapped(wrap_pyfunction!(config_from_cli))?;
	m.add_wrapped(wrap_pyfunction!(build))?;
	m.add_wrapped(wrap_pyfunction!(rpc_query))?;
	m.add_wrapped(wrap_pyfunction!(subscribe_ws))?;

	Ok(())
}
