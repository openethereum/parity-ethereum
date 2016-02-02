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
pub use std::sync::*;
pub use std::cell::*;
pub use std::collections::*;

pub use rustc_serialize::json::Json;
pub use rustc_serialize::base64::FromBase64;
pub use rustc_serialize::hex::{FromHex, FromHexError};

pub use heapsize::HeapSizeOf;
pub use itertools::Itertools;
