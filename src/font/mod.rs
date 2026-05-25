mod buffer_glyph;
pub use buffer_glyph::{FontBufferColoredGlyph, FontBufferGlyph};

mod buffer;
pub use buffer::{FontBuffer, RenderResult, RenderResultRef, RenderResultType, RenderResultUnion};

mod glyph;
pub use glyph::{FontGlyph, FontGlyphBuffer};

pub mod provider;

mod bytes;
pub use bytes::Bytes;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontChar {
    ch: char,
    width: u8,
    bold: bool,
    italic: bool,
}

impl FontChar {
    pub const fn new(ch: char, width: u8, bold: bool, italic: bool) -> Self {
        Self {
            ch,
            width,
            bold,
            italic,
        }
    }
}

/// A trait for rendering fonts. It provides methods to get the size of the font, the size of a character, and to get the glyph of a character.
pub trait FontRenderer: Send + Sync {
    fn size(&self) -> (usize, usize);
    fn sizeof(&self, ch: char, bold: bool, italic: bool) -> Option<(usize, usize)>;
    fn get(&mut self, ch: char, bold: bool, italic: bool) -> FontGlyph<'_>;
}
