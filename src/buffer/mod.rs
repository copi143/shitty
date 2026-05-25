use alloc::boxed::Box;
use alloc::string::String;
use core::ops::Range;
use core::time::Duration;

use crate::buffer::{alternate::Alternate, primary::Primary};
use crate::color::WrappedDrawable;
use crate::font::{FontBuffer, FontRenderer};
use crate::{CursorShape, Palette};

mod r#trait;
use r#trait::Buffer;

mod cell;
use cell::{Cell, CellSlice, LineSlice};
pub use cell::{Char, CharFlags};

#[macro_use]
mod screen;
pub use screen::{Screen, TerminalScreen};

mod alternate;
mod primary;

mod drawable;
pub use drawable::{Drawable, OwnedDrawable};

mod types;
pub use types::{AutoWrap, BufMode, CursorPosition};
use types::{Cursor, Tick};

/// 终端的缓冲区实现
pub struct TerminalBuffer {
    /// 屏幕缓冲区
    screen: TerminalScreen,
    /// 主字符缓冲区
    primary: Primary,
    /// 备用字符缓冲区
    alternate: Alternate,
    /// 是否在备用屏幕模式
    alt_screen_mode: bool,
    /// 字体缓冲区
    font: FontBuffer,
    /// 在下一个输入字符时换行
    wrap_next: bool,
    /// 自动换行模式
    auto_wrap: AutoWrap,
    scroll_region: Option<Range<u32>>,
    cursor: Cursor,
}

impl Drop for TerminalBuffer {
    fn drop(&mut self) {
        self.screen.clear(&mut self.font);
    }
}

assert_send_sync!(TerminalBuffer);

impl TerminalBuffer {
    #[must_use]
    pub fn width(&self) -> u32 {
        self.screen.width
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.screen.height
    }

    #[must_use]
    pub fn cursor(&self) -> CursorPosition {
        self.cursor.pos
    }

    pub fn set_cursor(&mut self, col: u32, row: u32) {
        self.cursor.pos.col = col.min(self.screen.width.saturating_sub(1));
        self.cursor.pos.row = row.min(self.screen.height.saturating_sub(1));
        self.wrap_next = false;
    }

    pub fn cursor_shape(&self) -> CursorShape {
        self.cursor.shape
    }

    pub fn set_cursor_shape(&mut self, shape: CursorShape) {
        self.cursor.shape = shape;
    }

    pub fn has_scroll_region(&self) -> bool {
        self.scroll_region.is_some()
    }

    pub fn scroll_region(&self) -> Range<u32> {
        self.scroll_region.clone().unwrap_or(0..self.screen.height)
    }

    pub fn set_scroll_region(&mut self, region: Range<u32>) {
        debug_assert!(region.start < region.end, "Invalid scroll region: {region:?}");
        debug_assert!(region.end <= self.screen.height, "Scroll region out of bounds: {region:?}");
        self.scroll_region = Some(region);
    }

    pub fn clear_scroll_region(&mut self) {
        self.scroll_region = None;
    }

    pub fn font_size(&self) -> (u32, u32) {
        (self.font.width, self.font.height)
    }

    pub fn auto_wrap(&self) -> AutoWrap {
        self.auto_wrap
    }

    pub fn set_auto_wrap(&mut self, mode: AutoWrap) {
        self.auto_wrap = mode;
    }

    pub fn show_cursor(&mut self) {
        self.cursor.visible = true;
    }

    pub fn hide_cursor(&mut self) {
        self.cursor.visible = false;
    }
}

impl TerminalBuffer {
    /// 创建一个新的终端缓冲区
    /// - 默认主缓冲区 1024 行历史记录
    #[must_use]
    pub fn new(buf_mode: BufMode) -> Self {
        let font = FontBuffer::default();
        Self {
            screen: TerminalScreen::new(80, 24, buf_mode),
            primary: Primary::new(80, 24, 1024),
            alternate: Alternate::new(),
            alt_screen_mode: false,
            font,
            wrap_next: false,
            auto_wrap: AutoWrap::default(),
            scroll_region: None,
            cursor: Cursor::default(),
        }
    }

    /// 调整终端缓冲区的大小。
    /// - 即使终端的字符计大小没有变化，也应该在像素大小变化时调用此方法，以确保字体和屏幕缓存正确更新。
    ///
    /// ---
    ///
    /// Resize the terminal buffer.
    /// - This method should be called even if the character cell size of the terminal does not change,
    ///   to ensure that the font and screen cache are correctly updated when the pixel size changes.
    pub fn resize(&mut self, display_width: u32, display_height: u32) {
        assert!(self.font.width > 0 && self.font.height > 0);
        assert!(display_width > 0 && display_height > 0);

        let width = display_width / self.font.width;
        let height = display_height / self.font.height;

        assert!(width > 0 && height > 0, "Terminal size too small");

        self.screen.invalidate_outer_cache();

        if self.screen.width == width && self.screen.height == height {
            return;
        }

        if self.alt_screen_mode {
            self.alternate.saved_cursor_pos = self.cursor.pos;
            self.alternate.resize(width, height, Char::empty());
            self.cursor.pos = self.alternate.saved_cursor_pos;
        } else {
            self.primary.saved_cursor_pos = self.cursor.pos;
            self.primary.resize(width, height, Char::empty());
            self.cursor.pos = self.primary.saved_cursor_pos;
        }
        if self.cursor.pos.col >= width {
            self.cursor.pos.col = width - 1;
            self.wrap_next = true;
        }

        self.screen.reinit(width, height, &mut self.font);

        self.wrap_next = false;
        self.scroll_region = None;
    }

    pub fn add_renderer(&mut self, font_renderer: Box<dyn FontRenderer>) {
        self.font.add_renderer(font_renderer);
        self.screen.invalidate_inner_cache();
    }

    pub fn set_color_scheme(&mut self, palette: &Palette) {
        self.screen.set_color_scheme(palette);
        self.screen.clear(&mut self.font);
    }

    /// Check if currently in alternate screen mode.
    pub fn is_alternate(&self) -> bool {
        self.alt_screen_mode
    }

    /// Enter alternate screen mode.
    pub fn enter_alternate(&mut self) {
        assert!(!self.alt_screen_mode, "Cannot enter alternate screen while already in it");
        self.alt_screen_mode = true;
        self.alternate.init(self.screen.width, self.screen.height, Char::empty());
        self.primary.saved_cursor_pos = self.cursor.pos;
        self.cursor.pos = self.alternate.saved_cursor_pos;
        self.wrap_next = false;
        if self.cursor.pos.col >= self.screen.width {
            self.cursor.pos.col = self.screen.width - 1;
            self.wrap_next = true;
        }
    }

    /// Exit alternate screen mode.
    pub fn exit_alternate(&mut self) {
        assert!(self.alt_screen_mode, "Cannot exit alternate screen while not in it");
        self.alt_screen_mode = false;
        self.alternate.deinit();
        if self.screen.width == self.primary.width() && self.screen.height == self.primary.height() {
            return;
        }
        self.primary.resize(self.screen.width, self.screen.height, Char::empty());
        self.alternate.saved_cursor_pos = self.cursor.pos;
        self.cursor.pos = self.primary.saved_cursor_pos;
        self.wrap_next = false;
        if self.cursor.pos.col >= self.screen.width {
            self.cursor.pos.col = self.screen.width - 1;
            self.wrap_next = true;
        }
    }
}

impl TerminalBuffer {
    /// 获取指定位置的字符
    ///
    /// ---
    ///
    /// Get the character at the specified position.
    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> Char {
        if self.alt_screen_mode {
            self.alternate.get(x, y)
        } else {
            self.primary.get(x, y)
        }
    }

    /// 设置指定位置的字符
    ///
    /// ---
    ///
    /// Set the character at the specified position.
    pub fn set(&mut self, x: u32, y: u32, cell: Char, fill: Char) {
        debug_assert!(cell.dirty, "Cell character must be dirty when setting");
        debug_assert!(fill.dirty, "Fill character must be dirty when setting");
        debug_assert!(!cell.is_placeholder(), "Cell character must not be a placeholder");
        debug_assert!(!fill.is_placeholder(), "Fill character must not be a placeholder");
        if self.alt_screen_mode {
            self.alternate.set(x, y, cell);
        } else {
            self.primary.set(x, y, cell);
        }
    }

    /// 向光标处插入一个字符，并根据自动换行设置调整光标位置
    ///
    /// ---
    ///
    /// Insert a character at the cursor position and adjust the cursor position according to the auto-wrap settings.
    pub fn put(&mut self, cell: Char, fill: Char) {
        debug_assert!(!fill.is_placeholder(), "Fill character must not be a placeholder");
        if self.wrap_next {
            self.newline(fill);
        }

        debug_assert!(cell.width > 0, "Cell width must be greater than 0");

        if self.cursor.pos.col + cell.width as u32 > self.screen.width {
            match self.auto_wrap {
                AutoWrap::Disabled => self.cr(),
                AutoWrap::Immediate | AutoWrap::Delayed => self.newline(fill),
            }
        }

        self.set(self.cursor.pos.col, self.cursor.pos.row, cell, fill);
        self.cursor.pos.col += cell.width as u32;

        match self.auto_wrap {
            AutoWrap::Disabled => {
                if self.cursor.pos.col == self.screen.width {
                    self.cr()
                }
            }
            AutoWrap::Immediate => {
                if self.cursor.pos.col == self.screen.width {
                    self.newline(fill);
                }
            }
            AutoWrap::Delayed => {
                if self.cursor.pos.col == self.screen.width {
                    self.wrap_next = true;
                    self.cursor.pos.col = self.screen.width - 1;
                }
            }
        }

        debug_assert!(self.cursor.pos.col < self.screen.width);
        debug_assert!(self.cursor.pos.row < self.screen.height);
    }

    /// Mark the end of line at the specified row.
    /// - This is typically called when a newline character is processed.
    pub fn endl(&mut self, y: u32) {
        if self.alt_screen_mode {
            self.alternate.endl(y);
        } else {
            self.primary.endl(y);
        }
    }

    pub fn cr(&mut self) {
        self.cursor.pos.col = 0;
        self.wrap_next = false;
    }

    pub fn lf(&mut self, fill: Char) {
        debug_assert!(!fill.is_placeholder(), "Fill character must not be a placeholder");
        self.endl(self.cursor.pos.row);
        if self.cursor.pos.row == self.scroll_region().end - 1 {
            self.scroll(1, fill);
        } else if self.cursor.pos.row < self.screen.height - 1 {
            self.cursor.pos.row += 1;
        }
    }

    pub fn newline(&mut self, fill: Char) {
        debug_assert!(!fill.is_placeholder(), "Fill character must not be a placeholder");
        self.cr();
        self.lf(fill);
    }

    pub fn clear_line(&mut self, y: u32, fill: Char) {
        debug_assert!(!fill.is_placeholder(), "Fill character must not be a placeholder");
        if self.alt_screen_mode {
            self.alternate.clear_line(y, fill);
        } else {
            self.primary.clear_line(y, fill);
        }
    }

    /// Clear the contents of the visible area.
    /// - The `fill` parameter is used to fill the cleared area.
    /// - This does not affect the scrollback history.
    pub fn clear(&mut self, fill: Char) {
        debug_assert!(!fill.is_placeholder(), "Fill character must not be a placeholder");
        if self.alt_screen_mode {
            self.alternate.clear(fill);
        } else {
            self.primary.clear(fill);
        }
    }

    /// Clear the entire buffer, including the scrollback history.
    /// - The `cell` parameter is used to fill the cleared area.
    /// - This resets the buffer to its initial empty state.
    pub fn clear_all(&mut self, fill: Char) {
        debug_assert!(!fill.is_placeholder(), "Fill character must not be a placeholder");
        if self.alt_screen_mode {
            self.alternate.clear_all(fill);
        } else {
            self.primary.clear_all(fill);
        }
    }
}

impl TerminalBuffer {
    /// 将缓冲区的内容刷新到可绘制表面上。
    /// - 如果可能，只刷新整个屏幕上改变的区域
    /// - 对于光标位置，每帧都刷新
    /// - 如果光标移动，重新绘制旧位置和新位置的字符，以正确显示光标
    ///
    /// ---
    ///
    /// Flush the buffer to the drawable surface.
    /// - If possible, only flush the changed regions of the screen.
    /// - For cursor position, always flush it every frame.
    /// - If the cursor moves, redraw the characters at the old and new positions to properly display the cursor.
    pub fn flush(&mut self, drawable: WrappedDrawable, _time: Duration) -> bool {
        if self.alt_screen_mode {
            self.screen.flush(drawable, &mut self.alternate, &mut self.font, &mut self.cursor);
        } else {
            self.screen.flush(drawable, &mut self.primary, &mut self.font, &mut self.cursor);
        };
        self.font.tick();
        true
    }

    /// 将从 `begin` 到 `end` 范围内的缓冲区内容转换为字符串。
    /// - 主要用于拷贝缓冲区内容到剪贴板。
    /// - 注意文本的样式会丢失，且换行符仅在行尾存在（即使某些行被自动换行了），以保持文本的原始结构。
    ///
    /// ---
    ///
    /// Convert the buffer content in the range from `begin` to `end` into a string.
    /// - Mainly used for copying buffer content to the clipboard.
    /// - Note that the styling of the text will be lost, and newline characters only exist at the end of lines (even if some lines are wrapped), to preserve the original structure of the text.
    #[must_use]
    pub fn tostr(&self, begin: (u32, u32), end: (u32, u32)) -> String {
        if self.alt_screen_mode {
            self.alternate.tostr(begin, end)
        } else {
            self.primary.tostr(begin, end)
        }
    }

    /// 尽可能的生成一段转义序列，以便在其它终端会话内重现当前的屏幕状态。
    /// - with_history 参数决定是否包含历史记录内容。
    /// - 该函数仅用于调试目的，生成的转义序列可能非常长，尤其是在包含历史记录时。
    ///
    /// ---
    ///
    /// Generates a sequence of escape codes that can reproduce the current screen state in another terminal session as much as possible.
    /// - The `with_history` parameter determines whether to include the history content.
    /// - This function is intended for debugging purposes only, and the generated escape sequence can be very long, especially when including history.
    ///
    /// ---
    ///
    /// TODO 目前未完整实现
    #[must_use]
    #[cfg(feature = "snapshot")]
    pub fn snapshot(&self, with_history: bool) -> String {
        if self.alt_screen_mode {
            self.alternate.snapshot(with_history)
        } else {
            self.primary.snapshot(with_history)
        }
    }
}

impl TerminalBuffer {
    pub fn view(&mut self, y: u32) {
        if self.alt_screen_mode {
            self.alternate.view(y);
        } else {
            self.primary.view(y);
        }
    }

    /// 获取当前历史记录行数
    ///
    /// Get the current history line count.
    pub fn history_size(&self) -> u32 {
        if self.alt_screen_mode {
            self.alternate.history_size()
        } else {
            self.primary.history_size()
        }
    }

    /// 设置最大历史记录行数
    ///
    /// Set the maximum history line count.
    pub fn set_history_size(&mut self, capacity: u32) {
        if self.alt_screen_mode {
            self.alternate.set_history_size(capacity);
        } else {
            self.primary.set_history_size(capacity);
        }
    }
}

impl TerminalBuffer {
    /// 按 `count` 行滚动缓冲区。
    /// - 如果 `count` 为正数，向下滚动；如果为负数，向上滚动。
    /// - `fill` 参数用于填充滚动的行。
    ///
    /// ---
    ///
    /// Scroll the buffer by `count` lines.
    /// - If `count` is positive, scroll down; if negative, scroll up.
    /// - The `fill` parameter is used to fill the scrolled lines.
    pub fn scroll_full(&mut self, count: i32, fill: Char) {
        debug_assert!(!fill.is_placeholder(), "Fill character must not be a placeholder");
        if self.alt_screen_mode {
            self.alternate.scroll_full(count, fill);
        } else {
            self.primary.scroll_full(count, fill);
        }
    }

    /// 按 `count` 行滚动缓冲区。
    /// - 如果 `count` 为正数，向下滚动；如果为负数，向上滚动。
    /// - `fill` 参数用于填充滚动的行。
    /// - `region` 参数指定要滚动的行范围。
    ///
    /// ---
    ///
    /// Scroll the buffer by `count` lines.
    /// - If `count` is positive, scroll down; if negative, scroll up.
    /// - The `fill` parameter is used to fill the scrolled lines.
    /// - The `region` parameter specifies the range of rows to scroll.
    pub fn scroll_partial(&mut self, count: i32, fill: Char, region: Range<u32>) {
        debug_assert!(!fill.is_placeholder(), "Fill character must not be a placeholder");
        if self.alt_screen_mode {
            self.alternate.scroll_partial(count, fill, region);
        } else {
            self.primary.scroll_partial(count, fill, region);
        }
    }

    /// 按 `count` 行滚动缓冲区，如果设置了滚动区域，则使用当前滚动区域。
    /// - 如果 `count` 为正数，向下滚动；如果为负数，向上滚动。
    /// - `fill` 参数用于填充滚动的行。
    ///
    /// ---
    ///
    /// Scroll the buffer by `count` lines, using the current scroll region if set.
    /// - If `count` is positive, scroll down; if negative, scroll up.
    /// - The `fill` parameter is used to fill the scrolled lines.
    pub fn scroll(&mut self, count: i32, fill: Char) {
        debug_assert!(!fill.is_placeholder(), "Fill character must not be a placeholder");
        if let Some(region) = self.scroll_region.clone() {
            self.scroll_partial(count, fill, region);
        } else {
            self.scroll_full(count, fill);
        }
    }
}
