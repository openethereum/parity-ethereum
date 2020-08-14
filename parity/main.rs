// Copyright 2015-2019 Parity Technologies (UK) Ltd.
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

//! Ethcore client application.

#![warn(missing_docs)]

extern crate ctrlc;
extern crate dir;
extern crate fdlimit;
#[macro_use]
extern crate log;
extern crate ansi_term;
extern crate panic_hook;
extern crate parity_daemonize;
extern crate parity_ethereum;
extern crate parking_lot;

extern crate ethcore_logger;
#[cfg(windows)]
extern crate winapi;

use std::{
    io::Write,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use ansi_term::Colour;
use ctrlc::CtrlC;
use ethcore_logger::setup_log;
use fdlimit::raise_fd_limit;
use parity_daemonize::AsHandle;
use parity_ethereum::{start, ExecutionAction};
use parking_lot::{Condvar, Mutex};

#[derive(Debug)]
/// Status used to exit or restart the program.
struct ExitStatus {
    /// Whether the program panicked.
    panicking: bool,
    /// Whether the program should exit.
    should_exit: bool,
}

fn main() -> Result<(), i32> {
    let conf = {
        let args = std::env::args().collect::<Vec<_>>();
        parity_ethereum::Configuration::parse_cli(&args).unwrap_or_else(|e| e.exit())
    };

    let logger = setup_log(&conf.logger_config()).unwrap_or_else(|e| {
        eprintln!("{}", e);
        process::exit(2)
    });

    // FIXME: `pid_file` shouldn't need to cloned here
    // see: `https://github.com/paritytech/parity-daemonize/pull/13` for more info
    let handle = if let Some(pid) = conf.args.arg_daemon_pid_file.clone() {
        info!(
            "{}",
            Colour::Blue.paint("starting in daemon mode").to_string()
        );
        let _ = std::io::stdout().flush();

        match parity_daemonize::daemonize(pid) {
            Ok(h) => Some(h),
            Err(e) => {
                error!("{}", Colour::Red.paint(format!("{}", e)));
                return Err(1);
            }
        }
    } else {
        None
    };

    // increase max number of open files
    raise_fd_limit();

    let exit = Arc::new((
        Mutex::new(ExitStatus {
            panicking: false,
            should_exit: false,
        }),
        Condvar::new(),
    ));

    // Double panic can happen. So when we lock `ExitStatus` after the main thread is notified, it cannot be locked
    // again.
    let exiting = Arc::new(AtomicBool::new(false));

    trace!(target: "mode", "Not hypervised: not setting exit handlers.");
    let exec = start(conf, logger);

    match exec {
        Ok(result) => match result {
            ExecutionAction::Instant(output) => {
                if let Some(s) = output {
                    println!("{}", s);
                }
            }
            ExecutionAction::Running(client) => {
                panic_hook::set_with({
                    let e = exit.clone();
                    let exiting = exiting.clone();
                    move |panic_msg| {
                        warn!("Panic occured, see stderr for details");
                        eprintln!("{}", panic_msg);
                        if !exiting.swap(true, Ordering::SeqCst) {
                            *e.0.lock() = ExitStatus {
                                panicking: true,
                                should_exit: true,
                            };
                            e.1.notify_all();
                        }
                    }
                });

                CtrlC::set_handler({
                    let e = exit.clone();
                    let exiting = exiting.clone();
                    move || {
                        if !exiting.swap(true, Ordering::SeqCst) {
                            *e.0.lock() = ExitStatus {
                                panicking: false,
                                should_exit: true,
                            };
                            e.1.notify_all();
                        }
                    }
                });

                // so the client has started successfully
                // if this is a daemon, detach from the parent process
                if let Some(mut handle) = handle {
                    handle.detach()
                }

                // Wait for signal
                let mut lock = exit.0.lock();
                if !lock.should_exit {
                    let _ = exit.1.wait(&mut lock);
                }

                client.shutdown();

                if lock.panicking {
                    return Err(1);
                }
            }
        },
        Err(err) => {
            // error occured during start up
            // if this is a daemon, detach from the parent process
            if let Some(mut handle) = handle {
                handle.detach_with_msg(format!("{}", Colour::Red.paint(&err)))
            }
            eprintln!("{}", err);
            return Err(1);
        }
    };

    Ok(())
}
