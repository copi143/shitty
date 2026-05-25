#![allow(dead_code)]

use core::fmt::{Debug, Display};
use core::hash::Hash;
use unicode_width::UnicodeWidthChar;

use crate::buffer::types::Tick;
use crate::color::{AnsiColor, AnsiNamedColor, ColorScheme, IColor};
use crate::font::{FontBuffer, FontChar};
use crate::font::{RenderResultRef, RenderResultType, RenderResultUnion};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontFlags(u8);

impl FontFlags {
    /// 粗体
    ///
    /// Bold text
    pub const BOLD: u8 = 1 << 0;
    /// 斜体
    ///
    /// Italic text
    pub const ITALIC: u8 = 1 << 1;

    #[inline(always)]
    pub const fn default() -> Self {
        Self(0)
    }

    #[inline(always)]
    pub const fn bold() -> Self {
        Self(Self::BOLD)
    }

    #[inline(always)]
    pub const fn italic() -> Self {
        Self(Self::ITALIC)
    }

    #[inline(always)]
    pub const fn bold_italic() -> Self {
        Self(Self::BOLD | Self::ITALIC)
    }

    #[inline(always)]
    pub const fn is_bold(&self) -> bool {
        self.0 & Self::BOLD != 0
    }

    #[inline(always)]
    pub const fn is_italic(&self) -> bool {
        self.0 & Self::ITALIC != 0
    }

    #[inline(always)]
    pub const fn set_bold(&mut self, bold: bool) {
        if bold {
            self.0 |= Self::BOLD;
        } else {
            self.0 &= !Self::BOLD;
        }
    }

    #[inline(always)]
    pub const fn set_italic(&mut self, italic: bool) {
        if italic {
            self.0 |= Self::ITALIC;
        } else {
            self.0 &= !Self::ITALIC;
        }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CharFlags: u8 {
        /// 下划线
        ///
        /// Underlined text
        const UNDERLINE = 1 << 2;
        /// 删除线
        ///
        /// Strikethrough text
        const STRIKETHROUGH = 1 << 3;
    }
}

#[must_use]
fn widthof(ch: char) -> u8 {
    ch.width().unwrap_or(1).max(1) as u8
}

static_assert!(core::mem::size_of::<Char>() == 16);

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Char {
    /// The character itself.
    pub ch: char,

    /// 单元格中字符的宽度。<br />
    /// 对于 ASCII 字符通常为 1，但对于宽字符可能更大。<br />
    /// 零宽字符被视为宽度为 1。<br />
    /// 值 0 保留用于占位符。<br />
    ///
    /// ---
    ///
    /// Width of the character in cells.<br />
    /// This is usually 1 for ASCII characters, but can be greater for wide characters.<br />
    /// Zero-width characters are considered to have a width of 1.<br />
    /// Value 0 is reserved for placeholders.<br />
    pub width: u8,

    /// See [`FontFlags`] for details.
    pub font: FontFlags,

    /// See [`CharFlags`] for details.
    pub flags: CharFlags,

    /// 如果单元格的内容发生了变化但尚未被刷新，则该字段为 true。
    ///
    /// If the content of the cell has changed but has not yet been flushed, this field is true.
    pub dirty: bool,

    /// Foreground index
    pub fg: AnsiColor,

    /// Background index
    pub bg: AnsiColor,
}

impl Default for Char {
    fn default() -> Self {
        Self::empty()
    }
}

impl Display for Char {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.fg)?;
        write!(f, "{:#}", self.bg)?;
        write!(f, "{}", self.ch)?;
        write!(f, "\x1b[0m")?;
        Ok(())
    }
}

impl Char {
    #[must_use]
    const fn new(ch: char, width: u8) -> Self {
        Self {
            ch,
            width,
            font: FontFlags::default(),
            flags: CharFlags::empty(),
            dirty: true,
            fg: AnsiColor::Named(AnsiNamedColor::Foreground),
            bg: AnsiColor::Named(AnsiNamedColor::Background),
        }
    }

    /// 终端中未被文本填充的部分使用此对象
    #[must_use]
    pub const fn empty() -> Self {
        Self::new('\0', 1)
    }

    /// 占位符
    #[must_use]
    pub const fn placeholder() -> Self {
        Self::new('\0', 0)
    }

    /// 用空格填充时使用此对象
    #[must_use]
    pub const fn space() -> Self {
        Self::new(' ', 1)
    }

    /// 将当前单元格的内容清空，但保留属性。
    /// - 字符被设置为 `\0`，宽度被设置为 1
    ///
    /// ---
    ///
    /// Clear the content of the current cell, but keep the attributes.
    /// - The character is set to `\0` and the width is set to 1.
    #[must_use]
    pub const fn as_empty(mut self) -> Self {
        self.ch = '\0';
        self.width = 1;
        self.dirty = true;
        self
    }

    /// 当前一个单元格的内容宽度大于 1 时，后续单元格使用此对象作为占位符
    /// - 占位符的属性与第一个单元格相同，但字符为 `\0`，宽度为 0
    ///
    /// ---
    ///
    /// When the content width of a cell is greater than 1, subsequent cells use this object as a placeholder.
    /// - The placeholder has the same properties as the first cell, but with character `\0` and width 0.
    #[must_use]
    pub const fn as_placeholder(mut self) -> Self {
        self.ch = '\0';
        self.width = 0;
        self.dirty = true;
        self
    }

    #[must_use]
    pub const fn as_space(mut self) -> Self {
        self.ch = ' ';
        self.width = 1;
        self.dirty = true;
        self
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.ch == '\0' && self.width == 1
    }

    #[must_use]
    pub const fn is_placeholder(&self) -> bool {
        self.ch == '\0' && self.width == 0
    }

    #[must_use]
    pub const fn is_space(&self) -> bool {
        self.ch == ' '
    }

    #[must_use]
    pub fn with_content(mut self, content: char) -> Self {
        self.ch = content;
        self.width = widthof(content);
        self.dirty = true;
        self
    }

    #[must_use]
    pub fn with_color(mut self, fg: AnsiColor, bg: AnsiColor) -> Self {
        self.fg = fg;
        self.bg = bg;
        self.dirty = true;
        self
    }

    #[must_use]
    pub fn invert_color(mut self) -> Self {
        core::mem::swap(&mut self.fg, &mut self.bg);
        self.dirty = true;
        self
    }

    #[must_use]
    pub fn as_font_char(&self) -> FontChar {
        FontChar::new(self.ch, self.width, self.font.is_bold(), self.font.is_italic())
    }
}

static_assert!(core::mem::size_of::<Cell<crate::Color>>() == 32);

/// 内部有数据时不要随便 clone，可能导致缓存的字体渲染结果未被释放。<br />
/// clone 出来的也要调用 unref 释放。<br />
///
/// --
///
/// Do not clone casually when there is internal data, which may cause the cached font rendering result to not be released.<br />
/// The cloned one should also call unref to release.<br />
#[repr(C, align(16))]
pub struct Cell<T: IColor> {
    pub ch: char,
    pub width: u8,
    pub font: FontFlags,
    pub flags: CharFlags,
    rendt: RenderResultType,
    rendr: RenderResultUnion,
    pub fg: T,
    pub bg: T,
    pub tick: Tick,
}

impl<T: IColor> Debug for Cell<T> {
    #[expect(unsafe_code)]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Cell")
            .field("ch", &self.ch)
            .field("font", &self.font)
            .field("flags", &self.flags)
            .field("rend", &unsafe { self.rendr.as_ref(&self.rendt) })
            .field("fg", &self.fg)
            .field("bg", &self.bg)
            .field("tick", &self.tick)
            .finish()
    }
}

impl<T: IColor> Clone for Cell<T> {
    #[expect(unsafe_code)]
    fn clone(&self) -> Self {
        Self {
            ch: self.ch,
            width: self.width,
            font: self.font,
            flags: self.flags,
            rendt: self.rendt.clone(),
            rendr: unsafe { self.rendr.clone(&self.rendt) },
            fg: self.fg,
            bg: self.bg,
            tick: self.tick,
        }
    }
}

impl<T: IColor> Drop for Cell<T> {
    fn drop(&mut self) {
        debug_assert!(
            self.rendt == RenderResultType::Empty || self.rendt == RenderResultType::Blank,
            "Cell should not be dropped with a non-empty render result.\n{:?}",
            self,
        );
    }
}

#[expect(unsafe_code)]
impl<T: IColor> Cell<T> {
    pub fn drop(mut self, font: &mut FontBuffer) {
        if self.rendt != RenderResultType::Empty && self.rendt != RenderResultType::Blank {
            let rendt = core::mem::replace(&mut self.rendt, RenderResultType::Empty);
            let rendr = core::mem::replace(&mut self.rendr, RenderResultUnion::empty());
            font.unref(self.as_font_char(), unsafe { rendt.merge(rendr) });
        }
    }

    #[must_use]
    pub fn new(scheme: &ColorScheme<T>) -> Self {
        Self {
            ch: '\0',
            width: 1,
            font: FontFlags::default(),
            flags: CharFlags::empty(),
            rendt: RenderResultType::Blank,
            rendr: RenderResultUnion::blank(),
            fg: scheme.foreground(),
            bg: scheme.background(),
            tick: Tick::default(),
        }
    }

    #[must_use]
    pub fn from(font: &mut FontBuffer, scheme: &ColorScheme<T>, ch: Char) -> Self {
        let (rendt, rendr) = font.get(ch.as_font_char()).split();
        Self {
            ch: ch.ch,
            width: ch.width,
            font: ch.font,
            flags: ch.flags,
            rendt,
            rendr,
            fg: scheme.get(ch.fg),
            bg: scheme.get(ch.bg),
            tick: Tick::default(),
        }
    }

    #[must_use]
    fn as_font_char(&self) -> FontChar {
        FontChar::new(self.ch, self.width, self.font.is_bold(), self.font.is_italic())
    }

    pub fn rend_ref(&self) -> RenderResultRef<'_> {
        unsafe { self.rendr.as_ref(&self.rendt) }
    }

    pub fn clear(&mut self, scheme: &ColorScheme<T>, font: &mut FontBuffer, clear_color: bool) {
        let font_char = self.as_font_char();
        self.ch = ' ';
        self.font = FontFlags::default();
        self.flags = CharFlags::empty();
        let rendt = core::mem::replace(&mut self.rendt, RenderResultType::Blank);
        let rendr = core::mem::replace(&mut self.rendr, RenderResultUnion::blank());
        if clear_color {
            self.fg = scheme.foreground();
            self.bg = scheme.background();
        }
        font.unref(font_char, unsafe { rendt.merge(rendr) });
    }

    pub fn update(&mut self, scheme: &ColorScheme<T>, font: &mut FontBuffer, ch: Char, tick: Tick) -> bool {
        let old_fc = self.as_font_char();
        self.ch = ch.ch;
        self.width = ch.width;
        self.font = ch.font;
        self.flags = ch.flags;
        self.fg = scheme.get(ch.fg);
        self.bg = scheme.get(ch.bg);
        let new_fc = self.as_font_char();
        let (new_rendt, new_rendr) = if ch.width == 0 {
            (RenderResultType::Empty, RenderResultUnion::empty())
        } else {
            font.get(new_fc).split()
        };
        let rendt = core::mem::replace(&mut self.rendt, new_rendt);
        let rendr = core::mem::replace(&mut self.rendr, new_rendr);
        font.unref(old_fc, unsafe { rendt.merge(rendr) });
        self.tick = tick;
        true
    }
}

/// 每行的切片，包含该行的字符切片以及脏标记
#[derive(Debug)]
pub struct LineSlice<'chars> {
    /// The slice of characters in the line.
    pub chars: &'chars mut [Char],
    /// Whether the line is dirty (i.e., has changes that have not been flushed).
    pub dirty: bool,
    /// Whether the entire line is dirty (i.e., all characters in the line are dirty).
    pub all_dirty: bool,
}

#[expect(unsafe_code)]
impl LineSlice<'_> {
    pub fn get(&self, x: u32) -> &Char {
        debug_assert!((x as usize) < self.chars.len(), "X coordinate out of bounds");
        unsafe { self.chars.get_unchecked(x as usize) }
    }

    pub fn get_mut(&mut self, x: u32) -> &mut Char {
        debug_assert!((x as usize) < self.chars.len(), "X coordinate out of bounds");
        unsafe { self.chars.get_unchecked_mut(x as usize) }
    }
}

#[derive(Debug)]
pub struct CellSlice<'cells, T: IColor> {
    pub cells: &'cells mut [Cell<T>],
}

#[expect(unsafe_code)]
impl<T: IColor> CellSlice<'_, T> {
    pub fn get(&self, x: u32) -> &Cell<T> {
        debug_assert!((x as usize) < self.cells.len(), "X coordinate out of bounds");
        unsafe { self.cells.get_unchecked(x as usize) }
    }

    pub fn get_mut(&mut self, x: u32) -> &mut Cell<T> {
        debug_assert!((x as usize) < self.cells.len(), "X coordinate out of bounds");
        unsafe { self.cells.get_unchecked_mut(x as usize) }
    }
}
