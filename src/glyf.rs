//! TrueType glyph data table
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use super::error::*;
use super::types::get_i16_unchecked;

pub(super) struct Glyf<'a>(pub(super) &'a [u8]);

impl<'a> std::fmt::Debug for Glyf<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Glyf")
    }
}

impl<'a> Glyf<'a> {
    pub(super) fn glyph(&self, offset: usize) -> Result<TTGlyph<'a>> {
        if offset + 10 > self.0.len() {
            return Err(Error::Invalid);
        }
        let glyf_data = &self.0[offset..];
        let num_contours = get_i16_unchecked(glyf_data, 0);
        if num_contours < 0 {
            Ok(TTGlyph::Composite(glyf_data))
        } else {
            Ok(TTGlyph::Simple(glyf_data))
        }
    }
}

pub(super) enum TTGlyph<'a> {
    Simple(&'a [u8]),
    Composite(&'a [u8]),
}

impl<'a> std::fmt::Debug for TTGlyph<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TTGlyph::Simple(_) => write!(f, "TTGlyph::Simple"),
            TTGlyph::Composite(_) =>  write!(f, "TTGlyph::Composite")
        }
    }
}

