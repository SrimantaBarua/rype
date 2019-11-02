//! Font header table
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use super::error::*;
use super::types::{get_i16_unchecked, get_u16_unchecked};

pub(super) struct Head<'a>(&'a [u8]);

impl<'a> std::fmt::Debug for Head<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Head")
            .field("units_per_em", &self._units_per_em())
            .field("xmin", &self._xmin())
            .field("ymin", &self._ymin())
            .field("xmax", &self._xmax())
            .field("ymax", &self._ymax())
            .field("lowest_rec_ppem", &self._lowest_rec_ppem())
            .field("index_to_loc_format", &self._idx_to_loc_fmt())
            .finish()
    }
}

impl<'a> Head<'a> {
    pub(super) fn load(data: &[u8]) -> Result<Head> {
        if data.len() < 54 {
            Err(Error::Invalid)
        } else {
            Ok(Head(data))
        }
    }

    pub(super) fn _units_per_em(&self) -> u16 {
        get_u16_unchecked(self.0, 18)
    }

    pub(super) fn _xmin(&self) -> i16 {
        get_i16_unchecked(self.0, 36)
    }

    pub(super) fn _ymin(&self) -> i16 {
        get_i16_unchecked(self.0, 38)
    }

    pub(super) fn _xmax(&self) -> i16 {
        get_i16_unchecked(self.0, 40)
    }

    pub(super) fn _ymax(&self) -> i16 {
        get_i16_unchecked(self.0, 42)
    }

    pub(super) fn _lowest_rec_ppem(&self) -> u16 {
        get_u16_unchecked(self.0, 46)
    }

    pub(super) fn _idx_to_loc_fmt(&self) -> i16 {
        get_i16_unchecked(self.0, 50)
    }
}
