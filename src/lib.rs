//! Rust OpenType font rasterization, shaping, and layout library
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use std::collections::HashMap;

pub mod error;
use error::*;

mod types;
use types::*;

/// A face within the OpenType font file. This face alone cannot be used to render glyphs -
/// it must be scaled first
#[derive(Debug)]
pub struct Face<'a> {
    tables: HashMap<Tag, &'a [u8]>,
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
        Ok(Face { tables: tables })
    }
}

/// An OpenType font file can either be a "font collection" (e.g. *.otc) file, or contain a
/// single font. To provide a uniform interface, rype opens a font file as a `FontCollection`.
/// The `FontCollection` can then be queried for individual `Face`s.
#[derive(Debug)]
pub struct FontCollection {
    data: Box<[u8]>,
    face_offsets: Vec<usize>,
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
    use std::path::PathBuf;
    use super::*;

    fn get_path(res: &str) -> PathBuf {
        let mut buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        buf.push("fonts");
        buf.push(res);
        buf
    }

    #[test]
    fn load_firacode() {
        let path = get_path("FiraCode-Regular.otf");
        let font_collection = FontCollection::new(&path).unwrap();
        let _face = font_collection.get_face(0).unwrap();
    }
}
