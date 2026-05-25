use alloc::vec::Vec;
use core::fmt::Debug;

use crate::color::Color;

macro_rules! FontGlyphBuffer {
    ($(
        $(#[$type_doc:meta])* $type:ident : { $(
            $(#[$doc:meta])* $variant:ident($field:ty) => $expr:expr
        ),* $(,)? }
    ),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum FontGlyphBufferType {
            None,
            $(
                $(#[$type_doc])*
                $type
            ),*
        }
        /// 字形缓冲区
        ///
        /// Glyph buffer
        #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum FontGlyphBuffer<'buf> {
            Blank,
            Filled,
            $($(
                $(#[$doc])*
                $variant($field),
            )*)*
        }
        impl Debug for FontGlyphBuffer<'_> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    FontGlyphBuffer::Blank => write!(f, "FontGlyphBuffer::Blank"),
                    FontGlyphBuffer::Filled => write!(f, "FontGlyphBuffer::Filled"),
                    $($(
                        FontGlyphBuffer::$variant(buf) => {
                            write!(f, "FontGlyphBuffer::{}(len={})", stringify!($variant), buf.len())
                        }
                    )*)*
                }
            }
        }
        impl FontGlyphBuffer<'_> {
            pub const fn ty(&self) -> FontGlyphBufferType {
                match self {
                    FontGlyphBuffer::Blank => FontGlyphBufferType::None,
                    FontGlyphBuffer::Filled => FontGlyphBufferType::None,
                    $($(
                        FontGlyphBuffer::$variant(_) => FontGlyphBufferType::$type,
                    )*)*
                }
            }
        }
        impl FontGlyph<'_> {
            #[inline(always)]
            pub fn get(&self, x: u32, y: u32) -> Color {
                let (left, top, width, height) = (self.left, self.top, self.width, self.height);
                let (dx, dy) = (x as i32 - left, y as i32 - top);
                if dx < 0 || dy < 0 || dx >= width || dy >= height {
                    return Color::BLACK;
                }
                match &self.buf {
                    FontGlyphBuffer::Blank => Color::BLACK,
                    FontGlyphBuffer::Filled => Color::WHITE,
                    $($(
                        FontGlyphBuffer::$variant(buf) => $expr(buf, dx as u32, dy as u32, width as u32, height as u32),
                    )*)*
                }
            }
        }
    };
}

FontGlyphBuffer!(
    /// 位图 (每个像素存储为一个位，1为白色，0为黑色)
    Bitmap: {
        /// 静态 1D 位图 (每位表示一个像素，1为白色，0为黑色)
        Static1DBitmap(&'static [u8]) => |buf: &[u8], dx, dy, width, _| {
            let idx = (dy * width + dx) as usize;
            if buf[idx / 8] & (1 << (7 - idx % 8)) != 0 {
                Color::WHITE
            } else {
                Color::BLACK
            }
        },
        /// 切片 1D 位图 (每位表示一个像素，1为白色，0为黑色)
        Slice1DBitmap(&'buf [u8]) => |buf: &[u8], dx, dy, width, _| {
            let idx = (dy * width + dx) as usize;
            if buf[idx / 8] & (1 << (7 - idx % 8)) != 0 {
                Color::WHITE
            } else {
                Color::BLACK
            }
        },
        /// 数组 1D 位图 (每位表示一个像素，1为白色，0为黑色)
        Vec1DBitmap(&'buf Vec<u8>) => |buf: &[u8], dx, dy, width, _| {
            let idx = (dy * width + dx) as usize;
            if buf[idx / 8] & (1 << (7 - idx % 8)) != 0 {
                Color::WHITE
            } else {
                Color::BLACK
            }
        },
        /// 拥有所有权的 1D 位图 (每位表示一个像素，1为白色，0为黑色)
        Owned1DBitmap(Vec<u8>) => |buf: &[u8], dx, dy, width, _| {
            let idx = (dy * width + dx) as usize;
            if buf[idx / 8] & (1 << (7 - idx % 8)) != 0 {
                Color::WHITE
            } else {
                Color::BLACK
            }
        },
    },

    /// 灰度图 (每个字节表示一个像素的灰度值)
    Gray: {
        /// 静态 1D 灰度图 (每个字节表示一个像素的灰度值)
        Static1DGray(&'static [u8]) => |buf: &[u8], dx, dy, width, _| Color::gray(buf[(dy * width + dx) as usize]),
        /// 切片 1D 灰度图 (每个字节表示一个像素的灰度值)
        Slice1DGray(&'buf [u8]) => |buf: &[u8], dx, dy, width, _| Color::gray(buf[(dy * width + dx) as usize]),
        /// 数组 1D 灰度图 (每个字节表示一个像素的灰度值)
        Vec1DGray(&'buf Vec<u8>) => |buf: &[u8], dx, dy, width, _| Color::gray(buf[(dy * width + dx) as usize]),
        /// 拥有所有权的 1D 灰度图 (每个字节表示一个像素的灰度值)
        Owned1DGray(Vec<u8>) => |buf: &[u8], dx, dy, width, _| Color::gray(buf[(dy * width + dx) as usize]),

        /// 静态 2D 灰度图 (每个字节表示一个像素的灰度值)
        Static2DGray(&'static [&'static [u8]]) => |buf: &[&[u8]], dx, dy, _, _| Color::gray(buf[dy as usize][dx as usize]),
        /// 切片 2D 灰度图 (每个字节表示一个像素的灰度值)
        Slice2DGray(&'buf [&'buf [u8]]) => |buf: &[&[u8]], dx, dy, _, _| Color::gray(buf[dy as usize][dx as usize]),
        /// 数组 2D 灰度图 (每个字节表示一个像素的灰度值)
        Vec2DGray(&'buf Vec<Vec<u8>>) => |buf: &[Vec<u8>], dx, dy, _, _| Color::gray(buf[dy as usize][dx as usize]),
        /// 拥有所有权的 2D 灰度图 (每个字节表示一个像素的灰度值)
        Owned2DGray(Vec<Vec<u8>>) => |buf: &[Vec<u8>], dx, dy, _, _| Color::gray(buf[dy as usize][dx as usize]),
    },

    /// 次像素图 (每个像素存储 RGB 通道的遮罩值而非合并为一个灰度值)
    Subpix: {
        /// 静态 1D 次像素图 (每个像素存储 RGB 通道的遮罩值而非合并为一个灰度值)
        Static1DSubpix(&'static [Color]) => |buf: &[Color], dx, dy, width, _| buf[(dy * width + dx) as usize],
        /// 切片 1D 次像素图 (每个像素存储 RGB 通道的遮罩值而非合并为一个灰度值)
        Slice1DSubpix(&'buf [Color]) => |buf: &[Color], dx, dy, width, _| buf[(dy * width + dx) as usize],
        /// 数组 1D 次像素图 (每个像素存储 RGB 通道的遮罩值而非合并为一个灰度值)
        Vec1DSubpix(&'buf Vec<Color>) => |buf: &[Color], dx, dy, width, _| buf[(dy * width + dx) as usize],
        /// 拥有所有权的 1D 次像素图 (每个像素存储 RGB 通道的遮罩值而非合并为一个灰度值)
        Owned1DSubpix(Vec<Color>) => |buf: &[Color], dx, dy, width, _| buf[(dy * width + dx) as usize],

        /// 静态 2D 次像素图 (每个像素存储 RGB 通道的遮罩值而非合并为一个灰度值)
        Static2DSubpix(&'static [&'static [Color]]) => |buf: &[&[Color]], dx, dy, _, _| buf[dy as usize][dx as usize],
        /// 切片 2D 次像素图 (每个像素存储 RGB 通道的遮罩值而非合并为一个灰度值)
        Slice2DSubpix(&'buf [&'buf [Color]]) => |buf: &[&[Color]], dx, dy, _, _| buf[dy as usize][dx as usize],
        /// 数组 2D 次像素图 (每个像素存储 RGB 通道的遮罩值而非合并为一个灰度值)
        Vec2DSubpix(&'buf Vec<Vec<Color>>) => |buf: &[Vec<Color>], dx, dy, _, _| buf[dy as usize][dx as usize],
        /// 拥有所有权的 2D 次像素图 (每个像素存储 RGB 通道的遮罩值而非合并为一个灰度值)
        Owned2DSubpix(Vec<Vec<Color>>) => |buf: &[Vec<Color>], dx, dy, _, _| buf[dy as usize][dx as usize],
    },

    /// 彩色图 (每个像素存储颜色和透明度信息)
    Colored: {
        /// 静态 1D 彩色图 (每个像素存储颜色和透明度信息)
        Static1DColored(&'static [Color]) => |buf: &[Color], dx, dy, width, _| buf[(dy * width + dx) as usize],
        /// 切片 1D 彩色图 (每个像素存储颜色和透明度信息)
        Slice1DColored(&'buf [Color]) => |buf: &[Color], dx, dy, width, _| buf[(dy * width + dx) as usize],
        /// 数组 1D 彩色图 (每个像素存储颜色和透明度信息)
        Vec1DColored(&'buf Vec<Color>) => |buf: &[Color], dx, dy, width, _| buf[(dy * width + dx) as usize],
        /// 拥有所有权的 1D 彩色图 (每个像素存储颜色和透明度信息)
        Owned1DColored(Vec<Color>) => |buf: &[Color], dx, dy, width, _| buf[(dy * width + dx) as usize],

        /// 静态 2D 彩色图 (每个像素存储颜色和透明度信息)
        Static2DColored(&'static [&'static [Color]]) => |buf: &[&[Color]], dx, dy, _, _| buf[dy as usize][dx as usize],
        /// 切片 2D 彩色图 (每个像素存储颜色和透明度信息)
        Slice2DColored(&'buf [&'buf [Color]]) => |buf: &[&[Color]], dx, dy, _, _| buf[dy as usize][dx as usize],
        /// 数组 2D 彩色图 (每个像素存储颜色和透明度信息)
        Vec2DColored(&'buf Vec<Vec<Color>>) => |buf: &[Vec<Color>], dx, dy, _, _| buf[dy as usize][dx as usize],
        /// 拥有所有权的 2D 彩色图 (每个像素存储颜色和透明度信息)
        Owned2DColored(Vec<Vec<Color>>) => |buf: &[Vec<Color>], dx, dy, _, _| buf[dy as usize][dx as usize],
    },
);

/// 由字体渲染器返回的字形
/// - 应当在得到字形后立即转换为 `FontBufferGlyph`，以使字形提供器可以立刻复用内部缓冲区。
/// - 不要缓存这个结构。
///
/// Glyph returned by the font renderer
/// - Should be converted to `FontBufferGlyph` immediately after obtaining, so that the glyph provider can reuse internal buffers immediately.
/// - Do not cache this structure.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontGlyph<'buf> {
    pub buf: FontGlyphBuffer<'buf>,
    pub code: char,
    /// 字形位图的左边距 (字形位图相对于字符位置的水平偏移)
    pub left: i32,
    /// 字形位图的上边距 (字形位图相对于字符位置的垂直偏移)
    pub top: i32,
    /// 字形位图的宽度
    pub width: i32,
    /// 字形位图的高度
    pub height: i32,
    /// 字形的水平前进距离 (当前字形的左边距到下一个字形的左边距的距离)
    pub advance: i32,
    /// 字形的行高 (当前行基线到下一行基线的距离)
    pub line_height: i32,
}

impl FontGlyph<'_> {
    /// 将字形的位图内容居中对齐到指定的目标宽度和高度。
    ///
    /// 有些字体的字形大小可能会与终端的单元大小不一致，此函数将字形的位图内容居中对齐到指定的目标宽度和高度，以适应终端单元格的大小。
    ///
    /// Align the bitmap content of the glyph to the specified target width and height.
    ///
    /// Some fonts may have glyph sizes that do not match the terminal cell size, this function will align the bitmap content of the glyph to the specified target width and height to fit the terminal cell size.
    pub fn align_to(self, target_width: u32, target_height: u32) -> Self {
        Self {
            buf: self.buf,
            code: self.code,
            left: self.left + (target_width as i32 - self.advance) / 2,
            top: self.top + (target_height as i32 - self.line_height) / 2,
            width: self.width,
            height: self.height,
            advance: target_width as i32,
            line_height: target_height as i32,
        }
    }
}
