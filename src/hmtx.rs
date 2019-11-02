//! Horizontal metrics table
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use super::error::*;
use super::types::{get_i16_unchecked, get_u16_unchecked, GlyphID};

pub(super) struct Hmtx<'a> {
    num_glyphs: usize,
    num_of_h_metrics: usize,
    data: &'a [u8],
}

impl<'a> Hmtx<'a> {
    pub(super) fn load(data: &[u8], num_glyphs: usize, num_of_h_metrics: usize) -> Result<Hmtx> {
        if data.len() < num_of_h_metrics * 2 + num_glyphs * 2 || num_of_h_metrics == 0 {
            Err(Error::Invalid)
        } else {
            Ok(Hmtx {
                num_glyphs: num_glyphs,
                num_of_h_metrics: num_of_h_metrics,
                data: data,
            })
        }
    }

    pub(super) fn get_metrics(&self, glyph_id: GlyphID) -> Result<(u16, i16)> {
        if (glyph_id.0 as usize) >= self.num_glyphs {
            Err(Error::GlyphIDOutOfBounds)
        } else {
            let glyph_off = glyph_id.0 as usize * 2;
            if (glyph_id.0 as usize) < self.num_of_h_metrics {
                Ok((
                    get_u16_unchecked(self.data, glyph_off),
                    get_i16_unchecked(self.data, glyph_off + 2),
                ))
            } else {
                Ok((
                    get_u16_unchecked(self.data, (self.num_of_h_metrics - 1) * 2),
                    get_i16_unchecked(self.data, glyph_off + 2),
                ))
            }
        }
    }
}
