//! Types and accessors
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use crate::error::*;

/// Get big-endian u16
pub(super) fn get_u16(data: &[u8], off: usize) -> Result<u16> {
    if off + 2 > data.len() {
        Err(Error::Invalid)
    } else {
        Ok(((data[off] as u16) << 8) | (data[off + 1] as u16))
    }
}

/// Get big-endian i16
pub(super) fn get_i16(data: &[u8], off: usize) -> Result<i16> {
    if off + 2 > data.len() {
        Err(Error::Invalid)
    } else {
        Ok(((data[off] as i16) << 8) | (data[off + 1] as i16))
    }
}

/// Get big-endian u32
pub(super) fn get_u32(data: &[u8], off: usize) -> Result<u32> {
    if off + 4 > data.len() {
        Err(Error::Invalid)
    } else {
        Ok(((data[off] as u32) << 24)
            | ((data[off + 1] as u32) << 16)
            | ((data[off + 2] as u32) << 8)
            | (data[off + 3] as u32))
    }
}

/// Get big-endian u16 without checking. Could panic
pub(super) fn get_u16_unchecked(data: &[u8], off: usize) -> u16 {
    ((data[off] as u16) << 8) | (data[off + 1] as u16)
}

/// Get big-endian i16 without checking. Could panic
pub(super) fn get_i16_unchecked(data: &[u8], off: usize) -> i16 {
    ((data[off] as i16) << 8) | (data[off + 1] as i16)
}

/// Get big-endian u32 without checking. Could panic
pub(super) fn get_u32_unchecked(data: &[u8], off: usize) -> u32 {
    ((data[off] as u32) << 24)
        | ((data[off + 1] as u32) << 16)
        | ((data[off + 2] as u32) << 8)
        | (data[off + 3] as u32)
}
/// OpenType "tag"s are used to uniquely identify resources like tables etc.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(super) struct Tag(pub(super) u32);

impl Tag {
    /// Create tag from string representation
    pub(super) fn from_str(s: &str) -> Tag {
        Tag(get_u32(s.as_bytes(), 0).unwrap())
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let a = (self.0 >> 24) as u8 as char;
        let b = ((self.0 >> 16) & 0xff) as u8 as char;
        let c = ((self.0 >> 8) & 0xff) as u8 as char;
        let d = (self.0 & 0xff) as u8 as char;
        write!(f, "{}{}{}{}", a, b, c, d)
    }
}

/// Get tag from big-endian data
pub(super) fn get_tag(data: &[u8], off: usize) -> Result<Tag> {
    get_u32(data, off).map(|n| Tag(n))
}

/// Glyph ID that is available to consumers of the library
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct GlyphID(pub(super) u32);
