use alloc::boxed::Box;
use alloc::string::{String, ToString as _};
use alloc::vec::Vec;
use core::cmp::min;
use core::ops::Range;
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use spin::{Mutex, MutexGuard, Spin};

use crate::CursorShape;
use crate::ansi::Parser;
use crate::buffer::{AutoWrap, BufMode, Char, CursorPosition, TerminalBuffer};
use crate::callback::Callbacks;
use crate::color::WrappedDrawable;
use crate::font::FontRenderer;
use crate::helper::TerminalProcessInput;
use crate::input::{Event, KeyboardManager, Pointer};
use crate::palette::Palette;

#[cfg(feature = "vte")]
use vte::ansi::{CharsetIndex, StandardCharset};

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TerminalMode: u32 {
        const APP_CURSOR = 1 << 1;
        const APP_KEYPAD = 1 << 2;
        const MOUSE_REPORT_CLICK = 1 << 3;
        const BRACKETED_PASTE = 1 << 4;
        const SGR_MOUSE = 1 << 5;
        const LINE_FEED_NEW_LINE = 1 << 8;
        const INSERT = 1 << 10;
        const URGENCY_HINTS = 1 << 18;
        /// 反转前景色和背景色。
        const REVERSE_COLOR = 1 << 19;
        const ANY = u32::MAX;
    }
}

pub struct Terminal {
    pub(crate) saved_cursor: CursorPosition,
    pub(crate) saved_cursor_shape: CursorShape,

    pub(crate) style: Char,
    pub(crate) mode: TerminalMode,
    pub(crate) buffer: TerminalBuffer,
    pub(crate) keyboard: KeyboardManager,

    pub(crate) pointer: Pointer,

    #[cfg(feature = "vte")]
    pub(crate) charsets: [StandardCharset; 4],
    #[cfg(feature = "vte")]
    pub(crate) active_charset: CharsetIndex,

    pub(crate) dirty: AtomicBool,
    pub(crate) callbacks: Mutex<Callbacks>,
    pub(crate) view_at: u32,
    pub(crate) title: Option<String>,
    pub(crate) title_stack: Vec<Option<String>>,
    pub(crate) title_stack_max: usize,
    /// 使用的转义序列解析器。
    pub(crate) parser: Parser,
}

assert_send_sync!(Terminal);

impl Terminal {
    /// 创建一个新的 `Terminal` 实例，并指定缓冲区模式。
    ///
    /// ---
    ///
    /// 终端的初始尺寸由 `width` 和 `height` 参数决定。
    /// - 这些参数可以初始化为 0。
    /// - **如果创建时宽度或高度为 0，应该调用 [`resize`](Terminal::resize) 方法设置初始尺寸。**
    ///
    /// ---
    ///
    /// 有关缓冲区模式的详细信息，请参见 [`BufMode`]。
    /// - 如果不确定使用哪种模式，建议设置为 [`BufMode::Double`]，该模式适用于大多数场景。
    ///
    /// ---
    ///
    /// 字体渲染方面，必须至少提供一个字体渲染器。
    ///
    /// ---
    ///
    /// Creates a new `Terminal` instance with the specified buffer mode.
    ///
    /// ---
    ///
    /// The initial size of the terminal is determined by the `width` and `height` parameters.
    /// - These parameters can be initialized to 0.
    /// - **You should call [`resize`](Terminal::resize) to set the initial size if the terminal is created with a width or height of 0.**
    ///
    /// ---
    ///
    /// See [`BufMode`] for more details on buffer modes.
    /// - If you are unsure which mode to use, set it to [`BufMode::Double`]. This mode is suitable for most scenarios.
    ///
    /// ---
    ///
    /// For font rendering, you need to provide at least one font renderer.
    ///
    /// ---
    ///
    /// Example usage:
    ///
    /// ```rust
    /// let renderer = shitty::EmptyFontRenderer::new(8, 16); // Replace with your actual font renderer.
    /// let mut terminal = shitty::Terminal::new(1280, 720, shitty::BufMode::Double, vec![renderer]);
    /// ```
    pub fn new(width: u32, height: u32, buf_mode: BufMode, fonts: Vec<Box<dyn FontRenderer>>) -> Self {
        if fonts.is_empty() {
            panic!("At least one font renderer must be provided");
        }
        let mut buffer = TerminalBuffer::new(buf_mode);
        for font in fonts {
            buffer.add_renderer(font);
        }
        if width > 0 && height > 0 {
            buffer.resize(width, height);
        }
        Self {
            saved_cursor_shape: CursorShape::default(),
            saved_cursor: CursorPosition::default(),
            style: Char::empty(),
            mode: TerminalMode::default(),
            buffer,
            keyboard: KeyboardManager::default(),
            pointer: Pointer::new(),
            #[cfg(feature = "vte")]
            charsets: Default::default(),
            #[cfg(feature = "vte")]
            active_charset: Default::default(),
            dirty: AtomicBool::new(true),
            callbacks: Mutex::new(Callbacks::default()),
            view_at: 0,
            title: None,
            title_stack: Vec::new(),
            title_stack_max: 64,
            parser: Default::default(),
        }
    }

    /// 将终端缓冲区调整为指定的宽度和高度。
    /// - 宽度和高度必须为正整数。
    /// - 宽度和高度必须不超过 65535 的最大值。
    ///
    /// ---
    ///
    /// Resizes the terminal buffer to the specified width and height.
    /// - Width and height must be positive integers.
    /// - Width and height must not exceed the maximum value of 65535.
    pub fn resize(&mut self, width: u32, height: u32) {
        assert!(width > 0 && height > 0, "Invalid terminal size");
        assert!(width <= u16::MAX as u32, "Width exceeds maximum");
        assert!(height <= u16::MAX as u32, "Height exceeds maximum");
        self.buffer.resize(width, height);
    }

    /// Returns whether the terminal is dirty (i.e., needs to be redrawn).
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    /// Returns the number of rows in the terminal buffer.
    pub fn rows(&self) -> u32 {
        self.buffer.height()
    }

    /// Returns the number of columns in the terminal buffer.
    pub fn cols(&self) -> u32 {
        self.buffer.width()
    }

    /// 将终端缓冲区的内容刷新到可绘制对象上。
    ///
    /// [程序 ->] 终端 -> 用户界面
    ///
    /// ---
    ///
    /// Flush the terminal buffer to the drawable.
    ///
    /// [Program ->] Terminal -> User Interface
    pub fn flush<'buf>(&mut self, drawable: impl Into<WrappedDrawable<'buf>>, time: Duration) -> bool {
        self.dirty.store(false, Ordering::Relaxed);
        self.buffer.flush(drawable.into(), time)
    }

    /// 输入终端内程序的输出用于渲染。
    ///
    /// 程序 -> 终端 [-> 用户界面]
    ///
    /// ---
    ///
    /// Inputs output from the program running inside the terminal for rendering.
    ///
    /// Program -> Terminal [-> User Interface]
    pub fn process<'input>(&mut self, bytes: impl Into<TerminalProcessInput<'input>>) {
        let bytes = bytes.into();
        let bytes = bytes.as_ref();
        if bytes.is_empty() {
            return;
        }
        // 如果视图在历史记录末尾，则在处理后保持视图在末尾
        let view_at_end = self.view_at == self.buffer.history_size();

        self.parser = {
            let mut parser = core::mem::replace(&mut self.parser, Parser::Invalid);
            parser.advance(self, bytes);
            parser
        };

        if view_at_end {
            self.view_history(self.buffer.history_size());
        }
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn put(&mut self, content: char) {
        #[cfg(feature = "vte")]
        let ch = {
            let index = self.active_charset as usize;
            self.style.with_content(self.charsets[index].map(content))
        };
        #[cfg(not(feature = "vte"))]
        let ch = self.style.with_content(content);

        self.buffer.put(ch, self.style);
    }

    /// 将用户输入的数据发送到终端内运行的程序。
    /// - 用户 => 终端 => 程序
    ///
    /// ---
    ///
    /// Sends user input data to the program running inside the terminal.
    /// - User => Terminal => Program
    pub fn user_input(&self, text: &[u8]) {
        call!(self, pty_write, text);
    }

    pub fn paste(&self, text: &str) {
        if self.mode.contains(TerminalMode::BRACKETED_PASTE) {
            self.user_input(format!("\x1b[200~{text}\x1b[201~").as_bytes());
        } else {
            self.user_input(text.as_bytes());
        }
    }

    #[cfg(feature = "keyboard-scancode")]
    fn handle_keyboard_scancode(&mut self, scancode: u8) {
        let event = self.keyboard.handle_pckb_key(scancode);
        if let Some(it) = event {
            self.handle_event(it);
        }
    }

    pub fn handle_event(&mut self, event: impl TryInto<Event>) -> bool {
        macro_rules! encode {
            ($name:ident, $($arg:expr),* $(,)?) => {
                #[expect(unsafe_code)]
                unsafe {
                    let buf = core::mem::transmute(self.pointer.$name($($arg),*));
                    self.user_input(buf);
                }
            };
        }
        let event = match event.try_into() {
            Ok(it) => it,
            Err(_) => return false,
        };
        match event {
            Event::SetColorScheme(index) => {
                self.set_color_scheme(index);
            }
            Event::KbdScroll { up, page } => {
                let lines = if page { self.rows() } else { 1 } as i32;
                self.view_at = self.view_at.saturating_add_signed(if up { -lines } else { lines });
                if self.view_at > self.buffer.history_size() {
                    self.view_at = self.buffer.history_size();
                }
                self.view_history(self.view_at);
            }
            Event::String(s) => {
                self.user_input(s.as_bytes());
            }
            Event::Char(c) => {
                self.user_input(c.to_string().as_bytes());
            }
            Event::Bytes(text) => {
                self.user_input(&text);
            }
            Event::Copy => {}
            Event::Paste => {
                let text = { self.callbacks.lock().clipboard_get.as_ref().and_then(|clipboard_get| clipboard_get()) };
                if let Some(text) = text {
                    self.paste(&text);
                }
            }
            #[cfg(feature = "winit")]
            Event::WinitKeyEvent(event) => {
                let event = self.keyboard.handle_winit_key(event.state, &event.logical_key, event.text.as_deref());
                if let Some(it) = event {
                    self.handle_event(it);
                }
            }
            #[cfg(feature = "winit")]
            Event::WinitModifiers(modifiers) => {
                self.keyboard.modifiers = modifiers;
            }

            Event::PointerMove(x, y) => {
                self.pointer.x = x;
                self.pointer.y = y;
                encode!(encode_move, self.buffer.font_size());
            }
            Event::Scroll(delta) => {
                if delta == 0 {
                    return false;
                }
                encode!(encode_scroll, delta, self.buffer.font_size());
            }
            Event::PointerPress(button) => {
                encode!(encode_press, button, self.buffer.font_size());
            }
            Event::PointerRelease(button) => {
                encode!(encode_release, button, self.buffer.font_size());
            }
            Event::PointerEnter => {
                encode!(encode_enter, self.buffer.font_size());
            }
            Event::PointerLeave => {
                encode!(encode_leave, self.buffer.font_size());
            }
        }
        true
    }

    /// 将从 `begin` 到 `end` 范围内的缓冲区内容转换为字符串。
    /// - 主要用于拷贝缓冲区内容到剪贴板。
    ///
    /// ---
    ///
    /// Convert the buffer content in the range from `begin` to `end` into a string.
    /// - Mainly used for copying buffer content to the clipboard.
    #[must_use]
    pub fn tostr(&self, begin: (u32, u32), end: (u32, u32)) -> String {
        self.buffer.tostr(begin, end)
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
    #[must_use]
    #[cfg(feature = "snapshot")]
    pub fn snapshot(&self, with_history: bool) -> String {
        self.buffer.snapshot(with_history)
    }

    /// 保存当前光标位置和样式，以便稍后恢复。
    pub fn save_cursor(&mut self) {
        self.saved_cursor = self.buffer.cursor();
        self.saved_cursor_shape = self.buffer.cursor_shape();
    }

    /// 恢复之前保存的光标位置和样式。
    pub fn restore_cursor(&mut self) {
        let CursorPosition { col, row } = self.saved_cursor;
        self.buffer.set_cursor(col, row);
        self.buffer.set_cursor_shape(self.saved_cursor_shape);
    }

    /// 保存当前光标的位置，以便稍后恢复。
    pub fn save_cursor_position(&mut self) {
        self.saved_cursor = self.buffer.cursor();
    }

    /// 恢复之前保存的光标位置。
    pub fn restore_cursor_position(&mut self) {
        let CursorPosition { col, row } = self.saved_cursor;
        self.buffer.set_cursor(col, row);
    }

    /// 重置终端状态，包括：
    /// - 退出备用屏幕（如果在备用屏幕中）
    /// - 清空主屏幕和备用屏幕的内容
    /// - 将光标移动到主屏幕的左上角
    /// - 重置光标样式为默认
    /// - 重置终端模式为默认
    pub fn reset(&mut self) {
        if self.buffer.is_alternate() {
            self.exit_alternate();
        }
        self.buffer.clear_all(Char::empty());
        self.buffer.set_cursor(0, 0);
        self.buffer.set_cursor_shape(CursorShape::default());
        self.saved_cursor = self.buffer.cursor();
        self.saved_cursor_shape = self.buffer.cursor_shape();
        self.mode = TerminalMode::default();
        self.style = Char::empty();
    }

    /// 获取 [`TerminalBuffer`] 的 cursor 位置信息。
    pub(crate) fn cursor(&self) -> CursorPosition {
        self.buffer.cursor()
    }

    pub fn auto_wrap(&self) -> AutoWrap {
        self.buffer.auto_wrap()
    }

    pub fn set_auto_wrap(&mut self, mode: AutoWrap) {
        self.buffer.set_auto_wrap(mode);
    }
}

impl Terminal {
    pub fn callbacks(&'_ self) -> MutexGuard<'_, Callbacks, Spin> {
        self.callbacks.lock()
    }

    pub fn set_history_size(&mut self, size: u32) {
        self.buffer.set_history_size(size);
    }

    pub fn set_scroll_speed(&mut self, speed: i32) {
        self.pointer.scroll_speed = speed;
    }

    pub fn set_crnl_mapping(&mut self, mapping: bool) {
        self.keyboard.crnl_mapping = mapping;
    }

    pub fn add_renderer(&mut self, font_renderer: Box<dyn FontRenderer>) {
        self.buffer.add_renderer(font_renderer);
    }

    pub fn set_color_scheme(&mut self, palette_index: usize) {
        self.buffer.set_color_scheme(Palette::get(palette_index));
    }

    pub fn set_custom_color_scheme(&mut self, palette: &Palette) {
        self.buffer.set_color_scheme(palette);
    }
}

impl Terminal {
    pub(crate) fn view_history(&mut self, at: u32) {
        self.view_at = at;
        self.buffer.view(at);
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub(crate) fn enter_alternate(&mut self) {
        if self.buffer.is_alternate() {
            info!("Ignoring enter alternate screen request");
            return;
        }
        info!("Entering alternate screen");
        self.buffer.enter_alternate();
    }

    pub(crate) fn exit_alternate(&mut self) {
        if !self.buffer.is_alternate() {
            info!("Ignoring exit alternate screen request");
            return;
        }
        info!("Exiting alternate screen");
        self.buffer.exit_alternate();
        self.style = Char::empty();
    }

    /// Move cursor to the beginning of the line.
    pub fn cr(&mut self) {
        self.buffer.cr();
    }

    /// Move cursor down by one line, scrolling if necessary.
    pub fn lf(&mut self) {
        if self.keyboard.crnl_mapping {
            self.cr();
        }
        self.buffer.lf(self.style);
    }

    pub fn set_scroll_region(&mut self, mut range: Range<u32>) {
        if range.end > self.buffer.height() {
            range.end = self.buffer.height();
        }
        if range.start >= range.end {
            return;
        }
        self.buffer.set_scroll_region(range);
    }

    /// Move cursor to the next tab stop.
    pub fn tab(&mut self) {
        let tab_stop = self.cursor().col.div_ceil(8) * 8;
        let end_column = tab_stop.min(self.buffer.width());
        let template = Char::empty().with_color(self.style.fg, self.style.bg);
        while self.cursor().col < end_column {
            self.buffer.set(self.cursor().col, self.cursor().row, template, template);
            self.buffer.set_cursor(self.cursor().col + 1, self.cursor().row);
        }
    }

    // TODO
    #[allow(dead_code)]
    pub(crate) fn tab2space(&mut self) -> &mut Self {
        info!("Tab to space");
        self
    }

    pub fn newline(&mut self) {
        self.buffer.newline(self.style);
    }

    /// See [`TerminalBuffer::scroll`] for details.
    pub fn scroll(&mut self, down_count: i32) {
        self.buffer.scroll(down_count, self.style);
    }

    pub(crate) fn goto_line(&mut self, row: i32) {
        if row < 0 {
            self.scroll(row);
            self.buffer.set_cursor(self.cursor().col, 0);
        } else if row >= self.buffer.height() as i32 {
            self.scroll(row - (self.buffer.height() as i32 - 1));
            self.buffer.set_cursor(self.cursor().col, self.buffer.height() - 1);
        } else {
            self.buffer.set_cursor(self.cursor().col, row as u32);
        }
    }

    pub(crate) fn goto_col(&mut self, col: u32) {
        self.buffer.set_cursor(col, self.cursor().row);
    }

    pub fn cursor_forward(&mut self, cols: u32) {
        let new_col = min(self.cursor().col + cols, self.buffer.width() - 1);
        self.buffer.set_cursor(new_col, self.cursor().row);
    }

    pub fn cursor_backward(&mut self, cols: u32) {
        let new_col = self.cursor().col.saturating_sub(cols);
        self.buffer.set_cursor(new_col, self.cursor().row);
    }

    pub fn cursor_up(&mut self, rows: u32) {
        let new_row = self.cursor().row.saturating_sub(rows);
        self.buffer.set_cursor(self.cursor().col, new_row);
    }

    pub fn cursor_down(&mut self, rows: u32) {
        let new_row = min(self.cursor().row + rows, self.buffer.height() - 1);
        self.buffer.set_cursor(self.cursor().col, new_row);
    }
}
