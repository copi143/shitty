use core::cmp::min;
use core::str;

use crate::buffer::{AutoWrap, Char, CharFlags};
use crate::color::{AnsiColor, AnsiNamedColor, AnsiRgb};
use crate::terminal::TerminalMode;

terminal_wrapper!();

const MAX_CSI_PARAMS: usize = 32;

impl TerminalWrapper<'_> {
    fn bell(&mut self) {
        call!(self, bell);
    }

    fn backspace(&mut self) {
        self.cursor_backward(1);
    }

    fn reset_state(&mut self) {
        self.reset();
    }

    fn clear_line(&mut self, mode: u8) {
        let row = self.cursor().row;
        let template = Char::empty().with_color(self.style.fg, self.style.bg);
        match mode {
            1 => {
                for column in 0..=self.cursor().col {
                    self.buffer.set(column, row, template, template);
                }
            }
            2 => self.buffer.clear_line(row, template),
            _ => {
                for column in self.cursor().col..self.buffer.width() {
                    self.buffer.set(column, row, template, template);
                }
            }
        }
    }

    fn clear_screen(&mut self, mode: u8) {
        let template = Char::empty().with_color(self.style.fg, self.style.bg);
        let cursor_row = self.cursor().row;
        let cursor_col = self.cursor().col;

        match mode {
            1 => {
                for row in 0..cursor_row {
                    self.buffer.clear_line(row, template);
                }
                for column in 0..=cursor_col {
                    self.buffer.set(column, cursor_row, template, template);
                }
            }
            2 => {
                self.buffer.clear(template);
                self.buffer.set_cursor(0, 0);
            }
            _ => {
                for column in cursor_col..self.buffer.width() {
                    self.buffer.set(column, cursor_row, template, template);
                }
                for row in cursor_row + 1..self.buffer.height() {
                    self.buffer.clear_line(row, template);
                }
            }
        }
    }

    fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        if top == 0 || top > bottom {
            return;
        }

        let height = self.buffer.height();
        let start = min(top as u32, height).saturating_sub(1);
        let end = min(bottom as u32, height);
        if start >= end {
            return;
        }

        self.buffer.set_scroll_region(start..end);
        self.goto_line(0);
        self.goto_col(0);
    }

    fn device_status(&self, arg: usize) {
        match arg {
            5 => self.user_input(b"\x1b[0n"),
            6 => self.user_input(format!("\x1b[{};{}R", self.cursor().row + 1, self.cursor().col + 1).as_bytes()),
            _ => {}
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum State {
    #[default]
    Ground,
    Escape,
    Csi,
    Osc,
    OscEscape,
}

#[derive(Debug, Default)]
pub struct Processor {
    state: State,
    csi_private: bool,
    csi_params: [u16; MAX_CSI_PARAMS],
    csi_count: usize,
    csi_current: u16,
    csi_has_current: bool,
    utf8_buf: [u8; 4],
    utf8_len: usize,
}

impl Processor {
    pub fn advance(&mut self, terminal: &mut TerminalWrapper<'_>, bytes: &[u8]) {
        let mut i = 0;

        while i < bytes.len() {
            if self.utf8_len > 0 {
                let byte = bytes[i];
                if byte & 0b1100_0000 == 0b1000_0000 {
                    self.push_utf8_byte(terminal, byte);
                    i += 1;
                } else {
                    self.flush_utf8(terminal);
                }
                continue;
            }

            match self.state {
                State::Ground => {
                    let start = i;
                    while i < bytes.len() {
                        match bytes[i] {
                            0x1b | 0x07 | 0x08 | 0x09 | 0x0a | 0x0b | 0x0c | 0x0d | 0x00..=0x1f | 0x7f => break,
                            _ => i += 1,
                        }
                    }

                    if i > start {
                        self.emit_text(terminal, &bytes[start..i]);
                    }

                    if i < bytes.len() {
                        self.ground_control(terminal, bytes[i]);
                        i += 1;
                    }
                }
                State::Escape => {
                    self.escape(terminal, bytes[i]);
                    i += 1;
                }
                State::Csi => {
                    self.csi(terminal, bytes[i]);
                    i += 1;
                }
                State::Osc => {
                    match bytes[i] {
                        0x07 => {
                            self.state = State::Ground;
                        }
                        0x1b => {
                            self.state = State::OscEscape;
                        }
                        _ => {}
                    }
                    i += 1;
                }
                State::OscEscape => {
                    self.state = if bytes[i] == b'\\' { State::Ground } else { State::Osc };
                    i += 1;
                }
            }
        }
    }

    fn ground_control(&mut self, terminal: &mut TerminalWrapper<'_>, byte: u8) {
        match byte {
            0x1b => {
                self.flush_utf8(terminal);
                self.state = State::Escape;
            }
            0x07 => terminal.bell(),
            0x08 => terminal.backspace(),
            0x09 => terminal.tab(),
            0x0a | 0x0b | 0x0c => terminal.lf(),
            0x0d => terminal.cr(),
            0x00..=0x1f | 0x7f => self.flush_utf8(terminal),
            _ => self.push_utf8_byte(terminal, byte),
        }
    }

    fn escape(&mut self, terminal: &mut TerminalWrapper<'_>, byte: u8) {
        self.state = State::Ground;
        match byte {
            b'[' => self.start_csi(false),
            b']' => self.start_osc(),
            b'7' => terminal.save_cursor(),
            b'8' => terminal.restore_cursor(),
            b'c' => terminal.reset_state(),
            b'D' => terminal.lf(),
            b'E' => terminal.newline(),
            b'M' => terminal.scroll(1),
            _ => {}
        }
    }

    fn start_csi(&mut self, private: bool) {
        self.state = State::Csi;
        self.csi_private = private;
        self.csi_params = [0; MAX_CSI_PARAMS];
        self.csi_count = 0;
        self.csi_current = 0;
        self.csi_has_current = false;
    }

    fn start_osc(&mut self) {
        self.state = State::Osc;
    }

    fn csi(&mut self, terminal: &mut TerminalWrapper<'_>, byte: u8) {
        match byte {
            b'0'..=b'9' => {
                self.csi_has_current = true;
                let digit = (byte - b'0') as u16;
                self.csi_current = self.csi_current.saturating_mul(10).saturating_add(digit);
            }
            b';' => {
                self.push_csi_param(self.csi_current);
                self.csi_current = 0;
                self.csi_has_current = false;
            }
            b'?' if self.csi_count == 0 && !self.csi_has_current => {
                self.csi_private = true;
            }
            b' '..=b'/' => {}
            0x40..=0x7e => {
                if self.csi_has_current || self.csi_count == 0 {
                    self.push_csi_param(self.csi_current);
                }
                if self.csi_private {
                    self.dispatch_private_csi(terminal, byte as char);
                } else {
                    self.dispatch_csi(terminal, byte as char);
                }
                self.state = State::Ground;
            }
            _ => {
                self.state = State::Ground;
            }
        }
    }

    fn dispatch_csi(&mut self, terminal: &mut TerminalWrapper<'_>, final_char: char) {
        let cursor_row = terminal.cursor().row as i32;
        match final_char {
            'A' => terminal.goto_line(cursor_row - self.param(0, 1) as i32),
            'B' => terminal.goto_line(cursor_row + self.param(0, 1) as i32),
            'C' => terminal.cursor_forward(self.param(0, 1) as u32),
            'D' => terminal.cursor_backward(self.param(0, 1) as u32),
            'E' => {
                terminal.goto_line(cursor_row + self.param(0, 1) as i32);
                terminal.cr();
            }
            'F' => {
                terminal.goto_line(cursor_row - self.param(0, 1) as i32);
                terminal.cr();
            }
            'G' => terminal.goto_col(self.param(0, 1).saturating_sub(1) as u32),
            'H' | 'f' => {
                let row = self.param(0, 1).saturating_sub(1) as i32;
                let col = self.param(1, 1).saturating_sub(1) as u32;
                terminal.goto_line(row);
                terminal.goto_col(col);
            }
            'J' => terminal.clear_screen(self.param(0, 0) as u8),
            'K' => terminal.clear_line(self.param(0, 0) as u8),
            'm' => self.dispatch_sgr(terminal),
            's' => terminal.save_cursor_position(),
            'u' => terminal.restore_cursor_position(),
            'r' => {
                let top = self.param(0, 1);
                let bottom = self.param(1, terminal.buffer.height() as usize);
                terminal.set_scroll_region(top, bottom);
            }
            'n' => terminal.device_status(self.param(0, 0)),
            _ => {}
        }
    }

    fn dispatch_private_csi(&mut self, terminal: &mut TerminalWrapper<'_>, final_char: char) {
        match final_char {
            'h' => {
                for mode in self.csi_params() {
                    match mode {
                        1 => {
                            terminal.mode.insert(TerminalMode::APP_CURSOR);
                            terminal.keyboard.set_app_cursor(true);
                        }
                        7 => terminal.set_auto_wrap(AutoWrap::Disabled),
                        25 => terminal.buffer.show_cursor(),
                        1049 => terminal.enter_alternate(),
                        2004 => terminal.mode.insert(TerminalMode::BRACKETED_PASTE),
                        _ => {}
                    }
                }
            }
            'l' => {
                for mode in self.csi_params() {
                    match mode {
                        1 => {
                            terminal.mode.remove(TerminalMode::APP_CURSOR);
                            terminal.keyboard.set_app_cursor(false);
                        }
                        7 => terminal.set_auto_wrap(AutoWrap::Delayed),
                        25 => terminal.buffer.hide_cursor(),
                        1049 => terminal.exit_alternate(),
                        2004 => terminal.mode.remove(TerminalMode::BRACKETED_PASTE),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn dispatch_sgr(&mut self, terminal: &mut TerminalWrapper<'_>) {
        let mut i = 0;
        let params = self.csi_params();
        while i < params.len() {
            let p = params[i];
            match p {
                0 => terminal.style = Char::empty(),
                1 => terminal.style.font.set_bold(true),
                3 => terminal.style.font.set_italic(true),
                4 => terminal.style.flags.insert(CharFlags::UNDERLINE),
                7 => terminal.mode.insert(TerminalMode::REVERSE_COLOR),
                22 => terminal.style.font.set_bold(false),
                23 => terminal.style.font.set_italic(false),
                24 => terminal.style.flags.remove(CharFlags::UNDERLINE),
                27 => terminal.mode.remove(TerminalMode::REVERSE_COLOR),
                39 => terminal.style.fg = AnsiColor::Named(AnsiNamedColor::Foreground),
                49 => terminal.style.bg = AnsiColor::Named(AnsiNamedColor::Background),
                30..=37 | 90..=97 => {
                    if let Some(color) = AnsiNamedColor::from_named_fg(p as u8) {
                        terminal.style.fg = AnsiColor::Named(color);
                    }
                }
                40..=47 | 100..=107 => {
                    if let Some(color) = AnsiNamedColor::from_named_bg(p as u8) {
                        terminal.style.bg = AnsiColor::Named(color);
                    }
                }
                38 | 48 => {
                    let is_fg = p == 38;
                    if i + 1 < params.len() && params[i + 1] == 5 {
                        if let Some(&index) = params.get(i + 2) {
                            let color = AnsiColor::Indexed(min(index, 255) as u8);
                            if is_fg {
                                terminal.style.fg = color;
                            } else {
                                terminal.style.bg = color;
                            }
                            i += 2;
                        }
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        let rgb = AnsiRgb {
                            r: min(params[i + 2], 255) as u8,
                            g: min(params[i + 3], 255) as u8,
                            b: min(params[i + 4], 255) as u8,
                        };
                        let color = AnsiColor::Spec(rgb);
                        if is_fg {
                            terminal.style.fg = color;
                        } else {
                            terminal.style.bg = color;
                        }
                        i += 4;
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    fn emit_text(&mut self, terminal: &mut TerminalWrapper<'_>, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            match str::from_utf8(bytes) {
                Ok(text) => {
                    for ch in text.chars() {
                        terminal.put(ch);
                    }
                    return;
                }
                Err(err) => {
                    let valid_up_to = err.valid_up_to();
                    if valid_up_to > 0 {
                        let valid = str::from_utf8(&bytes[..valid_up_to]).unwrap_or("");
                        for ch in valid.chars() {
                            terminal.put(ch);
                        }
                        bytes = &bytes[valid_up_to..];
                    }

                    match err.error_len() {
                        Some(error_len) => {
                            terminal.put('�');
                            bytes = &bytes[error_len..];
                        }
                        None => {
                            self.utf8_len = bytes.len();
                            self.utf8_buf[..self.utf8_len].copy_from_slice(bytes);
                            return;
                        }
                    }
                }
            }
        }
    }

    fn push_utf8_byte(&mut self, terminal: &mut TerminalWrapper<'_>, byte: u8) {
        if self.utf8_len == 0 && byte < 0x80 {
            terminal.put(byte as char);
            return;
        }

        if self.utf8_len >= self.utf8_buf.len() {
            self.flush_utf8(terminal);
        }

        self.utf8_buf[self.utf8_len] = byte;
        self.utf8_len += 1;

        match str::from_utf8(&self.utf8_buf[..self.utf8_len]) {
            Ok(text) => {
                if let Some(ch) = text.chars().next() {
                    terminal.put(ch);
                    self.utf8_len = 0;
                }
            }
            Err(err) => {
                if err.error_len().is_some() {
                    terminal.put('�');
                    self.utf8_len = 0;
                }
            }
        }
    }

    fn flush_utf8(&mut self, terminal: &mut TerminalWrapper<'_>) {
        if self.utf8_len > 0 {
            terminal.put('�');
            self.utf8_len = 0;
        }
    }

    fn push_csi_param(&mut self, value: u16) {
        if self.csi_count < self.csi_params.len() {
            self.csi_params[self.csi_count] = value;
            self.csi_count += 1;
        }
    }

    fn csi_params(&self) -> &[u16] {
        &self.csi_params[..self.csi_count]
    }

    fn param(&self, idx: usize, default: usize) -> usize {
        self.csi_params()
            .get(idx)
            .copied()
            .map(|value| if value == 0 { default } else { value as usize })
            .unwrap_or(default)
    }
}
