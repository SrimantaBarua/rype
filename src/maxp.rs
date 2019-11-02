//! Maximum profile
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use super::error::*;
use super::types::get_u16_unchecked;

pub(super) struct Maxp<'a>(&'a [u8]);

impl<'a> std::fmt::Debug for Maxp<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Maxp")
            .field("num_glyphs", &self.num_glyphs())
            .finish()
    }
}

impl<'a> Maxp<'a> {
    pub(super) fn load(data: &[u8]) -> Result<Maxp> {
        if data.len() < 6 {
            Err(Error::Invalid)
        } else {
            Ok(Maxp(data))
        }
    }

    pub(super) fn num_glyphs(&self) -> u16 {
        get_u16_unchecked(self.0, 4)
    }
}
