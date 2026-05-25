use aligned_vec::{ABox, avec};
use alloc::sync::Arc;

use crate::color::{Color, ColorI16};
use crate::font::FontGlyph;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FontBufferGlyph {
    pub buf: ABox<[Color]>,
    pub width: u32,
    pub height: u32,
}

#[expect(unsafe_code)]
impl FontBufferGlyph {
    pub fn from(glyph: FontGlyph) -> Self {
        let (width, height) = (glyph.advance as u32, glyph.line_height as u32);

        debug_assert!(width > 0 && height > 0, "Glyph width and height must be greater than 0");

        let mut buf = avec![Color::default(); (width * height) as usize].into_boxed_slice();

        for y in 0..height {
            for x in 0..width {
                unsafe { *buf.get_unchecked_mut((y * width + x) as usize) = glyph.get(x, y) };
            }
        }

        Self { buf, width, height }
    }

    pub fn blank(width: u32, height: u32) -> Self {
        Self {
            buf: avec![Color::default(); (width * height) as usize].into_boxed_slice(),
            width,
            height,
        }
    }

    #[inline(always)]
    pub(crate) fn at(&self, x: u32, y: u32) -> Color {
        debug_assert!(x < self.width && y < self.height, "Coordinates out of bounds");
        unsafe { *self.buf.get_unchecked((y * self.width + x) as usize) }
    }

    #[inline(always)]
    pub(crate) fn at_index(&self, idx: usize) -> Color {
        debug_assert!(idx < self.buf.len(), "Index out of bounds");
        unsafe { *self.buf.get_unchecked(idx) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FontBufferColoredGlyph {
    pub buf: ABox<[Color]>,
    pub width: u32,
    pub height: u32,
}

impl FontBufferColoredGlyph {
    #[deprecated]
    pub fn from(glyph: &Arc<FontBufferGlyph>, fg: Color, bg: Color) -> Self {
        let fg = ColorI16::from(fg);
        let bg = ColorI16::from(bg);
        let a = fg - bg;
        let b = bg * 255 + 255;
        let size = (glyph.width * glyph.height) as usize;
        let mut buf = avec![Color::default();size].into_boxed_slice();
        for i in 0..size {
            let k = ColorI16::from(glyph.at_index(i));
            buf[i] = ((k * a + b) >> 8).into();
        }
        Self {
            buf,
            width: glyph.width,
            height: glyph.height,
        }
    }
}
