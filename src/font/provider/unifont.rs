use alloc::boxed::Box;
use baremetal_unifont::{char_width, get_glyph};

use super::super::{FontGlyph, FontGlyphBuffer, FontRenderer};

pub struct Unifont {
    buf: [u8; 32],
}

assert_send_sync!(Unifont);

impl Unifont {
    /// Create a new `Unifont` instance.
    /// - Returns a boxed `Unifont` that implements the `FontRenderer` trait.
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Box<dyn FontRenderer> {
        Box::new(Self { buf: [0; 32] })
    }
}

impl FontRenderer for Unifont {
    fn size(&self) -> (usize, usize) {
        (8, 16)
    }

    fn sizeof(&self, ch: char, _bold: bool, _italic: bool) -> Option<(usize, usize)> {
        Some((char_width(ch).unwrap_or(1) * 8, 16))
    }

    fn get(&mut self, ch: char, _bold: bool, _italic: bool) -> FontGlyph<'_> {
        let glyph = get_glyph(ch).unwrap_or_default();
        match glyph.width() {
            8 => {
                for y in 0..16 {
                    self.buf[y] = 0;
                    for x in 0..8 {
                        self.buf[y] |= (if glyph.get(x, y) { 1 } else { 0 }) << (7 - x);
                    }
                }
            }
            16 => {
                for y in 0..16 {
                    self.buf[y * 2] = 0;
                    self.buf[y * 2 + 1] = 0;
                    for x in 0..8 {
                        self.buf[y * 2] |= (if glyph.get(x, y) { 1 } else { 0 }) << (7 - x);
                    }
                    for x in 8..16 {
                        self.buf[y * 2 + 1] |= (if glyph.get(x, y) { 1 } else { 0 }) << (15 - x);
                    }
                }
            }
            _ => unreachable!(),
        }

        FontGlyph {
            buf: FontGlyphBuffer::Slice1DBitmap(&self.buf),
            code: ch,
            top: 0,
            left: 0,
            width: glyph.width() as i32,
            height: 16,
            advance: glyph.width() as i32,
            line_height: 16,
        }
    }
}
