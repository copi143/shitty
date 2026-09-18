use aligned_vec::{ABox, avec};
use core::ops::{Deref, DerefMut};

use crate::color::{AnsiColor, AnsiNamedColor, Color, IColor};
use crate::palette::Palette;

static_assert!(core::mem::size_of::<ColorScheme<Color>>() == core::mem::size_of::<usize>());

/// 512 色调色板。包含 16 个 ANSI 色 + 216 个 6x6x6 立方体色 + 24 阶灰度 + 前景/背景/光标色。
///
/// 索引布局：
/// - 0-15: ANSI 16 色
/// - 16-231: 6x6x6 颜色立方体 (216 色)
/// - 232-255: 24 阶灰度
/// - 256: 前景色
/// - 257: 背景色
/// - 258+: 其他特殊色（光标等）
///
/// 512-color palette. Contains 16 ANSI colors + 216 6x6x6 cube colors + 24 grayscale + fg/bg/cursor.
///
/// Index layout:
/// - 0-15: ANSI 16 colors
/// - 16-231: 6x6x6 color cube (216 colors)
/// - 232-255: 24-step grayscale
/// - 256: Foreground
/// - 257: Background
/// - 258+: Other special colors (cursor, etc.)
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorScheme<T: IColor>(ABox<[T; 512]>);

impl<T: IColor> Default for ColorScheme<T> {
    fn default() -> Self {
        Self::from(Palette::default())
    }
}

impl<T: IColor> Deref for ColorScheme<T> {
    type Target = [T; 512];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: IColor> DerefMut for ColorScheme<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[rustfmt::skip]
impl<T: IColor> ColorScheme<T> {
    /// 从 [`Palette`] 创建一个新的颜色方案。
    ///
    /// Create a new color scheme from a [`Palette`].
    pub fn new(palette: &Palette) -> Self {
        Self::from(palette)
    }

    /// 通过 [`AnsiColor`] 索引获取颜色。
    ///
    /// Get the color by [`AnsiColor`] index.
    pub fn get(&self, color: AnsiColor) -> T {
        match color {
            AnsiColor::Spec(rgb) => T::from(Color::from(rgb)),
            AnsiColor::Named(named) => self[named as usize],
            AnsiColor::Indexed(index) => self[index as usize],
        }
    }

    pub fn black(&self) -> T { self[AnsiNamedColor::Black as usize] }
    pub fn red(&self) -> T { self[AnsiNamedColor::Red as usize] }
    pub fn green(&self) -> T { self[AnsiNamedColor::Green as usize] }
    pub fn yellow(&self) -> T { self[AnsiNamedColor::Yellow as usize] }
    pub fn blue(&self) -> T { self[AnsiNamedColor::Blue as usize] }
    pub fn magenta(&self) -> T { self[AnsiNamedColor::Magenta as usize] }
    pub fn cyan(&self) -> T { self[AnsiNamedColor::Cyan as usize] }
    pub fn white(&self) -> T { self[AnsiNamedColor::White as usize] }

    /// 获取前景色。
    ///
    /// Get the foreground color.
    pub fn foreground(&self) -> T {
        self[AnsiNamedColor::Foreground as usize]
    }

    /// 获取背景色。
    ///
    /// Get the background color.
    pub fn background(&self) -> T {
        self[AnsiNamedColor::Background as usize]
    }

    /// 获取光标色。
    ///
    /// Get the cursor color.
    pub fn cursor(&self) -> T {
        self[AnsiNamedColor::Cursor as usize]
    }
}

impl<T: IColor> From<&Palette<'_>> for ColorScheme<T> {
    /// 从 [`Palette`] 构建 512 色调色板，展开 ANSI 色、6x6x6 立方体和灰度。
    ///
    /// Build a 512-color palette from a [`Palette`], expanding ANSI colors, 6x6x6 cube, and grayscale.
    fn from(palette: &Palette) -> Self {
        const fn palette256_scale(c: usize) -> u8 {
            if c == 0 { 0 } else { (c * 40 + 55) as u8 }
        }

        let mut colors = ABox::new(0, [T::default(); 512]);

        // 0-15: ANSI 16 色
        for index in 0..16 {
            colors[index] = palette.ansi_colors[index].into();
        }

        // 16-231: 6x6x6 颜色立方体 (216 色)
        for index in 0..216 {
            let r = palette256_scale(index / 36);
            let g = palette256_scale(index % 36 / 6);
            let b = palette256_scale(index % 6);
            colors[index + 16] = Color::rgb(r, g, b).into();
        }

        // 232-255: 24 阶灰度
        for index in 0..24 {
            let luminance = (index * 10 + 8) as u8;
            colors[index + 16 + 216] = Color::rgb(luminance, luminance, luminance).into();
        }

        // 256+: 前景/背景/光标
        colors[AnsiNamedColor::Foreground as usize] = palette.foreground.into();
        colors[AnsiNamedColor::Background as usize] = palette.background.into();
        colors[AnsiNamedColor::Cursor as usize] = palette.cursor.into();

        Self(colors)
    }
}
