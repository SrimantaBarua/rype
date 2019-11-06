//! Errors within `rype`
// (C) 2019 Srimanta Barua <srimanta.barua1@gmail.com>

/// `rype`'s Result type
pub type Result<T> = std::result::Result<T, Error>;

/// Errors within `rype`
#[derive(Debug)]
pub enum Error {
    /// IO errors (reading files, etc.)
    Io(std::io::Error),
    /// Invalid font file
    Invalid,
    /// Face index out of bounds
    FaceIndexOutOfBounds,
    /// Glyph ID > number of available glyphs in the font
    GlyphIDOutOfBounds,
    /// No charmap loaded
    NoCharmap,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(ref e) => write!(f, "IO error: {}", e),
            Error::Invalid => write!(f, "invalid font file"),
            Error::FaceIndexOutOfBounds => write!(f, "face index out of bounds"),
            Error::GlyphIDOutOfBounds => write!(f, "glyph ID out of bounds"),
            Error::NoCharmap => write!(f, "no charmap loaded")
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::Io(ref e) => e.description(),
            Error::Invalid => "invalid font file",
            Error::FaceIndexOutOfBounds => "face index out of bounds",
            Error::GlyphIDOutOfBounds => "glyph ID out of bounds",
            Error::NoCharmap => "no charmap loaded"
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Error::Io(ref e) => Some(e),
            _ => None,
        }
    }
}
