use alloc::boxed::Box;

use super::super::{FontGlyph, FontGlyphBuffer, FontRenderer};

pub struct IBM_VGA_8x16;

assert_send_sync!(IBM_VGA_8x16);

impl IBM_VGA_8x16 {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Box<dyn FontRenderer> {
        Box::new(Self)
    }
}

impl FontRenderer for IBM_VGA_8x16 {
    fn size(&self) -> (usize, usize) {
        (8, 16)
    }

    fn sizeof(&self, _ch: char, _bold: bool, _italic: bool) -> Option<(usize, usize)> {
        Some((8, 16))
    }

    fn get(&mut self, ch: char, _bold: bool, _italic: bool) -> FontGlyph<'_> {
        // TODO
        FontGlyph {
            buf: FontGlyphBuffer::Blank,
            code: ch,
            top: 0,
            left: 0,
            width: 8,
            height: 16,
            advance: 8,
            line_height: 16,
        }
    }
}
