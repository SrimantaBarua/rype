//! Character to glyph mapping table
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use super::error::*;
use super::types::{get_u16, get_u16_unchecked, get_u32_unchecked};

/// A cmap encoding record
pub(super) struct Subtable<'a> {
    pub(super) platform_id: u16,
    pub(super) encoding_id: u16,
    data: &'a [u8],
}

impl<'a> std::fmt::Debug for Subtable<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Subtable")
            .field("platform_id", &self.platform_id)
            .field("encoding_id", &self.encoding_id)
            .field("format", &self.format())
            .finish()
    }
}

impl<'a> Subtable<'a> {
    pub(super) fn format(&self) -> Result<u16> {
        get_u16(self.data, 0)
    }
}

/// Handle to cmap table
#[derive(Debug)]
pub(super) struct Cmap<'a>(Vec<Subtable<'a>>);

impl<'a> Cmap<'a> {
    pub(super) fn load(data: &[u8]) -> Result<Cmap> {
        if data.len() < 4 {
            return Err(Error::Invalid);
        }
        let num_tables = get_u16_unchecked(data, 2) as usize;
        if data.len() < 4 + num_tables * 8 {
            return Err(Error::Invalid);
        }
        let mut subtables = Vec::with_capacity(num_tables);
        let mut enc_rec_off = 4;
        for _ in 0..num_tables {
            let platform_id = get_u16_unchecked(data, enc_rec_off);
            let encoding_id = get_u16_unchecked(data, enc_rec_off + 2);
            let offset = get_u32_unchecked(data, enc_rec_off + 4) as usize;
            if offset >= data.len() {
                return Err(Error::Invalid);
            }
            subtables.push(Subtable {
                platform_id: platform_id,
                encoding_id: encoding_id,
                data: &data[offset..],
            });
            enc_rec_off += 8;
        }
        Ok(Cmap(subtables))
    }

    pub(super) fn subtables(&self) -> std::slice::Iter<Subtable> {
        self.0.iter()
    }
}
