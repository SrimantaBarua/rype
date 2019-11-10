// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

use rype::{FontCollection, GlyphBitmap};
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

fn render_pgm(path: &str, bitmap: GlyphBitmap) {
    let mut f = File::create(path).unwrap();
    write!(f, "P5\n{} {}\n255\n", bitmap.width, bitmap.height).unwrap();
    f.write(&bitmap.data).unwrap();
}

fn main() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fonts/Hack-Regular.ttf");
    let fc = FontCollection::new(&path).unwrap();
    let face = fc.get_face(0).unwrap();
    let scaled_face = face.scale(128.0, 128.0, 139, 144);

    let chars = String::from_utf8((b'a'..=b'z').chain(b'A'..=b'Z').chain(b'0'..=b'9').collect()).unwrap();
    for c in chars.chars() {
        let gid = scaled_face.get_glyph_id(c as u32).unwrap();
        let glyph = scaled_face.get_glyph(gid).unwrap();
        let bitmap = glyph.render().unwrap();
        render_pgm(&format!("{}.pgm", c), bitmap);
    }
}
