use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::fmt::Debug;
use core_maths::CoreFloat as _;
use swash::FontRef;
use swash::scale::image::Content;
use swash::scale::{Render, ScaleContext, Source};
use swash::zeno::Format;

use super::super::{Bytes, FontGlyph, FontGlyphBuffer, FontRenderer};

/// 基于 swash 的 TrueType/OpenType 字体渲染器。支持可变字体和高级排版特性。
///
/// Swash-based TrueType/OpenType font renderer. Supports variable fonts and advanced typography features.
pub struct SwashFont {
    height: i32,
    width: i32,
    base_line: f32,
    font_size: f32,
    font_bytes: Arc<[u8]>,
    italic_font_bytes: Option<Arc<[u8]>>,
}

assert_send_sync!(SwashFont);

impl Debug for SwashFont {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SwashFont")
            .field("height", &self.height)
            .field("width", &self.width)
            .field("font_size", &self.font_size)
            .finish()
    }
}

impl SwashFont {
    fn into_arc(font_bytes: Bytes) -> Arc<[u8]> {
        match font_bytes {
            Bytes::Static(bytes) => Arc::from(bytes),
            Bytes::Arc(bytes) => bytes,
        }
    }

    fn font_ref<'a>(bytes: &'a [u8], error: &'static str) -> FontRef<'a> {
        FontRef::from_index(bytes, 0).expect(error)
    }

    fn select_font(&self, italic: bool) -> FontRef<'_> {
        match self.italic_font_bytes.as_ref().filter(|_| italic) {
            Some(italic_font) => Self::font_ref(italic_font, "Failed to parse italic font data"),
            None => Self::font_ref(&self.font_bytes, "Failed to parse font data"),
        }
    }

    fn measure(font: FontRef<'_>, font_size: f32) -> (i32, i32, f32) {
        let metrics = font.metrics(&[]).scale(font_size);
        let glyph_metrics = font.glyph_metrics(&[]).scale(font_size);
        let width = glyph_metrics.advance_width(font.charmap().map('M')).ceil() as i32;
        let height = (metrics.ascent + metrics.descent + metrics.leading).ceil() as i32;
        (width.max(1), height.max(1), metrics.ascent)
    }

    fn render_to_gray(image: swash::scale::image::Image) -> FontGlyphBuffer<'static> {
        match image.content {
            Content::Mask => FontGlyphBuffer::Owned1DGray(image.data),
            Content::SubpixelMask | Content::Color => {
                let mut gray = Vec::with_capacity((image.placement.width * image.placement.height) as usize);
                for px in image.data.chunks_exact(4) {
                    gray.push(px[3]);
                }
                FontGlyphBuffer::Owned1DGray(gray)
            }
        }
    }

    /// Create a new `SwashFont` from the given font size and font bytes.
    /// - Returns a boxed `SwashFont` that implements the `FontRenderer` trait.
    #[expect(clippy::new_ret_no_self)]
    pub fn new<T: Into<Bytes>>(font_size: i32, font_bytes: T) -> Box<dyn FontRenderer> {
        assert!(font_size > 0, "Font size must be positive");
        assert!(font_size <= u16::MAX as i32, "Font size too large");

        let font_bytes = Self::into_arc(font_bytes.into());
        assert!(!font_bytes.is_empty(), "Font bytes cannot be empty");

        let font = Self::font_ref(&font_bytes, "Failed to parse font data");
        let font_size = font_size as f32;
        let (width, height, base_line) = Self::measure(font, font_size);

        Box::new(Self {
            height,
            width,
            base_line,
            font_size,
            font_bytes,
            italic_font_bytes: None,
        })
    }

    pub fn with_italic<T: Into<Bytes>>(mut self, font_bytes: T) -> Self {
        let font_bytes = Self::into_arc(font_bytes.into());
        assert!(!font_bytes.is_empty(), "Font bytes cannot be empty");
        let _ = Self::font_ref(&font_bytes, "Failed to parse italic font data");
        self.italic_font_bytes = Some(font_bytes);
        self
    }
}

impl FontRenderer for SwashFont {
    fn size(&self) -> (usize, usize) {
        (self.width as usize, self.height as usize)
    }

    fn sizeof(&self, ch: char, _bold: bool, italic: bool) -> Option<(usize, usize)> {
        let select_font = self.select_font(italic);
        let glyph_id = select_font.charmap().map(ch);
        if glyph_id == 0 {
            return None;
        }

        let glyph_metrics = select_font.glyph_metrics(&[]).scale(self.font_size);
        let advance = glyph_metrics.advance_width(glyph_id).ceil() as i32;
        let advance = (advance + self.width - 1) / self.width * self.width;
        Some((advance.max(self.width) as usize, self.height as usize))
    }

    fn get(&mut self, ch: char, bold: bool, italic: bool) -> FontGlyph<'_> {
        let select_font = self.select_font(italic);
        let glyph_id = select_font.charmap().map(ch);

        if glyph_id == 0 {
            return FontGlyph {
                buf: FontGlyphBuffer::Blank,
                code: ch,
                top: 0,
                left: 0,
                width: 0,
                height: 0,
                advance: self.width,
                line_height: self.height,
            };
        }

        let mut scale_context = ScaleContext::new();
        let weight = if bold { 700.0 } else { 400.0 };
        let mut scaler = scale_context
            .builder(select_font)
            .size(self.font_size)
            .hint(true)
            .variations(&[("wght", weight)])
            .build();
        let image = Render::new(&[Source::Outline]).format(Format::Alpha).render(&mut scaler, glyph_id);

        let glyph_metrics = select_font.glyph_metrics(&[]).scale(self.font_size);
        let advance = glyph_metrics.advance_width(glyph_id).ceil() as i32;

        if let Some(image) = image {
            let top = image.placement.top;
            let left = image.placement.left;
            let width = image.placement.width as i32;
            let height = image.placement.height as i32;
            FontGlyph {
                buf: Self::render_to_gray(image),
                code: ch,
                top: top + self.base_line as i32,
                left,
                width,
                height,
                advance: advance.max(self.width),
                line_height: self.height,
            }
        } else {
            FontGlyph {
                buf: FontGlyphBuffer::Blank,
                code: ch,
                top: 0,
                left: 0,
                width: 0,
                height: 0,
                advance: advance.max(self.width),
                line_height: self.height,
            }
        }
    }
}
