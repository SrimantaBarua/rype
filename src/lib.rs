//! Rust OpenType font rasterization, shaping, and layout library
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use std::collections::HashMap;

pub mod error;
use error::*;

mod types;
pub use types::GlyphID;
use types::*;

mod cmap;
mod head;
mod hhea;
mod hmtx;
mod maxp;

/// A face within the OpenType font file. This face alone cannot be used to render glyphs -
/// it must be scaled first
pub struct Face<'a> {
    tables: HashMap<Tag, &'a [u8]>,
    head: head::Head<'a>,
    hhea: hhea::Hhea<'a>,
    maxp: maxp::Maxp<'a>,
    hmtx: hmtx::Hmtx<'a>,
    cmap: cmap::Cmap<'a>,
}

impl<'a> std::fmt::Debug for Face<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Face")
            .field("head", &self.head)
            .field("hhea", &self.hhea)
            .field("maxp", &self.maxp)
            .field("cmap", &self.cmap)
            .finish()
    }
}

impl<'a> Face<'a> {
    /// Load face information from data. The `offset` provided is the offset from the beginning
    /// of the file to the Offset Table for the face
    fn load(data: &[u8], offset: usize) -> Result<Face> {
        //let version = get_tag(data, offset)?;
        let num_tables = get_u16(data, offset + 4)? as usize;
        let mut record_off = offset + 12;
        let mut tables = HashMap::new();
        for _ in 0..num_tables {
            let tag = get_tag(data, record_off)?;
            let table_off = get_u32(data, record_off + 8)? as usize;
            let table_len = get_u32(data, record_off + 12)? as usize;
            if table_off + table_len > data.len() {
                return Err(Error::Invalid);
            }
            let table_data = &data[table_off..(table_off + table_len)];
            tables.insert(tag, table_data);
            record_off += 16;
        }
        // Get the tables we need
        let head = tables
            .get(&Tag::from_str("head"))
            .ok_or(Error::Invalid)
            .and_then(|data| head::Head::load(data))?;
        let hhea = tables
            .get(&Tag::from_str("hhea"))
            .ok_or(Error::Invalid)
            .and_then(|data| hhea::Hhea::load(data))?;
        let maxp = tables
            .get(&Tag::from_str("maxp"))
            .ok_or(Error::Invalid)
            .and_then(|data| maxp::Maxp::load(data))?;
        let hmtx = tables
            .get(&Tag::from_str("hmtx"))
            .ok_or(Error::Invalid)
            .and_then(|data| {
                hmtx::Hmtx::load(
                    data,
                    maxp.num_glyphs() as usize,
                    hhea.num_of_h_metrics() as usize,
                )
            })?;
        let cmap = tables
            .get(&Tag::from_str("cmap"))
            .ok_or(Error::Invalid)
            .and_then(|data| cmap::Cmap::load(data))?;
        Ok(Face {
            tables: tables,
            head: head,
            hhea: hhea,
            maxp: maxp,
            hmtx: hmtx,
            cmap: cmap,
        })
    }
}

/// An OpenType font file can either be a "font collection" (e.g. *.otc) file, or contain a
/// single font. To provide a uniform interface, rype opens a font file as a `FontCollection`.
/// The `FontCollection` can then be queried for individual `Face`s.
pub struct FontCollection {
    data: Box<[u8]>,
    face_offsets: Vec<usize>,
}

impl std::fmt::Debug for FontCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("FontCollection")
            .field(
                "faces",
                &self
                    .face_offsets
                    .iter()
                    .map(|off| Face::load(&self.data, *off))
                    .collect::<Vec<Result<Face>>>(),
            )
            .finish()
    }
}

impl FontCollection {
    /// Load font collection from file
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<FontCollection> {
        let data = std::fs::read(path)?.into_boxed_slice();
        FontCollection::from_data(data)
    }

    /// Load font collection from in-memory buffer. (Note: this creates a copy of the memory
    /// buffer)
    pub fn new_from(data: &[u8]) -> Result<FontCollection> {
        FontCollection::from_data(data.into())
    }

    /// Get face at given index
    pub fn get_face(&self, idx: usize) -> Result<Face> {
        self.face_offsets
            .get(idx)
            .ok_or(Error::FaceIndexOutOfBounds)
            .and_then(|&off| Face::load(&self.data, off))
    }

    /// Get number of faces in font collection
    pub fn num_faces(&self) -> usize {
        self.face_offsets.len()
    }

    /// Load font collection from data
    fn from_data(data: Box<[u8]>) -> Result<FontCollection> {
        // Is this a font collection, or a single face?
        let tag = get_tag(&data, 0)?;
        let face_offsets = if tag == Tag::from_str("ttcf") {
            let num_fonts = get_u32(&data, 8)? as usize;
            let mut face_offsets = Vec::with_capacity(num_fonts);
            let mut off = 12;
            for _ in 0..num_fonts {
                let face_off = get_u32(&data, off)? as usize;
                face_offsets.push(face_off);
                off += 12;
            }
            face_offsets
        } else {
            vec![0]
        };
        Ok(FontCollection {
            data: data,
            face_offsets: face_offsets,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn get_path(res: &str) -> PathBuf {
        let mut buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        buf.push("fonts");
        buf.push(res);
        buf
    }

    #[test]
    fn test_firacode() {
        let path = get_path("FiraCode-Regular.otf");
        let fc = FontCollection::new(&path).unwrap();
        assert_eq!(
            &format!("{:?}", fc),
            "FontCollection { faces: [Ok(Face { head: Head \
             { units_per_em: 1950, xmin: -3556, ymin: -1001, xmax: 2385, ymax: 2401, \
             lowest_rec_ppem: 3, index_to_loc_format: 0 }, hhea: Hhea { ascender: 1800, \
             descender: -600, num_of_h_metrics: 1746 }, maxp: Maxp { num_glyphs: 1746 }, \
             cmap: Cmap { subtables: [Subtable { platform_id: 0, encoding_id: 3, format: Ok(4) \
             }, Subtable { platform_id: 3, encoding_id: 1, format: Ok(4) }, Subtable { \
             platform_id: 0, encoding_id: 4, format: Ok(12) }, Subtable { platform_id: 3, \
             encoding_id: 10, format: Ok(12) }], active: Some(Subtable { platform_id: 3, \
             encoding_id: 10, format: Ok(12) }) } })] }"
        );
    }

    #[test]
    fn test_firacode_cmap() {
        let path = get_path("FiraCode-Regular.otf");
        let fc = FontCollection::new(&path).unwrap();
        let face = fc.get_face(0).unwrap();
        assert_eq!(face.cmap.get_glyph_id('A' as u32).unwrap(), GlyphID(1));
        assert_eq!(face.cmap.get_glyph_id('B' as u32).unwrap(), GlyphID(13));
        assert_eq!(face.cmap.get_glyph_id('C' as u32).unwrap(), GlyphID(14));
        assert_eq!(face.cmap.get_glyph_id('D' as u32).unwrap(), GlyphID(20));
        assert_eq!(face.cmap.get_glyph_id('E' as u32).unwrap(), GlyphID(24));
        assert_eq!(face.cmap.get_glyph_id('F' as u32).unwrap(), GlyphID(34));
        assert_eq!(face.cmap.get_glyph_id('a' as u32).unwrap(), GlyphID(134));
        assert_eq!(face.cmap.get_glyph_id('>' as u32).unwrap(), GlyphID(1171));
        assert_eq!(face.cmap.get_glyph_id('=' as u32).unwrap(), GlyphID(1169));
    }
}
