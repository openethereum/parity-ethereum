//! To/From Bytes conversation for basic types
//!
//! Types implementing `ToBytes` and `FromBytes` traits
//! can be easily converted to and from bytes
//! 
//! # Examples
//!

use std::fmt;
use std::error::Error as StdError;

/// TODO: optimise some conversations
pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
    fn to_bytes_len(&self) -> usize { self.to_bytes().len() }
    fn first_byte(&self) -> Option<u8> { self.to_bytes().first().map(|&x| { x })}
}

impl <'a> ToBytes for &'a str {
    fn to_bytes(&self) -> Vec<u8> {
        From::from(*self)
    }
    
    fn to_bytes_len(&self) -> usize { self.len() }
}

impl ToBytes for String {
    fn to_bytes(&self) -> Vec<u8> {
        let s: &str = self.as_ref();
        From::from(s)
    }
    
    fn to_bytes_len(&self) -> usize { self.len() }
}

impl ToBytes for u8 {
    fn to_bytes(&self) -> Vec<u8> {
        match *self {
            0 => vec![],
            _ => vec![*self]
        }
    }

    fn to_bytes_len(&self) -> usize {
        match *self {
            0 => 0,
            _ => 1
        }
    }
    fn first_byte(&self) -> Option<u8> { 
        match *self {
            0 => None,
            _ => Some(*self) 
        }
    }
}

impl ToBytes for u64 {
    fn to_bytes(&self) -> Vec<u8> {
        let mut res= vec![];
        let count = self.to_bytes_len();
        for i in 0..count {
            let j = count - 1 - i;
            res.push((*self >> (j * 8)) as u8);
        }
        res
    }

    fn to_bytes_len(&self) -> usize { 8 - self.leading_zeros() as usize / 8 }
}

macro_rules! impl_map_to_bytes {
    ($from: ident, $to: ty) => {
        impl ToBytes for $from {
            fn to_bytes(&self) -> Vec<u8> { (*self as $to).to_bytes() }
            fn to_bytes_len(&self) -> usize { (*self as $to).to_bytes_len() }
        }
    }
}

impl_map_to_bytes!(usize, u64);
impl_map_to_bytes!(u16, u64);
impl_map_to_bytes!(u32, u64);

#[derive(Debug, PartialEq, Eq)]
pub enum FromBytesError {
    UnexpectedEnd
}

impl StdError for FromBytesError {
    fn description(&self) -> &str { "from_bytes error" }
}

impl fmt::Display for FromBytesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

pub type FromBytesResult<T> = Result<T, FromBytesError>;

/// implements "Sized", so the compiler can deducate the size
/// of the return type
pub trait FromBytes: Sized {
    fn from_bytes(bytes: &[u8]) -> FromBytesResult<Self>;
}

impl FromBytes for String {
    fn from_bytes(bytes: &[u8]) -> FromBytesResult<String> {
        Ok(::std::str::from_utf8(bytes).unwrap().to_string())
    }
}

impl FromBytes for u8 {
    fn from_bytes(bytes: &[u8]) -> FromBytesResult<u8> {
        match bytes.len() {
            0 => Ok(0),
            _ => Ok(bytes[0])
        }
    }
}

impl FromBytes for u64 {
    fn from_bytes(bytes: &[u8]) -> FromBytesResult<u64> {
        match bytes.len() {
            0 => Ok(0),
            l => {
                let mut res = 0u64;
                for i in 0..l {
                    let shift = (l - 1 - i) * 8;
                    res = res + ((bytes[i] as u64) << shift);
                }
                Ok(res)
            }
        }
    }
}

macro_rules! impl_map_from_bytes {
    ($from: ident, $to: ident) => {
        impl FromBytes for $from {
            fn from_bytes(bytes: &[u8]) -> FromBytesResult<$from> {
                $to::from_bytes(bytes).map(| x | { x as $from })
            }
        }
    }
}

impl_map_from_bytes!(usize, u64);
impl_map_from_bytes!(u16, u64);
impl_map_from_bytes!(u32, u64);
