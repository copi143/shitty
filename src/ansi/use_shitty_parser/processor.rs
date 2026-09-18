use alloc::string::String;
use alloc::vec::Vec;
use base64ct::{Base64, Encoding};
use core::cmp::min;

use crate::ansi::use_shitty_parser::TerminalWrapper;
use crate::ansi::use_shitty_parser::parse_state::ParseState;
use crate::buffer::{AutoWrap, Char, CharFlags};
use crate::color::{AnsiColor, AnsiNamedColor, AnsiRgb};
use crate::terminal::TerminalMode;

const MAX_PARAMS: usize = 16;
const MAX_OSC_BYTES: usize = 4096;

#[derive(Default)]
pub struct Processor {
    state: ParseState,
    csi_private: bool,
    csi_params: [u16; MAX_PARAMS],
    csi_count: usize,
    csi_current: u16,
    csi_has_current: bool,
    osc: Vec<u8>,
    utf8_buf: [u8; 4],
    utf8_len: usize,
}

impl Processor {
    pub fn advance(&mut self, terminal: &mut TerminalWrapper<'_>, bytes: &[u8]) {
        for &byte in bytes {
            match self.state {
                ParseState::Ground => self.ground(terminal, byte),
                ParseState::Escape => self.escape(terminal, byte),
                ParseState::EscapeCharset => {
                    self.state = ParseState::Ground;
                }
                ParseState::Csi => self.csi(terminal, byte),
                ParseState::Osc => match byte {
                    0x07 => {
                        self.finish_osc(terminal);
                        self.state = ParseState::Ground;
                    }
                    0x1b => {
                        self.state = ParseState::OscEscape;
                    }
                    _ => {
                        if self.osc.len() < MAX_OSC_BYTES {
                            self.osc.push(byte);
                        }
                    }
                },
                ParseState::OscEscape => {
                    if byte == b'\\' {
                        self.finish_osc(terminal);
                        self.state = ParseState::Ground;
                    } else {
                        if self.osc.len() < MAX_OSC_BYTES {
                            self.osc.push(b'\x1b');
                        }
                        self.state = ParseState::Osc;
                        if self.osc.len() < MAX_OSC_BYTES {
                            self.osc.push(byte);
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::manual_range_patterns)]
    fn ground(&mut self, terminal: &mut TerminalWrapper<'_>, byte: u8) {
        match byte {
            0x1b => {
                self.flush_utf8_invalid(terminal);
                self.state = ParseState::Escape;
            }
            0x07 => terminal.bell(),
            0x08 => terminal.backspace(),
            0x09 => {
                self.flush_utf8_invalid(terminal);
                terminal.tab();
            }
            0x0a | 0x0b | 0x0c => {
                self.flush_utf8_invalid(terminal);
                terminal.lf();
            }
            0x0d => {
                self.flush_utf8_invalid(terminal);
                terminal.cr();
            }
            0x00..=0x1f | 0x7f => {
                self.flush_utf8_invalid(terminal);
            }
            _ => self.push_utf8_byte(terminal, byte),
        }
    }

    fn escape(&mut self, terminal: &mut TerminalWrapper<'_>, byte: u8) {
        self.state = ParseState::Ground;
        match byte {
            b'[' => self.start_csi(false),
            b']' => self.start_osc(),
            b'7' => terminal.save_cursor(),
            b'8' => terminal.restore_cursor(),
            b'D' => {
                terminal.lf();
            }
            b'E' => terminal.newline(),
            b'M' => {
                if terminal.cursor().row == terminal.buffer.scroll_region().start {
                    terminal.scroll(1);
                } else {
                    terminal.cursor().row = terminal.cursor().row.saturating_sub(1);
                }
            }
            b'c' => terminal.reset_state(),
            b'=' => terminal.mode.insert(TerminalMode::APP_KEYPAD),
            b'>' => terminal.mode.remove(TerminalMode::APP_KEYPAD),
            b'(' | b')' | b'*' | b'+' => {
                self.state = ParseState::EscapeCharset;
            }
            _ => {}
        }
    }

    fn start_csi(&mut self, private: bool) {
        self.state = ParseState::Csi;
        self.csi_private = private;
        self.csi_params = [0; MAX_PARAMS];
        self.csi_count = 0;
        self.csi_current = 0;
        self.csi_has_current = false;
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
                self.dispatch_csi(terminal, byte as char);
                self.state = ParseState::Ground;
            }
            _ => {
                self.state = ParseState::Ground;
            }
        }
    }

    fn dispatch_csi(&mut self, terminal: &mut TerminalWrapper<'_>, final_char: char) {
        if self.csi_private {
            self.dispatch_private_csi(terminal, final_char);
            return;
        }

        match final_char {
            'A' => {
                let rows = self.param(0, 1) as i32;
                let row = terminal.cursor().row as i32;
                terminal.goto_line(row - rows);
            }
            'B' => {
                let rows = self.param(0, 1) as i32;
                let row = terminal.cursor().row as i32;
                terminal.goto_line(row + rows);
            }
            'C' => {
                let cols = self.param(0, 1) as u32;
                let col = terminal.cursor().col;
                terminal.goto_col(col.saturating_add(cols));
            }
            'D' => {
                let cols = self.param(0, 1) as u32;
                let col = terminal.cursor().col;
                terminal.goto_col(col.saturating_sub(cols));
            }
            'E' => {
                let rows = self.param(0, 1) as i32;
                let row = terminal.cursor().row as i32;
                terminal.goto_line(row + rows);
                terminal.cr();
            }
            'F' => {
                let rows = self.param(0, 1) as i32;
                let row = terminal.cursor().row as i32;
                terminal.goto_line(row - rows);
                terminal.cr();
            }
            'G' => terminal.goto_col(self.param(0, 1).saturating_sub(1) as u32),
            'd' => terminal.goto_line(self.param(0, 1).saturating_sub(1) as i32),
            'H' | 'f' => {
                let row = self.param(0, 1).saturating_sub(1) as i32;
                let col = self.param(1, 1).saturating_sub(1) as u32;
                terminal.goto_line(row);
                terminal.goto_col(col);
            }
            'J' => terminal.clear_screen(self.param(0, 0) as u8),
            'K' => terminal.clear_line(self.param(0, 0) as u8),
            'L' => terminal.scroll(self.param(0, 1) as i32),
            'M' => terminal.scroll(-(self.param(0, 1) as i32)),
            'P' => terminal.delete_chars(self.param(0, 1)),
            'S' => terminal.scroll(-(self.param(0, 1) as i32)),
            'T' => terminal.scroll(self.param(0, 1) as i32),
            'X' => terminal.erase_chars(self.param(0, 1)),
            '@' => terminal.insert_blank(self.param(0, 1)),
            'm' => self.dispatch_sgr(terminal),
            's' => terminal.save_cursor(),
            'u' => terminal.restore_cursor(),
            'r' => {
                let top = self.param(0, 1);
                let bottom = self.param(1, terminal.buffer.height() as usize);
                terminal.set_scroll_region(top, bottom);
            }
            'n' => terminal.device_status(self.param(0, 0)),
            'h' => {
                for mode in self.csi_params() {
                    match mode {
                        4 => terminal.mode.insert(TerminalMode::INSERT),
                        20 => terminal.mode.insert(TerminalMode::LINE_FEED_NEW_LINE),
                        _ => {}
                    }
                }
            }
            'l' => {
                for mode in self.csi_params() {
                    match mode {
                        4 => terminal.mode.remove(TerminalMode::INSERT),
                        20 => terminal.mode.remove(TerminalMode::LINE_FEED_NEW_LINE),
                        _ => {}
                    }
                }
            }
            'c' if self.param(0, 0) == 0 => {
                terminal.user_input(b"\x1b[?6c");
            }
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
                            terminal.keyboard.app_cursor_mode = true;
                        }
                        7 => terminal.set_auto_wrap(AutoWrap::Delayed),
                        9 | 1000 | 1001 | 1002 | 1003 | 1004 | 1005 | 1006 | 1007 | 1015 | 1016 => {
                            terminal.pointer.set_dec_mode(*mode, true);
                        }
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
                            terminal.keyboard.app_cursor_mode = false;
                        }
                        7 => terminal.set_auto_wrap(AutoWrap::Disabled),
                        9 | 1000 | 1001 | 1002 | 1003 | 1004 | 1005 | 1006 | 1007 | 1015 | 1016 => {
                            terminal.pointer.set_dec_mode(*mode, false);
                        }
                        25 => terminal.buffer.hide_cursor(),
                        1049 => terminal.exit_alternate(),
                        2004 => terminal.mode.remove(TerminalMode::BRACKETED_PASTE),
                        _ => {}
                    }
                }
            }
            'p' => {
                for mode in self.csi_params() {
                    terminal.report_dec_private(*mode);
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
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        let c = min(params[i + 2], 255) as u8;
                        if is_fg {
                            terminal.style.fg = AnsiColor::Indexed(c);
                        } else {
                            terminal.style.bg = AnsiColor::Indexed(c);
                        }
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        let rgb = AnsiRgb {
                            r: min(params[i + 2], 255) as u8,
                            g: min(params[i + 3], 255) as u8,
                            b: min(params[i + 4], 255) as u8,
                        };
                        if is_fg {
                            terminal.style.fg = AnsiColor::Spec(rgb);
                        } else {
                            terminal.style.bg = AnsiColor::Spec(rgb);
                        }
                        i += 4;
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    fn start_osc(&mut self) {
        self.state = ParseState::Osc;
        self.osc.clear();
    }

    fn finish_osc(&mut self, terminal: &mut TerminalWrapper<'_>) {
        let payload = core::mem::take(&mut self.osc);
        let Ok(payload) = String::from_utf8(payload) else {
            return;
        };
        let Some((kind, rest)) = payload.split_once(';') else {
            return;
        };

        match kind {
            "0" | "2" => terminal.set_title(Some(String::from(rest))),
            "52" => {
                let Some((_, base64)) = rest.split_once(';') else {
                    return;
                };
                let text = (str::from_utf8(base64.as_bytes()).ok())
                    .and_then(|text| Base64::decode_vec(text).ok())
                    .and_then(|data| String::from_utf8(data).ok());

                if let Some(text) = text {
                    call!(terminal, clipboard_set, text);
                }
            }
            _ => {}
        }
    }

    fn push_csi_param(&mut self, value: u16) {
        if self.csi_count < MAX_PARAMS {
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
            .map(|v| if v == 0 { default } else { v as usize })
            .unwrap_or(default)
    }

    fn push_utf8_byte(&mut self, terminal: &mut TerminalWrapper<'_>, byte: u8) {
        if self.utf8_len == 0 && byte < 0x80 {
            terminal.put(byte as char);
            return;
        }

        if self.utf8_len >= self.utf8_buf.len() {
            self.utf8_len = 0;
            terminal.put(core::char::REPLACEMENT_CHARACTER);
        }

        self.utf8_buf[self.utf8_len] = byte;
        self.utf8_len += 1;

        match str::from_utf8(&self.utf8_buf[..self.utf8_len]) {
            Ok(s) => {
                if let Some(ch) = s.chars().next() {
                    terminal.put(ch);
                    self.utf8_len = 0;
                }
            }
            Err(err) => {
                if err.error_len().is_some() {
                    terminal.put(core::char::REPLACEMENT_CHARACTER);
                    self.utf8_len = 0;
                }
            }
        }
    }

    fn flush_utf8_invalid(&mut self, terminal: &mut TerminalWrapper<'_>) {
        if self.utf8_len > 0 {
            terminal.put(core::char::REPLACEMENT_CHARACTER);
            self.utf8_len = 0;
        }
    }
}
