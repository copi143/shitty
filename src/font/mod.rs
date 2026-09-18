//! 字体渲染。通过 [`FontRenderer`] trait 支持可插拔的字体后端。
//!
//! 内置字体提供者（通过 feature 启用）：
//! - `font-unifont`：Unicode 位图字体，no_std 兼容
//! - `font-abglyph` / `font-swash`：TrueType/OpenType 字体
//! - `font-bitmap`：内嵌 Noto Sans Mono 位图
//! - `IBM_VGA_8x16`：经典 VGA 硬件字体
//!
//! 字形渲染结果通过 [`FontGlyph`] / [`FontBufferGlyph`] 传递到渲染管线。
//!
//! ---
//!
//! Font rendering. Supports pluggable font backends via the [`FontRenderer`] trait.
//!
//! Built-in font providers (enabled via features):
//! - `font-unifont`: Unicode bitmap font, no_std compatible
//! - `font-abglyph` / `font-swash`: TrueType/OpenType fonts
//! - `font-bitmap`: Embedded Noto Sans Mono bitmap
//! - `IBM_VGA_8x16`: Classic VGA hardware font
//!
//! Glyph rendering results are passed to the rendering pipeline via [`FontGlyph`] / [`FontBufferGlyph`].

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
