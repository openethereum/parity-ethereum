//! Rlp serialization module

use std::fmt;
use std::cell::Cell;
use std::error::Error as StdError;
use bytes::{FromBytes, FromBytesError};

/// rlp container
#[derive(Debug)]
pub struct Rlp<'a>{
    bytes: &'a [u8],
    cache: Cell<OffsetCache>
}

/// rlp offset
#[derive(Copy, Clone, Debug)]
struct OffsetCache {
    index: usize,
    offset: usize
}

impl OffsetCache {
    fn new(index: usize, offset: usize) -> OffsetCache {
        OffsetCache { index: index, offset: offset }
    }
}

/// stores basic information about item
struct ItemInfo {
    prefix_len: usize,
    value_len: usize
}

impl ItemInfo {
    fn new(prefix_len: usize, value_len: usize) -> ItemInfo {
        ItemInfo { prefix_len: prefix_len, value_len: value_len }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DecoderError {
    FromBytesError(FromBytesError),
    RlpIsTooShort,
    RlpExpectedToBeArray,
    BadRlp,
}
impl StdError for DecoderError {
    fn description(&self) -> &str { "builder error" }
}

impl fmt::Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl From<FromBytesError> for DecoderError {
    fn from(err: FromBytesError) -> DecoderError { DecoderError::FromBytesError(err) }
}

impl <'a>Rlp<'a> {
    /// returns new instance of `Rlp`
    pub fn new(bytes: &'a[u8]) -> Rlp<'a> { 
        Rlp { 
            bytes: bytes,
            cache: Cell::new(OffsetCache::new(usize::max_value(), 0))
        }
    }

    /// get container subset at given index
    ///
    /// paren container caches searched position
    pub fn at(&self, index: usize) -> Result<Rlp<'a>, DecoderError> {
        if !self.is_array() {
            return Err(DecoderError::RlpExpectedToBeArray);
        }

        // move to cached position if it's index is less or equal to
        // current search index, otherwise move to beginning of array
        let c = self.cache.get();
        let (mut bytes, to_skip) = match c.index <= index {
            true => (try!(Rlp::consume(self.bytes, c.offset)), index - c.index),
            false => (try!(self.consume_array_prefix()), index)
        };

        // skip up to x items
        bytes = try!(Rlp::consume_items(bytes, to_skip));

        // update the cache
        self.cache.set(OffsetCache::new(index, self.bytes.len() - bytes.len()));

        // construct new rlp
        let found = try!(Rlp::item_info(bytes));
        Ok(Rlp::new(&bytes[0..found.prefix_len + found.value_len]))
    }

    /// returns true if rlp is an array
    pub fn is_array(&self) -> bool {
        self.bytes.len() > 0 && self.bytes[0] >= 0xc0
    }

    /// returns true if rlp is a value
    pub fn is_value(&self) -> bool {
        self.bytes.len() > 0 && self.bytes[0] <= 0xbf
    }

    /// returns rlp iterator
    pub fn iter(&'a self) -> RlpIterator<'a> {
        self.into_iter()
    }

    /// consumes first found prefix
    fn consume_array_prefix(&self) -> Result<&'a [u8], DecoderError> {
        let item = try!(Rlp::item_info(self.bytes));
        let bytes = try!(Rlp::consume(self.bytes, item.prefix_len));
        Ok(bytes)
    }

    /// consumes fixed number of items
    fn consume_items(bytes: &'a [u8], items: usize) -> Result<&'a [u8], DecoderError> {
        let mut result = bytes;
        for _ in 0..items {
            let i = try!(Rlp::item_info(result));
            result = try!(Rlp::consume(result, (i.prefix_len + i.value_len)));
        }
        Ok(result)
    }

    /// return first item info
    fn item_info(bytes: &[u8]) -> Result<ItemInfo, DecoderError> {
        let item = match bytes.first().map(|&x| x) {
            None => return Err(DecoderError::RlpIsTooShort),
            Some(0...0x7f) => ItemInfo::new(0, 1),
            Some(l @ 0x80...0xb7) => ItemInfo::new(1, l as usize - 0x80),
            Some(l @ 0xb8...0xbf) => {
                let len_of_len = l as usize - 0xb7;
                let prefix_len = 1 + len_of_len;
                let value_len = try!(usize::from_bytes(&bytes[1..prefix_len]));
                ItemInfo::new(prefix_len, value_len)
            }
            Some(l @ 0xc0...0xf7) => ItemInfo::new(1, l as usize - 0xc0),
            Some(l @ 0xf8...0xff) => {
                let len_of_len = l as usize - 0xf7;
                let prefix_len = 1 + len_of_len;
                let value_len = try!(usize::from_bytes(&bytes[1..prefix_len]));
                ItemInfo::new(prefix_len, value_len)
            },
            _ => return Err(DecoderError::BadRlp)
        };

        match item.prefix_len + item.value_len <= bytes.len() {
            true => Ok(item),
            false => Err(DecoderError::RlpIsTooShort)
        }
    }

    /// consumes slice prefix of length `len`
    fn consume(bytes: &'a [u8], len: usize) -> Result<&'a [u8], DecoderError> {
        match bytes.len() >= len {
            true => Ok(&bytes[len..]),
            false => Err(DecoderError::RlpIsTooShort)
        }
    }
}

/// non-consuming rlp iterator
pub struct RlpIterator<'a> {
    rlp: &'a Rlp<'a>,
    index: usize
}

impl <'a> IntoIterator for &'a Rlp<'a> {
    type Item = Rlp<'a>;
    type IntoIter = RlpIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        RlpIterator { rlp: self, index: 0 }
    }
}

impl <'a> Iterator for RlpIterator<'a> {
    type Item = Rlp<'a>;

    fn next(&mut self) -> Option<Rlp<'a>> {
        let index = self.index;
        let result = self.rlp.at(index).ok();
        self.index += 1;
        result
    }
}

#[cfg(test)]
mod tests {
    use rlp;
    use rlp::Rlp;

    #[test]
    fn rlp_at() {
        let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
        {
            let rlp = Rlp::new(&data);
            assert!(rlp.is_array());
           
            let cat = rlp.at(0).unwrap();
            assert!(cat.is_value());
            assert_eq!(cat.bytes, &[0x83, b'c', b'a', b't']);
            
            let dog = rlp.at(1).unwrap();
            assert!(dog.is_value());
            assert_eq!(dog.bytes, &[0x83, b'd', b'o', b'g']);

            let cat_again = rlp.at(0).unwrap();
            assert!(cat_again.is_value());
            assert_eq!(cat_again.bytes, &[0x83, b'c', b'a', b't']);
        }
    }

    #[test]
    fn rlp_at_err() {
        let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o'];
        {
            let rlp = Rlp::new(&data);
            assert!(rlp.is_array());

            let cat_err = rlp.at(0).unwrap_err();
            assert_eq!(cat_err, rlp::DecoderError::RlpIsTooShort);

            let dog_err = rlp.at(1).unwrap_err();
            assert_eq!(dog_err, rlp::DecoderError::RlpIsTooShort);
        }
    }

    #[test]
    fn rlp_iter() {
        let data = vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'];
        {
            let rlp = Rlp::new(&data);
            let mut iter = rlp.iter();

            let cat = iter.next().unwrap();
            assert!(cat.is_value());
            assert_eq!(cat.bytes, &[0x83, b'c', b'a', b't']);

            let dog = iter.next().unwrap();
            assert!(dog.is_value());
            assert_eq!(dog.bytes, &[0x83, b'd', b'o', b'g']);

            let none = iter.next();
            assert!(none.is_none());

            let cat_again = rlp.at(0).unwrap();
            assert!(cat_again.is_value());
            assert_eq!(cat_again.bytes, &[0x83, b'c', b'a', b't']);
        }
    }
}

