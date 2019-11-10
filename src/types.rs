//! Types and accessors
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use rster::Point;
use crate::error::*;

/// Get u8 checked
pub(super) fn get_u8(data: &[u8], off: usize) -> Result<u8> {
    if off + 1 > data.len() {
        Err(Error::Invalid)
    } else {
        Ok(data[off])
    }
}

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

/// Affine transformation matrix
#[derive(Clone, Debug)]
pub(super) struct Affine {
    a0: f32,
    b0: f32,
    c0: f32,
    a1: f32,
    b1: f32,
    c1: f32,
}

impl Affine {
    pub(super) fn apply_point(&self, p: &Point) -> Point {
        Point {
            x: self.a0 * p.x + self.b0 * p.y + self.c0,
            y: self.a1 * p.x + self.b1 * p.y + self.c1,
        }
    }

    pub(super) fn ident() -> Affine {
        Affine {
            a0: 1.0,
            b0: 0.0,
            c0: 0.0,
            a1: 0.0,
            b1: 1.0,
            c1: 0.0,
        }
    }

    pub(super) fn scaling(x: f32, y: f32) -> Affine {
        Affine {
            a0: x,
            b0: 0.0,
            c0: 0.0,
            a1: 0.0,
            b1: y,
            c1: 0.0,
        }
    }

    pub(super) fn translation(x: f32, y: f32) -> Affine {
        Affine {
            a0: 1.0,
            b0: 0.0,
            c0: x,
            a1: 0.0,
            b1: 1.0,
            c1: y,
        }
    }

    pub(super) fn rotation(angle: f32) -> Affine {
        let cos = angle.cos();
        let sin = angle.sin();
        Affine {
            a0: cos,
            b0: -sin,
            c0: 0.0,
            a1: sin,
            b1: cos,
            c1: 0.0,
        }
    }

    pub(super) fn scaled(self, x: f32, y: f32) -> Affine {
        Affine {
            a0: self.a0 * x,
            b0: self.b0 * x,
            c0: self.c0 * x,
            a1: self.a1 * y,
            b1: self.b1 * y,
            c1: self.c1 * y,
        }
    }

    pub(super) fn translated(self, x: f32, y: f32) -> Affine {
        Affine {
            a0: self.a0,
            b0: self.b0,
            c0: self.c0 * x,
            a1: self.a1,
            b1: self.b1,
            c1: self.c1 * y,
        }
    }

    pub(super) fn rotated(self, angle: f32) -> Affine {
        let cos = angle.cos();
        let sin = angle.sin();
        Affine {
            a0: self.a0 * cos - self.a1 * sin,
            b0: self.b0 * cos - self.b1 * sin,
            c0: self.c0 * cos - self.c1 * sin,
            a1: self.a0 * sin + self.a1 * cos,
            b1: self.a0 * sin + self.a1 * cos,
            c1: self.a0 * sin + self.a1 * cos,
        }
    }
}
