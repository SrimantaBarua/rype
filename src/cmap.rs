//! Character to glyph mapping table
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use super::error::*;
use super::types::{get_u16, get_u16_unchecked, get_u32, get_u32_unchecked, GlyphID};

/// A cmap encoding record
#[derive(Clone)]
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
pub(super) struct Cmap<'a> {
    subtables: Vec<Subtable<'a>>,
    active: Option<Subtable<'a>>,
}

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
        let mut active = None;
        let mut enc_rec_off = 4;
        for _ in 0..num_tables {
            let platform_id = get_u16_unchecked(data, enc_rec_off);
            let encoding_id = get_u16_unchecked(data, enc_rec_off + 2);
            let offset = get_u32_unchecked(data, enc_rec_off + 4) as usize;
            if offset >= data.len() {
                return Err(Error::Invalid);
            }
            let subtable = Subtable {
                platform_id: platform_id,
                encoding_id: encoding_id,
                data: &data[offset..],
            };
            if platform_id == 3 {
                if encoding_id == 10 {
                    active = Some(subtable.clone());
                } else if active.is_none() && encoding_id == 1 {
                    active = Some(subtable.clone());
                }
            }
            subtables.push(subtable);
            enc_rec_off += 8;
        }
        Ok(Cmap {
            subtables: subtables,
            active: active,
        })
    }

    pub(super) fn subtables(&self) -> std::slice::Iter<Subtable> {
        self.subtables.iter()
    }

    pub(super) fn set_active_subtable(&mut self, subtable: &Subtable<'a>) {
        self.active = Some(subtable.clone())
    }

    // TODO: We only handle formats 4 and 12 for now
    pub(super) fn get_glyph_id(&self, codepoint: u32) -> Result<GlyphID> {
        if let Some(active) = &self.active {
            match active.format() {
                Ok(4) => {
                    let segcnt_2 = get_u16(active.data, 6)? as usize;
                    if active.data.len() < 16 + segcnt_2 * 4 {
                        return Err(Error::Invalid);
                    }
                    for off in (0..segcnt_2).step_by(2) {
                        let end = get_u16_unchecked(active.data, 14 + off) as u32;
                        if codepoint > end {
                            continue;
                        }
                        let start = get_u16_unchecked(active.data, 16 + segcnt_2 + off) as u32;
                        if codepoint < start {
                            break;
                        }
                        let delta = get_u16_unchecked(active.data, 16 + segcnt_2 * 2 + off) as u32;
                        let range = get_u16_unchecked(active.data, 16 + segcnt_2 * 3 + off) as u32;
                        if range == 0 {
                            return Ok(GlyphID((codepoint + delta) & 0xffff));
                        } else {
                            let gloff = (range + (codepoint - start) * 2) as usize
                                + 16
                                + segcnt_2 * 3
                                + off;
                            return Ok(GlyphID(
                                (get_u16(active.data, gloff)? as u32 + delta) & 0xffff,
                            ));
                        }
                    }
                    Ok(GlyphID(0))
                }
                Ok(12) => {
                    let num_groups = get_u32(active.data, 12)? as usize;
                    if active.data.len() < 16 + 12 * num_groups {
                        return Err(Error::Invalid);
                    }
                    let mut off = 16;
                    for _ in 0..num_groups {
                        let start = get_u32_unchecked(active.data, off);
                        if codepoint < start {
                            break;
                        }
                        let end = get_u32_unchecked(active.data, off + 4);
                        if codepoint > end {
                            off += 12;
                            continue;
                        }
                        let glyph = get_u32_unchecked(active.data, off + 8);
                        return Ok(GlyphID(codepoint - start + glyph));
                    }
                    return Ok(GlyphID(0));
                }
                Err(_) => Err(Error::Invalid),
                _ => Ok(GlyphID(0)),
            }
        } else {
            Err(Error::NoCharmap)
        }
    }
}
