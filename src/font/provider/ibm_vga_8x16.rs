use alloc::boxed::Box;

use super::super::{FontGlyph, FontGlyphBuffer, FontRenderer};

/// IBM VGA 8x16 硬件字体渲染器。
/// - 固定尺寸 8x16 像素
/// - **注意：当前实现只返回空白字形，TODO 需要填充实际字模数据**
///
/// IBM VGA 8x16 hardware font renderer.
/// - Fixed size 8x16 pixels
/// - **Note: The current implementation only returns blank glyphs; TODO: needs actual glyph data**
pub struct IBM_VGA_8x16;

assert_send_sync!(IBM_VGA_8x16);

impl IBM_VGA_8x16 {
    /// 创建一个 IBM VGA 8x16 字体渲染器实例。
    ///
    /// Create an IBM VGA 8x16 font renderer instance.
    #[expect(clippy::new_ret_no_self)]
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
