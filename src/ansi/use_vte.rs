use alloc::string::String;
use base64ct::{Base64, Encoding};
use core::cmp::min;
use core::time::Duration;
use vte::ansi::{Attr, NamedMode, Rgb};
use vte::ansi::{CharsetIndex, StandardCharset, TabulationClearMode};
use vte::ansi::{ClearMode, CursorShape, Timeout};
use vte::ansi::{CursorStyle, Hyperlink, KeyboardModes};
use vte::ansi::{Handler, LineClearMode, Mode, NamedPrivateMode, PrivateMode};

use crate::buffer::{AutoWrap, Char, CharFlags, CursorPosition};
use crate::terminal::TerminalMode;

#[derive(Default)]
struct DummySyncHandler;

#[rustfmt::skip]
impl Timeout for DummySyncHandler {
    fn set_timeout(&mut self, _: Duration) {}
    fn clear_timeout(&mut self) {}
    fn pending_timeout(&self) -> bool { false }
}

terminal_wrapper!();

#[derive(Default)]
pub struct Processor(vte::ansi::Processor<DummySyncHandler>);

impl Processor {
    pub fn advance(&mut self, terminal: &mut TerminalWrapper<'_>, bytes: &[u8]) {
        self.0.advance(terminal, bytes);
    }
}

impl Handler for TerminalWrapper<'_> {
    fn set_title(&mut self, title: Option<String>) {
        info!("set_title: {:?}", title);
        call!(self, title, title);
    }

    fn set_cursor_style(&mut self, style: Option<CursorStyle>) {
        debug!("Set cursor style: {:?}", style);
        if let Some(style) = style {
            self.set_cursor_shape(style.shape);
        }
    }

    fn set_cursor_shape(&mut self, shape: CursorShape) {
        debug!("Set cursor shape: {:?}", shape);
        self.buffer.set_cursor_shape(shape.into());
    }

    fn input(&mut self, content: char) {
        self.put(content);
    }

    fn goto(&mut self, row: i32, col: usize) {
        debug!("Goto position: ({}, {})", row, col);
        self.terminal.goto_line(row);
        self.terminal.goto_col(col as u32);
    }

    fn goto_line(&mut self, row: i32) {
        debug!("Goto line: {}", row);
        self.terminal.goto_line(row);
    }

    fn goto_col(&mut self, col: usize) {
        debug!("Goto column: {}", col);
        self.terminal.goto_col(col as u32);
    }

    fn insert_blank(&mut self, count: usize) {
        debug!("Insert blank: {}", count);
        let (row, columns) = (self.cursor().row, self.buffer.width());
        let count = min(count as u32, columns - self.cursor().col);

        let template = Char::empty().with_color(self.style.fg, self.style.bg);
        for column in (self.cursor().col..columns - count).rev() {
            let buffer = &mut self.buffer;
            buffer.set(column + count, row, buffer.get(column, row), template);
            buffer.set(column, row, template, template);
        }
    }

    fn move_up(&mut self, rows: usize) {
        debug!("Move up: {}", rows);
        self.goto(self.cursor().row.saturating_sub(rows as u32) as i32, self.cursor().col as usize);
    }

    fn move_down(&mut self, rows: usize) {
        debug!("Move down: {}", rows);
        let goto_line = min(self.cursor().row + rows as u32, self.buffer.height() - 1) as i32;
        self.goto(goto_line, self.cursor().col as usize);
    }

    fn identify_terminal(&mut self, intermediate: Option<char>) {
        debug!("Identify terminal: {:?}", intermediate);

        let version_number = |version: &str| -> usize {
            let mut result = 0;
            let semver_versions = version.split('.');
            for (i, part) in semver_versions.rev().enumerate() {
                let semver_number = part.parse::<usize>().unwrap_or(0);
                result += usize::pow(100, i as u32) * semver_number;
            }
            result
        };

        match intermediate {
            None => self.terminal.user_input(b"\x1b[?6c"),
            Some('>') => {
                let version = version_number(env!("CARGO_PKG_VERSION"));
                self.terminal.user_input(format!("\x1b[>0;{version};1c").as_bytes());
            }
            _ => debug!("Unsupported device attributes intermediate"),
        }
    }

    fn device_status(&mut self, arg: usize) {
        match arg {
            5 => self.terminal.user_input(b"\x1b[0n"),
            6 => {
                let (row, column) = (self.cursor().row, self.cursor().col);
                self.terminal.user_input(format!("\x1b[{};{}R", row + 1, column + 1).as_bytes());
            }
            _ => debug!("Unknown device status query: {}", arg),
        };
    }

    fn move_forward(&mut self, cols: usize) {
        debug!("Move forward: {}", cols);
        self.cursor_forward(cols as u32);
    }

    fn move_backward(&mut self, cols: usize) {
        debug!("Move backward: {}", cols);
        self.cursor_backward(cols as u32);
    }

    fn move_up_and_cr(&mut self, rows: usize) {
        debug!("Move up and cr: {}", rows);
        self.cursor_up(rows as u32);
        self.carriage_return();
    }

    fn move_down_and_cr(&mut self, rows: usize) {
        debug!("Move down and cr: {}", rows);
        self.cursor_down(rows as u32);
        self.carriage_return();
    }

    fn put_tab(&mut self, count: u16) {
        debug!("Put tab: {}", count);
        for _ in 0..count {
            self.tab();
        }
    }

    fn backspace(&mut self) {
        self.cursor_backward(1);
    }

    fn carriage_return(&mut self) {
        self.cr();
    }

    fn linefeed(&mut self) {
        self.lf();
    }

    fn bell(&mut self) {
        debug!("Bell triggered!");
        call!(self, bell);
    }

    fn substitute(&mut self) {
        debug!("Unhandled substitute!");
    }

    fn newline(&mut self) {
        self.linefeed();

        if self.mode.contains(TerminalMode::LINE_FEED_NEW_LINE) {
            self.carriage_return();
        }
    }

    fn set_horizontal_tabstop(&mut self) {
        info!("Unhandled set horizontal tabstop!");
    }

    fn scroll_up(&mut self, count: usize) {
        self.scroll(-(count as i32));
    }

    fn scroll_down(&mut self, count: usize) {
        self.scroll(count as i32);
    }

    fn insert_blank_lines(&mut self, count: usize) {
        debug!("Insert blank lines: {}", count);
        self.scroll(count as i32);
    }

    fn delete_lines(&mut self, count: usize) {
        debug!("Delete lines: {}", count);
        self.scroll(-(count as i32));
    }

    fn erase_chars(&mut self, count: usize) {
        debug!("Erase chars: {}", count);
        let start = self.buffer.cursor().col;
        let end = min(start + count as u32, self.buffer.width());

        let template = Char::empty().with_color(self.style.fg, self.style.bg);
        for column in start..end {
            let row = self.buffer.cursor().row;
            self.buffer.set(column, row, template, template);
        }
    }

    fn delete_chars(&mut self, count: usize) {
        debug!("Delete chars: {}", count);
        let (row, width) = (self.buffer.cursor().row, self.buffer.width());
        let count = min(count as u32, width - self.buffer.cursor().col - 1);

        let template = Char::empty().with_color(self.style.fg, self.style.bg);
        for i in self.buffer.cursor().col..width - count {
            let buffer = &mut self.buffer;
            buffer.set(i, row, buffer.get(i + count, row), template);
        }
        for i in width - count..width {
            self.buffer.set(i, row, template, template);
        }
    }

    fn move_backward_tabs(&mut self, count: u16) {
        debug!("Unhandled move backward tabs: {}", count);
    }

    fn move_forward_tabs(&mut self, count: u16) {
        debug!("Unhandled move forward tabs: {}", count);
    }

    fn save_cursor_position(&mut self) {
        debug!("Save cursor position");
        self.terminal.save_cursor_position();
    }

    fn restore_cursor_position(&mut self) {
        debug!("Restore cursor position");
        self.terminal.restore_cursor_position();
    }

    fn clear_line(&mut self, mode: LineClearMode) {
        debug!("Clear line: {:?}", mode);
        let row = self.buffer.cursor().row;
        let template = Char::empty().with_color(self.style.fg, self.style.bg);
        match mode {
            LineClearMode::Right => {
                for column in self.buffer.cursor().col..self.buffer.width() {
                    self.buffer.set(column, row, template, template);
                }
            }
            LineClearMode::Left => {
                for column in 0..=self.buffer.cursor().col {
                    self.buffer.set(column, row, template, template);
                }
            }
            LineClearMode::All => {
                self.buffer.clear_line(row, template);
            }
        }
    }

    fn clear_screen(&mut self, mode: ClearMode) {
        debug!("Clear screen: {:?}", mode);
        let template = Char::empty().with_color(self.style.fg, self.style.bg);

        match mode {
            ClearMode::All => {
                self.buffer.clear(template);
                self.buffer.set_cursor(0, 0);
            }
            ClearMode::Saved => {
                self.buffer.clear_all(template);
                self.buffer.set_cursor(0, 0);
            }
            ClearMode::Above => {
                for row in 0..self.buffer.cursor().row {
                    self.buffer.clear_line(row, template);
                }
                let row = self.buffer.cursor().row;
                for column in 0..=self.buffer.cursor().col {
                    self.buffer.set(column, row, template, template);
                }
            }
            ClearMode::Below => {
                let row = self.buffer.cursor().row;
                for column in self.buffer.cursor().col..self.buffer.width() {
                    self.buffer.set(column, row, template, template);
                }
                for row in self.buffer.cursor().row + 1..self.buffer.height() {
                    self.buffer.clear_line(row, template);
                }
            }
        }
    }

    fn clear_tabs(&mut self, mode: TabulationClearMode) {
        debug!("Unhandled clear tabs: {:?}", mode);
    }

    fn reset_state(&mut self) {
        debug!("Reset state");
        self.reset();
    }

    fn reverse_index(&mut self) {
        debug!("Reverse index");
        if self.buffer.cursor().row == self.buffer.scroll_region().start {
            self.scroll_down(1);
        } else {
            let CursorPosition { col, row } = self.buffer.cursor();
            self.buffer.set_cursor(col, row - 1);
        }
    }

    fn terminal_attribute(&mut self, attr: Attr) {
        match attr {
            Attr::Foreground(color) => self.style.fg = color.into(),
            Attr::Background(color) => self.style.bg = color.into(),
            Attr::Reset => self.style = Char::empty(),
            Attr::Reverse => self.mode.insert(TerminalMode::REVERSE_COLOR),
            Attr::CancelReverse => self.mode.remove(TerminalMode::REVERSE_COLOR),
            Attr::Bold => self.style.font.set_bold(true),
            Attr::CancelBold => self.style.font.set_bold(false),
            Attr::CancelBoldDim => self.style.font.set_bold(false),
            Attr::Italic => self.style.font.set_italic(true),
            Attr::CancelItalic => self.style.font.set_italic(false),
            Attr::Underline => self.style.flags.insert(CharFlags::UNDERLINE),
            Attr::CancelUnderline => self.style.flags.remove(CharFlags::UNDERLINE),
            Attr::Hidden => {}
            Attr::CancelHidden => {}
            _ => debug!("Unhandled terminal attribute: {:?}", attr),
        }
    }

    fn set_mode(&mut self, mode: Mode) {
        let mode = match mode {
            Mode::Named(mode) => mode,
            Mode::Unknown(mode) => {
                debug!("Ignoring unknown mode {} in set_mode", mode);
                return;
            }
        };

        match mode {
            NamedMode::Insert => self.mode.insert(TerminalMode::INSERT),
            NamedMode::LineFeedNewLine => self.mode.insert(TerminalMode::LINE_FEED_NEW_LINE),
        }
    }

    fn unset_mode(&mut self, mode: Mode) {
        let mode = match mode {
            Mode::Named(mode) => mode,
            Mode::Unknown(mode) => {
                debug!("Ignoring unknown mode {} in unset_mode", mode);
                return;
            }
        };

        match mode {
            NamedMode::Insert => self.mode.remove(TerminalMode::INSERT),
            NamedMode::LineFeedNewLine => self.mode.remove(TerminalMode::LINE_FEED_NEW_LINE),
        }
    }

    fn report_mode(&mut self, mode: Mode) {
        debug!("Unhandled report mode: {:?}", mode);
    }

    fn set_private_mode(&mut self, mode: PrivateMode) {
        let mode = match mode {
            PrivateMode::Named(mode) => mode,
            PrivateMode::Unknown(mode) => {
                debug!("Ignoring unknown mode {} in set_private_mode", mode);
                return;
            }
        };

        macro_rules! pointer {
            ($name:ident) => {{
                self.pointer.$name = true;
                self.pointer.calc_mode();
            }};
        }

        match mode {
            NamedPrivateMode::SwapScreenAndSetRestoreCursor => self.enter_alternate(),
            NamedPrivateMode::ShowCursor => self.buffer.show_cursor(),
            NamedPrivateMode::CursorKeys => {
                self.mode.insert(TerminalMode::APP_CURSOR);
                self.keyboard.set_app_cursor(true);
            }
            NamedPrivateMode::LineWrap => self.set_auto_wrap(AutoWrap::Delayed),
            NamedPrivateMode::BracketedPaste => self.mode.insert(TerminalMode::BRACKETED_PASTE),
            NamedPrivateMode::ReportMouseClicks => pointer!(enabled_1000),
            NamedPrivateMode::ReportCellMouseMotion => pointer!(enabled_1002),
            NamedPrivateMode::ReportAllMouseMotion => pointer!(enabled_1003),
            NamedPrivateMode::ReportFocusInOut => {
                // TODO
            }
            NamedPrivateMode::Utf8Mouse => pointer!(enabled_1005),
            NamedPrivateMode::SgrMouse => pointer!(enabled_1006),
            _ => debug!("Unhandled set mode: {:?}", mode),
        }
    }

    fn unset_private_mode(&mut self, mode: PrivateMode) {
        let mode = match mode {
            PrivateMode::Named(mode) => mode,
            PrivateMode::Unknown(mode) => {
                debug!("Ignoring unknown mode {} in unset_private_mode", mode);
                return;
            }
        };

        macro_rules! pointer {
            ($name:ident) => {{
                self.pointer.$name = false;
                self.pointer.calc_mode();
            }};
        }

        match mode {
            NamedPrivateMode::SwapScreenAndSetRestoreCursor => self.exit_alternate(),
            NamedPrivateMode::ShowCursor => self.buffer.hide_cursor(),
            NamedPrivateMode::CursorKeys => {
                self.mode.remove(TerminalMode::APP_CURSOR);
                self.keyboard.set_app_cursor(false);
            }
            NamedPrivateMode::LineWrap => self.set_auto_wrap(AutoWrap::Disabled),
            NamedPrivateMode::BracketedPaste => self.mode.remove(TerminalMode::BRACKETED_PASTE),
            NamedPrivateMode::ReportMouseClicks => pointer!(enabled_1000),
            NamedPrivateMode::ReportCellMouseMotion => pointer!(enabled_1002),
            NamedPrivateMode::ReportAllMouseMotion => pointer!(enabled_1003),
            NamedPrivateMode::ReportFocusInOut => {
                // TODO
            }
            NamedPrivateMode::Utf8Mouse => pointer!(enabled_1005),
            NamedPrivateMode::SgrMouse => pointer!(enabled_1006),
            _ => debug!("Unhandled unset mode: {:?}", mode),
        }
    }

    fn report_private_mode(&mut self, mode: PrivateMode) {
        debug!("Unhandled report private mode: {:?}", mode);
    }

    fn set_scrolling_region(&mut self, top: usize, bottom: Option<usize>) {
        debug!("Set scrolling region: top={}, bottom={:?}", top, bottom);
        let bottom = bottom.unwrap_or(self.buffer.height() as usize);

        if top >= bottom {
            debug!("Invalid scrolling region: ({};{})", top, bottom);
            return;
        }

        self.set_scroll_region(top as u32..bottom as u32);
        self.goto(0, 0);
    }

    fn set_keypad_application_mode(&mut self) {
        debug!("Set keypad application mode");
        self.mode.insert(TerminalMode::APP_KEYPAD);
    }

    fn unset_keypad_application_mode(&mut self) {
        debug!("Unset keypad application mode");
        self.mode.remove(TerminalMode::APP_KEYPAD);
    }

    fn set_active_charset(&mut self, index: CharsetIndex) {
        debug!("Set active charset: {:?}", index);
        self.active_charset = index;
    }

    fn configure_charset(&mut self, index: CharsetIndex, charset: StandardCharset) {
        debug!("Configure charset: {:?}, {:?}", index, charset);
        self.charsets[index as usize] = charset;
    }

    fn set_color(&mut self, index: usize, color: Rgb) {
        debug!("Unhandled set color: {}, {:?}", index, color);
    }

    fn dynamic_color_sequence(&mut self, prefix: String, index: usize, terminator: &str) {
        debug!("Unhandled dynamic color sequence: {}, {}, {}", prefix, index, terminator);
    }

    fn reset_color(&mut self, index: usize) {
        debug!("Unhandled reset color: {}", index);
    }

    fn clipboard_store(&mut self, clipboard: u8, base64: &[u8]) {
        debug!("Clipboard store: {}, {:?}", clipboard, base64);

        let text = (str::from_utf8(base64).ok())
            .and_then(|text| Base64::decode_vec(text).ok())
            .and_then(|data| String::from_utf8(data).ok());

        if let Some(text) = text {
            call!(self, clipboard_set, text);
        }
    }

    fn clipboard_load(&mut self, clipboard: u8, terminator: &str) {
        debug!("Clipboard load: {}, {}", clipboard, terminator);

        let mut text = None;
        if let Some(handler) = self.callbacks.lock().clipboard_get.as_mut() {
            let Some(t) = handler() else {
                return;
            };
            text = Some(t);
        };

        let base64 = Base64::encode_string(text.unwrap().as_bytes());
        let result = format!("\x1b]52;{};{base64}{terminator}", clipboard as char);
        self.terminal.user_input(result.as_bytes());
    }

    // TODO test
    fn decaln(&mut self) {
        info!("DECALN - Fill screen with E");
        let term = &mut *self.terminal;
        term.buffer.clear(term.style.with_content('E'));
    }

    fn push_title(&mut self) {
        if self.title_stack.len() >= self.title_stack_max {
            error!("Title stack overflow, cannot push title");
        } else {
            let title = self.title.clone();
            self.title_stack.push(title);
        }
    }

    fn pop_title(&mut self) {
        if let Some(title) = self.title_stack.pop() {
            self.set_title(title);
        } else {
            error!("Title stack is empty, cannot pop title");
        }
    }

    fn text_area_size_pixels(&mut self) {
        warn!("Unhandled text area size pixels!");
    }

    fn text_area_size_chars(&mut self) {
        warn!("Unhandled text area size chars!");
    }

    fn set_hyperlink(&mut self, hyperlink: Option<Hyperlink>) {
        warn!("Unhandled set hyperlink: {:?}", hyperlink);
    }

    fn report_keyboard_mode(&mut self) {
        warn!("Report keyboard mode!");
        let current_mode = KeyboardModes::NO_MODE.bits();
        self.terminal.user_input(format!("\x1b[?{current_mode}u").as_bytes());
    }

    fn push_keyboard_mode(&mut self, mode: KeyboardModes) {
        warn!("Unhandled push keyboard mode: {:?}", mode);
    }

    fn pop_keyboard_modes(&mut self, to_pop: u16) {
        warn!("Unhandled pop keyboard modes: {}", to_pop);
    }
}
