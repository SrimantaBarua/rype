//! Horizontal header table
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use super::error::*;
use super::types::{get_i16, get_u16};

pub(super) struct Hhea<'a>(pub(super) &'a [u8]);

impl<'a> std::fmt::Debug for Hhea<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Hhea")
            .field("ascender", &self._ascender().unwrap())
            .field("descender", &self._descender().unwrap())
            .field("num_of_h_metrics", &self._num_of_h_metrics().unwrap())
            .finish()
    }
}

impl<'a> Hhea<'a> {
    pub(super) fn _ascender(&self) -> Result<i16> {
        get_i16(self.0, 4)
    }

    pub(super) fn _descender(&self) -> Result<i16> {
        get_i16(self.0, 6)
    }

    pub(super) fn _num_of_h_metrics(&self) -> Result<u16> {
        get_u16(self.0, 34)
    }
}
