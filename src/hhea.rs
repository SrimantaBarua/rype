//! Horizontal header table
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use super::error::*;
use super::types::{get_i16_unchecked, get_u16_unchecked};

pub(super) struct Hhea<'a>(&'a [u8]);

impl<'a> std::fmt::Debug for Hhea<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Hhea")
            .field("ascender", &self._ascender())
            .field("descender", &self._descender())
            .field("num_of_h_metrics", &self.num_of_h_metrics())
            .finish()
    }
}

impl<'a> Hhea<'a> {
    pub(super) fn load(data: &[u8]) -> Result<Hhea> {
        if data.len() < 36 {
            Err(Error::Invalid)
        } else {
            Ok(Hhea(data))
        }
    }

    pub(super) fn _ascender(&self) -> i16 {
        get_i16_unchecked(self.0, 4)
    }

    pub(super) fn _descender(&self) -> i16 {
        get_i16_unchecked(self.0, 6)
    }

    pub(super) fn num_of_h_metrics(&self) -> u16 {
        get_u16_unchecked(self.0, 34)
    }
}
