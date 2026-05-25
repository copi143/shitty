use alloc::boxed::Box;

use super::super::{FontGlyph, FontGlyphBuffer, FontRenderer};

pub struct EmptyFontRenderer {
    width: usize,
    height: usize,
}

assert_send_sync!(EmptyFontRenderer);

impl EmptyFontRenderer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(width: usize, height: usize) -> Box<dyn FontRenderer> {
        Box::new(Self { width, height })
    }
}

impl FontRenderer for EmptyFontRenderer {
    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn sizeof(&self, _ch: char, _bold: bool, _italic: bool) -> Option<(usize, usize)> {
        Some((self.width, self.height))
    }

    fn get(&mut self, ch: char, _bold: bool, _italic: bool) -> FontGlyph<'_> {
        FontGlyph {
            buf: FontGlyphBuffer::Blank,
            code: ch,
            top: 0,
            left: 0,
            width: self.width as i32,
            height: self.height as i32,
            advance: self.width as i32,
            line_height: self.height as i32,
        }
    }
}
