//! Rlp serialization module

use std::fmt;
use std::cell::Cell;
use std::io::{Write, BufWriter};
use std::io::Error as IoError;
use std::error::Error as StdError;
use bytes::{ToBytes, FromBytes, FromBytesError};

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
pub struct ItemInfo {
    prefix_len: usize,
    value_len: usize
}

impl ItemInfo {
    pub fn new(prefix_len: usize, value_len: usize) -> ItemInfo {
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

/// shortcut function to encode a `T: Encodable` into a Rlp `Vec<u8>`
pub fn encode<E>(object: &E) -> Result<Vec<u8>, EncoderError> where E: Encodable {
    let mut ret: Vec<u8> = vec![];
    {
        let mut encoder = BasicEncoder::new(&mut ret);
        try!(object.encode(&mut encoder));
    }
    Ok(ret)
}

#[derive(Debug)]
pub enum EncoderError {
    IoError(IoError)
}

impl StdError for EncoderError {
    fn description(&self) -> &str { "encoder error" }
}

impl fmt::Display for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl From<IoError> for EncoderError {
    fn from(err: IoError) -> EncoderError { EncoderError::IoError(err) }
}

pub trait Encodable {
    fn encode<E>(&self, encoder: &mut E) -> Result<(), E::Error> where E: Encoder;
    fn item_info(&self) -> ItemInfo;
}

pub trait Encoder {
    type Error;

    fn emit_value<V>(&mut self, value: &V) -> Result<(), Self::Error> where V: Encodable + ToBytes;
    fn emit_array<V>(&mut self, array: &[V]) -> Result<(), Self::Error> where V: Encodable;
}

impl <T> Encodable for T where T: ToBytes {
    fn encode<E>(&self, encoder: &mut E) -> Result<(), E::Error> where E: Encoder {
        encoder.emit_value(self)
    }

    fn item_info(&self) -> ItemInfo {
        match self.to_bytes_len() {
            // just 0
            0 => ItemInfo::new(0, 1),
            // byte is its own encoding
            1 if self.first_byte().unwrap() < 0x80 => ItemInfo::new(0, 1),
            // (prefix + length), followed by the stirng
            len @ 1...55 => ItemInfo::new(1, len),
            // (prefix + length of length), followed by the length, followed by the value
            len => ItemInfo::new(1 + len.to_bytes_len(), len)
        }
    }
}

impl <'a, T> Encodable for &'a [T] where T: Encodable + 'a {
    fn encode<E>(&self, encoder: &mut E) -> Result<(), E::Error> where E: Encoder {
        encoder.emit_array(self)
    }

    fn item_info(&self) -> ItemInfo {
        let prefix_len = match self.len() {
            0...55 => 1,
            len => len.to_bytes_len()
        };

        let value_len = self.iter().fold(0, |acc, ref enc| { 
            let item = enc.item_info(); 
            acc + item.prefix_len + item.value_len 
        });

        ItemInfo::new(prefix_len, value_len)
    }
}

impl <T> Encodable for Vec<T> where T: Encodable {
    fn encode<E>(&self, encoder: &mut E) -> Result<(), E::Error> where E: Encoder {
        let r: &[T] = self.as_ref();
        r.encode(encoder)
    }

    fn item_info(&self) -> ItemInfo {
        let r: &[T] = self.as_ref();
        r.item_info()
    }
}

struct BasicEncoder<W> where W: Write {
    writer: BufWriter<W>
}

impl <W> BasicEncoder<W> where W: Write {
    pub fn new(writer: W) -> BasicEncoder<W> {
        BasicEncoder { writer: BufWriter::new(writer) }
    }
}

impl <W> Encoder for BasicEncoder<W> where W: Write {
    type Error = EncoderError;

    fn emit_value<V>(&mut self, value: &V) -> Result<(), Self::Error> where V: Encodable + ToBytes {
        let v = value.to_bytes();
        let bytes: &[u8] = v.as_ref();

        match bytes.len() {
            // just 0
            0 => { try!(self.writer.write(&[0x80u8])); },
            // byte is its own encoding
            1 if bytes[0] < 0x80 => { try!(self.writer.write(bytes)); },
            // (prefix + length), followed by the string
            len @ 1 ... 55 => {
                try!(self.writer.write(&[0x80u8 + len as u8]));
                try!(self.writer.write(bytes));
            }
            // (prefix + length of length), followed by the length, followd by the string
            len => {
                try!(self.writer.write(&[0xb7 + len.to_bytes_len() as u8]));
                try!(self.writer.write(&len.to_bytes()));
                try!(self.writer.write(bytes));
            }
        }
        Ok(())
    }

    fn emit_array<V>(&mut self, array: &[V]) -> Result<(), Self::Error> where V: Encodable {
        let item = array.item_info();

        match item.value_len {
            len @ 0...55 => { try!(self.writer.write(&[0xc0u8 + len as u8])); }
            len => {
                try!(self.writer.write(&[0x7fu8 + len.to_bytes_len() as u8]));
                try!(self.writer.write(&len.to_bytes()));
            }
        };
        
        for el in array.iter() {
            try!(el.encode(self));
        }

        Ok(())
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

    struct ETestPair<T>(T, Vec<u8>) where T: rlp::Encodable;

    fn run_encode_tests<T>(tests: Vec<ETestPair<T>>) where T: rlp::Encodable {
        for t in &tests {
            let res = rlp::encode(&t.0).unwrap();
            assert_eq!(res, &t.1[..]);
        }
    }

    #[test]
    fn encode_u8() {
        let tests = vec![
            ETestPair(0u8, vec![0x80u8]),
            ETestPair(15, vec![15]),
            ETestPair(55, vec![55]),
            ETestPair(56, vec![56]),
            ETestPair(0x7f, vec![0x7f]),
            ETestPair(0x80, vec![0x81, 0x80]),
            ETestPair(0xff, vec![0x81, 0xff]),
        ];
        run_encode_tests(tests);
    }

    #[test]
    fn encode_u16() {
        let tests = vec![
            ETestPair(0u16, vec![0x80u8]),
            ETestPair(0x100, vec![0x82, 0x01, 0x00]),
            ETestPair(0xffff, vec![0x82, 0xff, 0xff]),
        ];
        run_encode_tests(tests);
    }

    #[test]
    fn encode_u32() {
        let tests = vec![
            ETestPair(0u32, vec![0x80u8]),
            ETestPair(0x10000, vec![0x83, 0x01, 0x00, 0x00]),
            ETestPair(0xffffff, vec![0x83, 0xff, 0xff, 0xff]),
        ];
        run_encode_tests(tests);
    }

    #[test]
    fn encode_u64() {
        let tests = vec![
            ETestPair(0u64, vec![0x80u8]),
            ETestPair(0x1000000, vec![0x84, 0x01, 0x00, 0x00, 0x00]),
            ETestPair(0xFFFFFFFF, vec![0x84, 0xff, 0xff, 0xff, 0xff]),
        ];
        run_encode_tests(tests);
    }

    #[test]
    fn encode_str() {
        let tests = vec![
            ETestPair("cat", vec![0x83, b'c', b'a', b't']),
            ETestPair("dog", vec![0x83, b'd', b'o', b'g']),
            ETestPair("Marek", vec![0x85, b'M', b'a', b'r', b'e', b'k']),
            ETestPair("", vec![0x80]),
            ETestPair("Lorem ipsum dolor sit amet, consectetur adipisicing elit",
                     vec![0xb8, 0x38, b'L', b'o', b'r', b'e', b'm', b' ', b'i',
                    b'p', b's', b'u', b'm', b' ', b'd', b'o', b'l', b'o', b'r',
                    b' ', b's', b'i', b't', b' ', b'a', b'm', b'e', b't', b',',
                    b' ', b'c', b'o', b'n', b's', b'e', b'c', b't', b'e', b't',
                    b'u', b'r', b' ', b'a', b'd', b'i', b'p', b'i', b's', b'i',
                    b'c', b'i', b'n', b'g', b' ', b'e', b'l', b'i', b't'])
        ];
        run_encode_tests(tests);
    }

    #[test]
    fn encode_vector_u8() {
        let tests = vec![
            ETestPair(vec![], vec![0xc0]),
            ETestPair(vec![15u8], vec![0xc1, 0x0f]),
            ETestPair(vec![1, 2, 3, 7, 0xff], vec![0xc6, 1, 2, 3, 7, 0x81, 0xff]),
        ];
        run_encode_tests(tests);
    }

    #[test]
    fn encode_vector_u64() {
        let tests = vec![
            ETestPair(vec![], vec![0xc0]),
            ETestPair(vec![15u64], vec![0xc1, 0x0f]),
            ETestPair(vec![1, 2, 3, 7, 0xff], vec![0xc6, 1, 2, 3, 7, 0x81, 0xff]),
            ETestPair(vec![0xffffffff, 1, 2, 3, 7, 0xff], vec![0xcb, 0x84, 0xff, 0xff, 0xff, 0xff,  1, 2, 3, 7, 0x81, 0xff]),
        ];
        run_encode_tests(tests);
    }

    #[test]
    fn encode_vector_str() {
        let tests = vec![
            ETestPair(vec!["cat", "dog"], vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g'])
        ];
        run_encode_tests(tests);
    }

    #[test]
    fn encode_vector_of_vectors_str() {
        let tests = vec![
            ETestPair(vec![vec!["cat"]], vec![0xc5, 0xc4, 0x83, b'c', b'a', b't'])
        ];
        run_encode_tests(tests);
    }
}

