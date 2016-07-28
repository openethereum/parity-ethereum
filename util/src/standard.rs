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

//! Std lib global reexports.

pub use std::io;
pub use std::fs;
pub use std::str;
pub use std::fmt;
pub use std::cmp;
pub use std::ptr;
pub use std::mem;
pub use std::ops;
pub use std::slice;
pub use std::result;
pub use std::option;

pub use std::path::Path;
pub use std::str::{FromStr};
pub use std::io::{Read,Write};
pub use std::hash::{Hash, Hasher};
pub use std::error::Error as StdError;

pub use std::ops::*;
pub use std::cmp::*;
pub use std::sync::Arc;
pub use std::collections::*;

pub use rustc_serialize::json::Json;
pub use rustc_serialize::base64::FromBase64;
pub use rustc_serialize::hex::{FromHex, FromHexError};

pub use heapsize::HeapSizeOf;
pub use itertools::Itertools;

pub use parking_lot::{Condvar, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};