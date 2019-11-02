//! Font header table
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use super::error::*;
use super::types::{get_i16, get_u16};

pub(super) struct Head<'a>(pub(super) &'a [u8]);

impl<'a> std::fmt::Debug for Head<'a> {
    fn fmt(&self, f: &mut  std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Head")
            .field("units_per_em", &self._units_per_em().unwrap())
            .field("xmin", &self._xmin().unwrap())
            .field("ymin", &self._ymin().unwrap())
            .field("xmax", &self._xmax().unwrap())
            .field("ymax", &self._ymax().unwrap())
            .field("lowest_rec_ppem", &self._lowest_rec_ppem().unwrap())
            .field("index_to_loc_format", &self._idx_to_loc_fmt().unwrap())
            .finish()
    }
}

impl<'a> Head<'a> {
    pub(super) fn _units_per_em(&self) -> Result<u16> {
        get_u16(self.0, 18)
    }

    pub(super) fn _xmin(&self) -> Result<i16> {
        get_i16(self.0, 36)
    }

    pub(super) fn _ymin(&self) -> Result<i16> {
        get_i16(self.0, 38)
    }

    pub(super) fn _xmax(&self) -> Result<i16> {
        get_i16(self.0, 40)
    }

    pub(super) fn _ymax(&self) -> Result<i16> {
        get_i16(self.0, 42)
    }

    pub(super) fn _lowest_rec_ppem(&self) -> Result<u16> {
        get_u16(self.0, 46)
    }

    pub(super) fn _idx_to_loc_fmt(&self) -> Result<i16> {
        get_i16(self.0, 50)
    }
}
