// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! in rust calling `fork` closes all open file descriptors
//! so to daemonize your program you have to call `fork` before you open any file descriptors
//! but you might want to confirm if the daemon actually started successfully
//! this library automatically pipes STDOUT/STDERR of your daemon process to STDOUT/STDERR of the parent process
//! and provides a handle to your daemon process to manually detach itself from the parent process
extern crate libc;
extern crate mio;
 #[macro_use]
 extern crate log;
#[macro_use]
extern crate failure;

use libc::{
	 close, dup2, fork, getpid, ioctl, pipe, splice, setsid, FIONREAD, STDERR_FILENO,
	STDIN_FILENO, STDOUT_FILENO, c_int, umask, open
};
use mio::*;
use std::{fs, mem};
use std::io::{self, Write};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
mod pipe;
pub mod error;

use pipe::Io;
pub use error::{Error, ErrorKind};
use std::env::set_current_dir;
use std::path::PathBuf;
pub use libc::{uid_t, gid_t, mode_t};

type Result<T> = std::result::Result<T, Error>;

macro_rules! assert_err {
	($e:expr, $err:expr) => {
		match $e {
			-1 => {
				Err::<_, Error>(From::from($err))
			}
			other => Ok(other),
		}
	};
}

unsafe fn get_pending_data_size(fd: RawFd, size: &mut usize) {
	if ioctl(fd, FIONREAD, size) == -1 {
		panic!("ioctl error {}", io::Error::last_os_error());
	}
}

/// handle returned from `daemonize` to the daemon process
/// the daemon should use this handle to detach itself from the
/// parent process, In cases where your program needs to run set up before starting
/// this can be useful, as the daemon will pipe it's stdout/stderr to the parent process
/// to communicate if start up was successful
pub struct Handle {
	file: Option<fs::File>
}

impl Handle {
	fn from_fd(fd: RawFd) -> Self {
		unsafe {
			Self {
				file: Some(fs::File::from_raw_fd(fd))
			}
		}
	}

	/// detach the daemon from the parent process
	/// this will write "Daemon started successfully" to stdout
	/// before detaching
	///
	/// # panics
	/// if detach is called more than once
	pub fn detach(&mut self) {
		self.detach_with_msg(b"Daemon started succesfully\n");
	}

	/// detach the daemon from the parent process
	/// with a custom message to be printed to stdout before detaching
	///
	/// # panics
	/// if detach_with_msg is called more than once
	pub fn detach_with_msg<T: AsRef<[u8]>>(&mut self, msg: T) {
		let mut file = self.file.take().expect("detach should only be called once");

		// redirect stdout/stderr to dev/null
		unsafe {
			let fd = open(mem::transmute(b"/dev/null\0"), libc::O_RDWR);
			let result = assert_err!(dup2(fd, STDERR_FILENO), ErrorKind::Dup2(io::Error::last_os_error())).and_then(
				|_| assert_err!(dup2(fd, STDOUT_FILENO), ErrorKind::Dup2(io::Error::last_os_error()))
			);
			if result.is_err() {
				error!(target: "daemonize", "Couldn't redirect STDOUT/STDERR to /dev/null, daemon will panic")
			}
		}

		file.write_all(msg.as_ref())
			.expect("Parent process won't exit until detach is called; \
			write can only fail if the read end of pipe is closed; qed");
	}
}

/// this will fork the calling process twice and return a handle to the
/// grandchild process aka daemon, use the handle to detach from the parent process
///
/// before `Handle::detach` is called the daemon process has it's STDOUT/STDERR
/// piped to the parent process' STDOUT/STDERR, this way any errors encountered by the
/// daemon during start up is reported.
#[cfg(not(windows))]
pub fn daemonize<T: Into<PathBuf>>(pid_file: T) -> Result<Handle> {
	unsafe {
		let mut chan = [-1 as c_int, -1 as c_int];
		let mut out_chan = [-1 as c_int, -1 as c_int];
		let mut err_chan = [-1 as c_int, -1 as c_int];

		assert_err!(pipe(&mut chan[0] as *mut c_int), ErrorKind::Pipe(io::Error::last_os_error()))?;
		assert_err!(pipe(&mut out_chan[0] as *mut c_int), ErrorKind::Pipe(io::Error::last_os_error()))?;
		assert_err!(pipe(&mut err_chan[0] as *mut c_int), ErrorKind::Pipe(io::Error::last_os_error()))?;

		let (rx, tx) = (chan[0], chan[1]);
		let (out_rx, out_tx) = (out_chan[0], out_chan[1]);
		let (err_rx, err_tx) = (err_chan[0], err_chan[1]);

		// fork once
		let pid = assert_err!(fork(), ErrorKind::Fork(io::Error::last_os_error()))?;

		if pid == 0 {
			trace!(target: "daemonize", "created child Process! {}", getpid());

			set_current_dir("/").map_err(|_| ErrorKind::ChangeDirectory)?;
			set_sid()?;
			umask(0o027);
			// fork again
			let pid = assert_err!(fork(), ErrorKind::Fork(io::Error::last_os_error()))?;

			// kill the the old parent
			if pid != 0 {
				trace!(target: "daemonize", "exiting child process! {}", getpid());
				::std::process::exit(0)
			}

			// we are now in the grandchild process aka daemon
			// close unused fds
			for fd in &[
				rx,
				out_rx,
				err_rx,
				STDERR_FILENO,
				STDIN_FILENO,
				STDOUT_FILENO,
			] {
				close(*fd);
			}

			let mut pid_file = fs::OpenOptions::new()
				.write(true)
				.truncate(true)
				.create(true)
				.open(pid_file.into())
				.map_err(|e| ErrorKind::OpenPidfile(e))?;

			pid_file
				.write_all(format!("{}", getpid()).as_bytes())
				.map_err(|e| ErrorKind::WritePid(e))?;

			// redirect stderr/stdout to out/err pipe
			assert_err!(dup2(err_tx, STDERR_FILENO), ErrorKind::Dup2(io::Error::last_os_error()))?;
			assert_err!(dup2(out_tx, STDOUT_FILENO), ErrorKind::Dup2(io::Error::last_os_error()))?;

			close(err_tx);
			close(out_tx);

			trace!(target: "daemonize", "grandchild process {}, aka daemon", getpid());

			return Ok(Handle::from_fd(tx));
		} else {
			// parent process
			trace!(target: "daemonize", "Parent process {}", getpid());

			for fd in &[tx, out_tx, err_tx] {
				close(*fd);
			}

			const STDOUT_READ_PIPE: Token = Token(0);
			const STDERR_READ_PIPE: Token = Token(1);
			const STATUS_REPORT_PIPE: Token = Token(3);

			let poll = mio::Poll::new().unwrap();

			let (stdout_read, stderr_read, status_read) = (
				Io::from_fd(out_rx).expect("failed to wrap pipe fd"),
				Io::from_fd(err_rx).expect("failed to wrap pipe fd"),
				Io::from_fd(rx).expect("failed to wrap pipe fd"),
			);

			poll.register(
				&stdout_read,
				STDOUT_READ_PIPE,
				Ready::readable(),
				PollOpt::edge(),
			).unwrap();

			poll.register(
				&stderr_read,
				STDERR_READ_PIPE,
				Ready::readable(),
				PollOpt::edge(),
			).unwrap();

			poll.register(
				&status_read,
				STATUS_REPORT_PIPE,
				Ready::readable(),
				PollOpt::edge(),
			).unwrap();

			let mut events = Events::with_capacity(1024);

			loop {
				poll.poll(&mut events, None).unwrap();

				for event in events.iter() {
					match event.token() {
						STDOUT_READ_PIPE => {
							let mut size = 0;
							get_pending_data_size(out_rx, &mut size);

							if splice(out_rx, 0 as *mut _, io::stdout().as_raw_fd(), 0 as *mut _, size, 0) == -1 {
								trace!(target: "daemonize", "splice failed");
							}
						}
						STDERR_READ_PIPE => {
							let mut size = 0;
							get_pending_data_size(err_rx, &mut size);

							if splice(err_rx,0 as *mut _,io::stderr().as_raw_fd(),0 as *mut _,size,0) == -1 {
								trace!(target: "daemonize", "splice failed {}", io::Error::last_os_error());
							}
						}
						STATUS_REPORT_PIPE => {
							let mut size = 0;
							get_pending_data_size(rx, &mut size);

							if splice(rx,0 as *mut _,io::stdout().as_raw_fd(),0 as *mut _,size,0) != -1 {
								trace!(target: "daemonize", "Exiting Parent Process");
								for fd in &[rx, out_rx, err_rx] {
									close(*fd);
								}
								::std::process::exit(0);
							} else {
								trace!(target: "daemonize", "splice failed {}", io::Error::last_os_error());
							}
						}
						_ => unreachable!(),
					}
				}
			}
		}
	}
}

#[cfg(windows)]
pub fn daemonize<T: Into<PathBuf>>(pid_file: T) -> Result<Handle> {
	Err(ErrorKind::Windows)?
}

unsafe fn set_sid() -> Result<()> {
	assert_err!(setsid(), ErrorKind::DetachSession(io::Error::last_os_error()))?;
	Ok(())
}

