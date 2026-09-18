use ab_glyph::{Font, FontRef, Glyph, PxScale, ScaleFont, VariableFont};
use alloc::{boxed::Box, sync::Arc};
use core::fmt::Debug;
use core_maths::CoreFloat as _;

use super::super::{Bytes, FontGlyph, FontGlyphBuffer, FontRenderer};

/// 基于 ab_glyph 的 TrueType/OpenType 字体渲染器。支持可变字体。
///
/// ab_glyph-based TrueType/OpenType font renderer. Supports variable fonts.
pub struct AbGlyphFont {
    font: FontRef<'static>,
    italic_font: Option<FontRef<'static>>,
    height: i32,
    width: i32,
    base_line: f32,
    font_size: PxScale,
    /// We need to keep the font bytes alive as long as the font is alive, because `FontRef` contains references to it.
    #[allow(dead_code)]
    font_bytes: Option<Arc<[u8]>>,
    italic_font_bytes: Option<Arc<[u8]>>,
}

assert_send_sync!(AbGlyphFont);

impl Debug for AbGlyphFont {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AbGlyphFont")
            .field("height", &self.height)
            .field("width", &self.width)
            .field("font_size", &self.font_size)
            .finish()
    }
}

impl AbGlyphFont {
    /// Create a new `AbGlyphFont` from the given font size and font bytes.
    /// - Returns a boxed `AbGlyphFont` that implements the `FontRenderer` trait.
    #[expect(clippy::new_ret_no_self)]
    pub fn new<T: Into<Bytes>>(font_size: i32, font_bytes: T) -> Box<dyn FontRenderer> {
        assert!(font_size > 0, "Font size must be positive");
        assert!(font_size <= u16::MAX as i32, "Font size too large");

        #[expect(unsafe_code)]
        let (font_bytes_ref, font_bytes) = match font_bytes.into() {
            Bytes::Static(bytes) => (bytes, None),
            Bytes::Arc(bytes) => (unsafe { core::mem::transmute(&bytes as &[u8]) }, Some(bytes)),
        };

        assert!(!font_bytes_ref.is_empty(), "Font bytes cannot be empty");

        let font_size = PxScale::from(font_size as f32);
        let font = FontRef::try_from_slice(font_bytes_ref).unwrap();
        let scaled_font = font.as_scaled(font_size);

        let height = scaled_font.height().ceil() as i32;
        let width = scaled_font.h_advance(scaled_font.glyph_id('M')).ceil() as i32;
        let base_line = scaled_font.ascent();

        Box::new(Self {
            font,
            italic_font: None,
            height,
            width,
            font_size,
            base_line,
            font_bytes,
            italic_font_bytes: None,
        })
    }

    pub fn with_italic<T: Into<Bytes>>(mut self, font_bytes: T) -> Self {
        #[expect(unsafe_code)]
        let (font_bytes_ref, font_bytes) = match font_bytes.into() {
            Bytes::Static(bytes) => (bytes, None),
            Bytes::Arc(bytes) => (unsafe { core::mem::transmute(&bytes as &[u8]) }, Some(bytes)),
        };

        assert!(!font_bytes_ref.is_empty(), "Font bytes cannot be empty");

        self.italic_font = Some(FontRef::try_from_slice(font_bytes_ref).unwrap());
        self.italic_font_bytes = font_bytes;
        self
    }
}

impl FontRenderer for AbGlyphFont {
    fn size(&self) -> (usize, usize) {
        (self.width as usize, self.height as usize)
    }

    fn sizeof(&self, ch: char, _bold: bool, _italic: bool) -> Option<(usize, usize)> {
        let scaled_font = self.font.as_scaled(self.font_size);
        let id = scaled_font.glyph_id(ch);
        if id.0 == 0 {
            return None;
        }
        let advance = scaled_font.h_advance(id).ceil() as i32;
        let advance = (advance + self.width - 1) / self.width * self.width;
        Some((advance as usize, self.height as usize))
    }

    fn get(&mut self, ch: char, bold: bool, italic: bool) -> FontGlyph<'_> {
        let select_font = self.italic_font.as_mut().filter(|_| italic).unwrap_or(&mut self.font);

        select_font.set_variation(b"wght", if bold { 700.0 } else { 400.0 });

        let mut f_left = 0i32;
        let mut f_top = 0i32;
        let mut f_width = 0u32;
        let mut f_height = 0u32;

        let glyph = Glyph {
            id: select_font.glyph_id(ch),
            scale: self.font_size,
            position: ab_glyph::point(0.0, self.base_line),
        };
        let buf = if let Some(bitmap) = select_font.outline_glyph(glyph) {
            let px_bounds = bitmap.px_bounds();
            f_top = px_bounds.min.y as i32;
            f_left = px_bounds.min.x as i32;
            f_width = px_bounds.width() as u32;
            f_height = px_bounds.height() as u32;
            let mut img = vec![0u8; (f_width * f_height) as usize];
            bitmap.draw(|x, y, c| {
                img[(y * f_width + x) as usize] = (c * 255.0) as u8;
            });
            FontGlyphBuffer::Owned1DGray(img)
        } else {
            FontGlyphBuffer::Blank
        };

        let scaled_font = select_font.as_scaled(self.font_size);
        let advance = scaled_font.h_advance(select_font.glyph_id(ch)).ceil() as i32;

        FontGlyph {
            buf,
            code: ch,
            top: f_top,
            left: f_left,
            width: f_width as i32,
            height: f_height as i32,
            advance,
            line_height: self.height,
        }
    }
}
