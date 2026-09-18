use alloc::{boxed::Box, sync::Arc};
use core::fmt::Debug;
use woff2::{convert_woff2_to_ttf, decode::is_woff2};

use super::super::{Bytes, FontGlyph, FontGlyphBuffer, FontRenderer};
#[cfg(feature = "font-abglyph")]
use super::abglyph::AbGlyphFont;
#[cfg(feature = "font-swash")]
use super::swash::SwashFont;

/// WOFF2 字体渲染器。自动检测并解压 WOFF2 格式字体，然后委托给底层 TrueType 后端。
///
/// WOFF2 font renderer. Automatically detects and decompresses WOFF2 format fonts,
/// then delegates to the underlying TrueType backend.
pub struct Woff2Font {
    backend: Box<dyn FontRenderer>,
}

assert_send_sync!(Woff2Font);

impl Debug for Woff2Font {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Woff2Font").finish()
    }
}

impl Woff2Font {
    #[expect(clippy::new_ret_no_self)]
    /// 创建一个新的 WOFF2 字体渲染器。如果输入是 WOFF2 格式会自动解压。
    ///
    /// Create a new WOFF2 font renderer. Automatically decompresses if the input is WOFF2 format.
    pub fn new<T: Into<Bytes>>(font_size: i32, font_bytes: T) -> Box<dyn FontRenderer> {
        let backend = match font_bytes.into() {
            Bytes::Static(bytes) => {
                if is_woff2(bytes) {
                    let decoded = Self::decode(bytes);
                    Self::new_backend(font_size, decoded)
                } else {
                    Self::new_backend(font_size, bytes)
                }
            }
            Bytes::Arc(bytes) => {
                if is_woff2(&bytes) {
                    let decoded = Self::decode(&bytes);
                    Self::new_backend(font_size, decoded)
                } else {
                    Self::new_backend(font_size, bytes)
                }
            }
        };
        Box::new(Self { backend })
    }

    fn decode(bytes: &[u8]) -> Arc<[u8]> {
        let mut input = bytes;
        Arc::from(convert_woff2_to_ttf(&mut input).expect("Failed to decompress WOFF2 font data"))
    }

    #[allow(unreachable_code)]
    fn new_backend<T: Into<Bytes>>(font_size: i32, font_bytes: T) -> Box<dyn FontRenderer> {
        #[cfg(feature = "font-abglyph")]
        return AbGlyphFont::new(font_size, font_bytes);
        #[cfg(feature = "font-swash")]
        return SwashFont::new(font_size, font_bytes);
    }
}

impl FontRenderer for Woff2Font {
    fn size(&self) -> (usize, usize) {
        self.backend.size()
    }

    fn sizeof(&self, ch: char, bold: bool, italic: bool) -> Option<(usize, usize)> {
        self.backend.sizeof(ch, bold, italic)
    }

    fn get(&mut self, ch: char, bold: bool, italic: bool) -> FontGlyph<'_> {
        self.backend.get(ch, bold, italic)
    }
}
