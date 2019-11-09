//! Rust OpenType font rasterization, shaping, and layout library
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use std::collections::HashMap;

pub mod error;
use error::*;

mod types;
pub use types::GlyphID;
use types::*;

mod cmap;
mod glyf;
mod head;
mod hhea;
mod hmtx;
mod loca;
mod maxp;

/// Glyph outline information
#[derive(Debug)]
enum GlyphOutline<'a> {
    TrueType(glyf::TTGlyph<'a>),
}

/// Data for a glyph
#[derive(Debug)]
struct Glyph<'a> {
    outline: GlyphOutline<'a>
}

/// A `Face` could either contain TrueType outlines, or CFF data
#[derive(Debug)]
enum FaceTyp<'a> {
    TrueType(loca::Loca<'a>, glyf::Glyf<'a>),
    CFF,
}

/// A face within the OpenType font file. This face alone cannot be used to render glyphs -
/// it must be scaled first
pub struct Face<'a> {
    tables: HashMap<Tag, &'a [u8]>,
    head: head::Head<'a>,
    hhea: hhea::Hhea<'a>,
    maxp: maxp::Maxp<'a>,
    hmtx: hmtx::Hmtx<'a>,
    cmap: cmap::Cmap<'a>,
    typ: FaceTyp<'a>,
}

impl<'a> std::fmt::Debug for Face<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Face")
            .field("head", &self.head)
            .field("hhea", &self.hhea)
            .field("maxp", &self.maxp)
            .field("cmap", &self.cmap)
            .field("typ", &self.typ)
            .finish()
    }
}

impl<'a> Face<'a> {
    /// Scale face to get a `ScaledFace`
    pub fn scale(&self, point_width: f32, point_height: f32, dpi_x: u16, dpi_y: u16) -> ScaledFace {
        let pix_width = (point_width * dpi_x as f32) / 72.0;
        let pix_height = (point_height * dpi_y as f32) / 72.0;
        ScaledFace {
            pix_width: pix_width,
            pix_height: pix_height,
            face: self,
        }
    }

    /// Get glyph ID for codepoint
    pub fn get_glyph_id(&self, codepoint: u32) -> Result<GlyphID> {
        self.cmap.get_glyph_id(codepoint)
    }

    /// Get glyph information
    fn get_glyph(&self, id: GlyphID) -> Result<Glyph> {
        match self.typ {
            FaceTyp::TrueType(ref loca, ref glyf) => loca
                .get_offset(id)
                .and_then(|off| glyf.glyph(off))
                .map(|ttglyph| Glyph {
                    outline: GlyphOutline::TrueType(ttglyph)
                }),
            FaceTyp::CFF => Err(Error::Unimplemented("CFF support".to_owned())),
        }
    }

    /// Load face information from data. The `offset` provided is the offset from the beginning
    /// of the file to the Offset Table for the face
    fn load(data: &[u8], offset: usize) -> Result<Face> {
        let sfnt_version = get_tag(data, offset)?;
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
        let num_glyphs = maxp.num_glyphs() as usize;
        let idx_to_loc_fmt = head.idx_to_loc_fmt()?;
        let typ = match sfnt_version {
            Tag(0x00010000) => {
                let loca = tables
                    .get(&Tag::from_str("loca"))
                    .ok_or(Error::Invalid)
                    .and_then(|data| loca::Loca::load(data, num_glyphs, idx_to_loc_fmt))?;
                let glyf = tables
                    .get(&Tag::from_str("glyf"))
                    .ok_or(Error::Invalid)
                    .map(|data| glyf::Glyf(data))?;
                FaceTyp::TrueType(loca, glyf)
            }
            Tag(0x4F54544F) => FaceTyp::CFF,
            _ => return Err(Error::Invalid),
        };
        Ok(Face {
            tables: tables,
            head: head,
            hhea: hhea,
            maxp: maxp,
            hmtx: hmtx,
            cmap: cmap,
            typ: typ,
        })
    }
}

/// Glyph data with scaling
#[derive(Debug)]
pub struct ScaledGlyph<'a> {
    pix_width: f32,
    pix_height: f32,
    glyph: Glyph<'a>,
}

/// We can't render glyphs for a face without appropriate scaling. So, only a `ScaledFace`
/// allows rendering of glyphs. Multiple `ScaledFace` instances can be created for the same
/// `Face`, at negligible extra cost
#[derive(Debug)]
pub struct ScaledFace<'a> {
    pix_width: f32,
    pix_height: f32,
    face: &'a Face<'a>,
}

impl<'a> ScaledFace<'a> {
    /// Get glyph ID for codepoint
    pub fn get_glyph_id(&self, codepoint: u32) -> Result<GlyphID> {
        self.face.get_glyph_id(codepoint)
    }

    /// Get glyph information for glyph_id
    pub fn get_glyph(&self, glyph_id: GlyphID) -> Result<ScaledGlyph> {
        self.face.get_glyph(glyph_id).map(|glyph| ScaledGlyph {
            pix_height: self.pix_height,
            pix_width: self.pix_width,
            glyph: glyph,
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
             lowest_rec_ppem: 3, index_to_loc_format: Off16 }, hhea: Hhea { ascender: 1800, \
             descender: -600, num_of_h_metrics: 1746 }, maxp: Maxp { num_glyphs: 1746 }, \
             cmap: Cmap { subtables: [Subtable { platform_id: 0, encoding_id: 3, format: Ok(4) \
             }, Subtable { platform_id: 3, encoding_id: 1, format: Ok(4) }, Subtable { \
             platform_id: 0, encoding_id: 4, format: Ok(12) }, Subtable { platform_id: 3, \
             encoding_id: 10, format: Ok(12) }], active: Some(Subtable { platform_id: 3, \
             encoding_id: 10, format: Ok(12) }) }, typ: CFF })] }"
        );
    }

    #[test]
    fn test_firacode_tables() {
        let path = get_path("FiraCode-Regular.otf");
        let fc = FontCollection::new(&path).unwrap();
        let face = fc.get_face(0).unwrap();
        let tables = face.tables;
        let mut names = tables
            .keys()
            .map(|k| format!("{}", k))
            .collect::<Vec<String>>();
        names.sort();
        assert_eq!(
            &names.join(", "),
            "CFF , GDEF, GPOS, GSUB, OS/2, cmap, head, hhea, \
             hmtx, maxp, name, post"
        );
    }

    #[test]
    fn test_firacode_cmap() {
        let path = get_path("FiraCode-Regular.otf");
        let fc = FontCollection::new(&path).unwrap();
        let face = fc.get_face(0).unwrap();
        assert_eq!(face.get_glyph_id('A' as u32).unwrap(), GlyphID(1));
        assert_eq!(face.get_glyph_id('B' as u32).unwrap(), GlyphID(13));
        assert_eq!(face.get_glyph_id('C' as u32).unwrap(), GlyphID(14));
        assert_eq!(face.get_glyph_id('D' as u32).unwrap(), GlyphID(20));
        assert_eq!(face.get_glyph_id('E' as u32).unwrap(), GlyphID(24));
        assert_eq!(face.get_glyph_id('F' as u32).unwrap(), GlyphID(34));
        assert_eq!(face.get_glyph_id('a' as u32).unwrap(), GlyphID(134));
        assert_eq!(face.get_glyph_id('>' as u32).unwrap(), GlyphID(1171));
        assert_eq!(face.get_glyph_id('=' as u32).unwrap(), GlyphID(1169));
    }

    #[test]
    fn test_hack() {
        let path = get_path("Hack-Regular.ttf");
        let fc = FontCollection::new(&path).unwrap();
        assert_eq!(
            &format!("{:?}", fc),
            "FontCollection { faces: [Ok(Face { head: Head { units_per_em: 2048, xmin: -954, \
             ymin: -605, xmax: 1355, ymax: 2027, lowest_rec_ppem: 6, index_to_loc_format: \
             Off32 }, hhea: Hhea { ascender: 1901, descender: -483, num_of_h_metrics: 1543 }, \
             maxp: Maxp { num_glyphs: 1573 }, cmap: Cmap { subtables: [Subtable { platform_id: 0, \
             encoding_id: 3, format: Ok(4) }, Subtable { platform_id: 3, encoding_id: 1, \
             format: Ok(4) }], active: Some(Subtable { platform_id: 3, encoding_id: 1, \
             format: Ok(4) }) }, typ: TrueType(Loca { num_glyphs: 1573, index_to_loc_fmt: \
             Off32 }, Glyf) })] }"
        );
    }

    #[test]
    fn test_hack_tables() {
        let path = get_path("Hack-Regular.ttf");
        let fc = FontCollection::new(&path).unwrap();
        let face = fc.get_face(0).unwrap();
        let tables = face.tables;
        let mut names = tables
            .keys()
            .map(|k| format!("{}", k))
            .collect::<Vec<String>>();
        names.sort();
        assert_eq!(
            &names.join(", "),
            "DSIG, GSUB, OS/2, TTFA, cmap, cvt , fpgm, gasp, glyf, \
             head, hhea, hmtx, loca, maxp, name, post, prep"
        );
    }

    #[test]
    fn test_hack_cmap() {
        let path = get_path("Hack-Regular.ttf");
        let fc = FontCollection::new(&path).unwrap();
        let face = fc.get_face(0).unwrap();
        assert_eq!(face.get_glyph_id('A' as u32).unwrap(), GlyphID(1425));
        assert_eq!(face.get_glyph_id('B' as u32).unwrap(), GlyphID(12));
        assert_eq!(face.get_glyph_id('C' as u32).unwrap(), GlyphID(13));
        assert_eq!(face.get_glyph_id('D' as u32).unwrap(), GlyphID(18));
        assert_eq!(face.get_glyph_id('E' as u32).unwrap(), GlyphID(22));
        assert_eq!(face.get_glyph_id('F' as u32).unwrap(), GlyphID(31));
        assert_eq!(face.get_glyph_id('a' as u32).unwrap(), GlyphID(118));
        assert_eq!(face.get_glyph_id('>' as u32).unwrap(), GlyphID(754));
        assert_eq!(face.get_glyph_id('=' as u32).unwrap(), GlyphID(750));
    }
}
