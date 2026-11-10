//! Bounded binary value codec conforming to CPPlayer KMP SDK Wire.kt.
//!
//! Enforces:
//! - Little-endian integer encoding
//! - MAX_BYTES = 4 MiB limit
//! - MAX_ITEMS = 500 items per list limit
//! - Valid UTF-8 string encoding

use std::fmt;

pub const MAX_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_ITEMS: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    Truncated,
    TrailingData,
    InvalidBoolean,
    InvalidUtf8,
    ByteLimitExceeded,
    ItemLimitExceeded,
    InvalidLength,
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WireError::Truncated => write!(f, "Truncated wire value"),
            WireError::TrailingData => write!(f, "Trailing wire data"),
            WireError::InvalidBoolean => write!(f, "Invalid wire boolean"),
            WireError::InvalidUtf8 => write!(f, "Invalid UTF-8 in wire string"),
            WireError::ByteLimitExceeded => write!(f, "Wire byte limit exceeded (max 4MB)"),
            WireError::ItemLimitExceeded => write!(f, "Wire item limit exceeded (max 500)"),
            WireError::InvalidLength => write!(f, "Invalid wire length"),
        }
    }
}

impl std::error::Error for WireError {}

/// Writer for encoding typed SDK records to the binary wire protocol.
pub struct WireWriter {
    out: Vec<u8>,
}

impl Default for WireWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl WireWriter {
    pub fn new() -> Self {
        Self {
            out: Vec::with_capacity(256),
        }
    }

    fn reserve(&mut self, count: usize) -> Result<(), WireError> {
        if self.out.len().saturating_add(count) > MAX_BYTES {
            return Err(WireError::ByteLimitExceeded);
        }
        self.out.reserve(count);
        Ok(())
    }

    pub fn int(&mut self, value: i32) {
        if self.reserve(4).is_ok() {
            self.out.extend_from_slice(&value.to_le_bytes());
        }
    }

    pub fn long(&mut self, value: i64) {
        if self.reserve(8).is_ok() {
            self.out.extend_from_slice(&value.to_le_bytes());
        }
    }

    pub fn bool(&mut self, value: bool) {
        if self.reserve(1).is_ok() {
            self.out.push(if value { 1 } else { 0 });
        }
    }

    pub fn bytes(&mut self, value: &[u8]) {
        if value.len() > MAX_BYTES {
            self.int(0);
            return;
        }
        self.int(value.len() as i32);
        if self.reserve(value.len()).is_ok() {
            self.out.extend_from_slice(value);
        }
    }

    pub fn string(&mut self, value: &str) {
        self.bytes(value.as_bytes());
    }

    pub fn optional<T: ?Sized, F>(&mut self, value: Option<&T>, mut write_fn: F)
    where
        F: FnMut(&mut Self, &T),
    {
        self.bool(value.is_some());
        if let Some(v) = value {
            write_fn(self, v);
        }
    }

    pub fn list<T, F>(&mut self, value: &[T], mut write_fn: F)
    where
        F: FnMut(&mut Self, &T),
    {
        let count = value.len().min(MAX_ITEMS);
        self.int(count as i32);
        for item in &value[..count] {
            write_fn(self, item);
        }
    }

    pub fn finish(self) -> Vec<u8> {
        self.out
    }
}

/// Reader for decoding typed SDK records from the binary wire protocol.
pub struct WireReader<'a> {
    input: &'a [u8],
    offset: usize,
}

impl<'a> WireReader<'a> {
    pub fn new(input: &'a [u8]) -> Result<Self, WireError> {
        if input.len() > MAX_BYTES {
            return Err(WireError::ByteLimitExceeded);
        }
        Ok(Self { input, offset: 0 })
    }

    fn next(&mut self) -> Result<u8, WireError> {
        if self.offset < self.input.len() {
            let b = self.input[self.offset];
            self.offset += 1;
            Ok(b)
        } else {
            Err(WireError::Truncated)
        }
    }

    pub fn int(&mut self) -> Result<i32, WireError> {
        if self.offset + 4 <= self.input.len() {
            let slice = &self.input[self.offset..self.offset + 4];
            self.offset += 4;
            Ok(i32::from_le_bytes(slice.try_into().unwrap()))
        } else {
            Err(WireError::Truncated)
        }
    }

    pub fn long(&mut self) -> Result<i64, WireError> {
        if self.offset + 8 <= self.input.len() {
            let slice = &self.input[self.offset..self.offset + 8];
            self.offset += 8;
            Ok(i64::from_le_bytes(slice.try_into().unwrap()))
        } else {
            Err(WireError::Truncated)
        }
    }

    pub fn bool(&mut self) -> Result<bool, WireError> {
        match self.next()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(WireError::InvalidBoolean),
        }
    }

    pub fn bytes(&mut self) -> Result<Vec<u8>, WireError> {
        let length = self.int()?;
        if length < 0 {
            return Err(WireError::InvalidLength);
        }
        let len = length as usize;
        if len > self.input.len() - self.offset {
            return Err(WireError::InvalidLength);
        }
        let slice = &self.input[self.offset..self.offset + len];
        self.offset += len;
        Ok(slice.to_vec())
    }

    pub fn string(&mut self) -> Result<String, WireError> {
        let raw = self.bytes()?;
        String::from_utf8(raw).map_err(|_| WireError::InvalidUtf8)
    }

    pub fn optional<T, F>(&mut self, mut read_fn: F) -> Result<Option<T>, WireError>
    where
        F: FnMut(&mut Self) -> Result<T, WireError>,
    {
        if self.bool()? {
            Ok(Some(read_fn(self)?))
        } else {
            Ok(None)
        }
    }

    pub fn list<T, F>(&mut self, mut read_fn: F) -> Result<Vec<T>, WireError>
    where
        F: FnMut(&mut Self) -> Result<T, WireError>,
    {
        let count = self.int()?;
        if count < 0 || count as usize > MAX_ITEMS {
            return Err(WireError::ItemLimitExceeded);
        }
        let mut result = Vec::with_capacity(count as usize);
        for _ in 0..count {
            result.push(read_fn(self)?);
        }
        Ok(result)
    }

    pub fn finish(&self) -> Result<(), WireError> {
        if self.offset == self.input.len() {
            Ok(())
        } else {
            Err(WireError::TrailingData)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wire_round_trip_primitives() {
        let mut w = WireWriter::new();
        w.int(42);
        w.long(9_007_199_254_740_991);
        w.bool(true);
        w.bool(false);
        w.string("Hello, 网易云音乐!");
        w.bytes(&[1, 2, 3, 4, 5]);

        let bytes = w.finish();
        let mut r = WireReader::new(&bytes).unwrap();
        assert_eq!(r.int().unwrap(), 42);
        assert_eq!(r.long().unwrap(), 9_007_199_254_740_991);
        assert!(r.bool().unwrap());
        assert!(!r.bool().unwrap());
        assert_eq!(r.string().unwrap(), "Hello, 网易云音乐!");
        assert_eq!(r.bytes().unwrap(), vec![1, 2, 3, 4, 5]);
        r.finish().unwrap();
    }

    #[test]
    fn test_wire_optional_and_list() {
        let mut w = WireWriter::new();
        w.optional(Some(&"present"), |writer, s| writer.string(s));
        w.optional(None::<&str>, |writer, s| writer.string(s));
        w.list(&["a", "b", "c"], |writer, s| writer.string(s));

        let bytes = w.finish();
        let mut r = WireReader::new(&bytes).unwrap();
        assert_eq!(r.optional(|reader| reader.string()).unwrap(), Some("present".to_string()));
        assert_eq!(r.optional(|reader| reader.string()).unwrap(), None);
        assert_eq!(
            r.list(|reader| reader.string()).unwrap(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        r.finish().unwrap();
    }

    #[test]
    fn test_wire_error_handling() {
        assert!(WireReader::new(&[]).unwrap().int().is_err());
        assert!(WireReader::new(&[2]).unwrap().bool().is_err());
        assert!(WireReader::new(&[0]).unwrap().finish().is_err()); // Trailing data
        let invalid_utf8 = vec![1, 0, 0, 0, 0xFF];
        assert!(WireReader::new(&invalid_utf8).unwrap().string().is_err());
    }
}
