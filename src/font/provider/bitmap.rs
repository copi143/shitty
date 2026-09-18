use alloc::boxed::Box;
use noto_sans_mono_bitmap::{FontWeight, RasterHeight};
use noto_sans_mono_bitmap::{get_raster, get_raster_width};

use super::super::{FontGlyph, FontGlyphBuffer, FontRenderer};

const FONT_WIDTH: usize = get_raster_width(FontWeight::Regular, FONT_HEIGHT);
const FONT_HEIGHT: RasterHeight = RasterHeight::Size20;

/// Noto Sans Mono 位图字体渲染器。使用内嵌的位图数据渲染字符。
///
/// Noto Sans Mono bitmap font renderer. Uses embedded bitmap data to render characters.
pub struct BitmapFont;

assert_send_sync!(BitmapFont);

impl BitmapFont {
    /// Create a new `BitmapFont` instance.
    /// - Returns a boxed `BitmapFont` that implements the `FontRenderer` trait.
    #[expect(clippy::new_ret_no_self)]
    pub fn new() -> Box<dyn FontRenderer> {
        Box::new(Self)
    }
}

impl FontRenderer for BitmapFont {
    fn size(&self) -> (usize, usize) {
        (FONT_WIDTH, FONT_HEIGHT as usize)
    }

    fn sizeof(&self, _ch: char, _bold: bool, _italic: bool) -> Option<(usize, usize)> {
        Some((FONT_WIDTH, FONT_HEIGHT as usize))
    }

    fn get(&mut self, ch: char, bold: bool, _italic: bool) -> FontGlyph<'_> {
        let font_weight = if bold { FontWeight::Bold } else { FontWeight::Regular };

        let buf = match get_raster(ch, font_weight, FONT_HEIGHT) {
            Some(char_raster) => FontGlyphBuffer::Slice2DGray(char_raster.raster()),
            None => FontGlyphBuffer::Blank,
        };

        FontGlyph {
            buf,
            code: ch,
            top: 0,
            left: 0,
            width: FONT_WIDTH as i32,
            height: FONT_HEIGHT as i32,
            advance: FONT_WIDTH as i32,
            line_height: FONT_HEIGHT as i32,
        }
    }
}
