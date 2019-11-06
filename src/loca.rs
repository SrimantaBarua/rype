//! Glyph index to location

use super::error::*;
use super::head::IdxToLocFmt;
use super::types::{get_u16_unchecked, get_u32_unchecked, GlyphID};

/// Handle to loca table
pub(super) struct Loca<'a> {
    num_glyphs: usize,
    idx_to_loc_fmt: IdxToLocFmt,
    data: &'a [u8],
}

impl<'a> std::fmt::Debug for Loca<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Loca")
            .field("num_glyphs", &self.num_glyphs)
            .field("index_to_loc_fmt", &self.idx_to_loc_fmt)
            .finish()
    }
}

impl<'a> Loca<'a> {
    /// Check if we have enough data
    pub(super) fn load(data: &[u8], num_glyphs: usize, fmt: IdxToLocFmt) -> Result<Loca> {
        match fmt {
            IdxToLocFmt::Off16 => {
                if data.len() < num_glyphs * 2 {
                    return Err(Error::Invalid);
                }
            }
            IdxToLocFmt::Off32 => {
                if data.len() < num_glyphs * 4 {
                    return Err(Error::Invalid);
                }
            }
        }
        Ok(Loca {
            num_glyphs: num_glyphs,
            idx_to_loc_fmt: fmt,
            data: data,
        })
    }

    /// Get offset into glyf table for glyph ID
    pub(super) fn get_offset(&self, id: GlyphID) -> Result<usize> {
        if id.0 as usize >= self.num_glyphs {
            return Err(Error::GlyphIDOutOfBounds);
        }
        Ok(match self.idx_to_loc_fmt {
            IdxToLocFmt::Off16 => (get_u16_unchecked(self.data, id.0 as usize * 2) as usize) * 2,
            IdxToLocFmt::Off32 => get_u32_unchecked(self.data, id.0 as usize * 4) as usize,
        })
    }
}
