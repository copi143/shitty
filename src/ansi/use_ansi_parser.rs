use alloc::string::String;
use alloc::vec::Vec;
use ansi_parser::{AnsiSequence, parse_escape};
use core::cmp::min;

use crate::buffer::{AutoWrap, Char, CharFlags};
use crate::color::{AnsiColor, AnsiNamedColor, AnsiRgb};
use crate::terminal::TerminalMode;

terminal_wrapper!();

impl TerminalWrapper<'_> {
    fn clear_line(&mut self, mode: u8) {
        let row = self.cursor().row;
        let template = Char::empty().with_color(self.style.fg, self.style.bg);
        match mode {
            1 => {
                for column in 0..=self.cursor().col {
                    self.buffer.set(column, row, template, template);
                }
            }
            2 => {
                self.buffer.clear_line(row, template);
            }
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
            3 => {
                self.buffer.clear_all(template);
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
}

#[derive(Default)]
pub struct Processor {
    pending: Vec<u8>,
    pending_offset: usize,
}

impl Processor {
    pub fn advance(&mut self, terminal: &mut TerminalWrapper<'_>, bytes: &[u8]) {
        self.pending.extend_from_slice(bytes);

        let mut pos = self.pending_offset;
        while pos < self.pending.len() {
            let esc = self.pending[pos..].iter().position(|&b| b == 0x1b).map(|index| pos + index);

            let Some(esc_pos) = esc else {
                Self::emit_text(terminal, &self.pending[pos..]);
                pos = self.pending.len();
                break;
            };

            if esc_pos > pos {
                Self::emit_text(terminal, &self.pending[pos..esc_pos]);
                pos = esc_pos;
            }

            let ascii_end = esc_pos + self.pending[esc_pos..].iter().take_while(|&&b| b < 0x80).count();
            let ascii_slice = &self.pending[esc_pos..ascii_end];

            if ascii_slice.is_empty() {
                pos += 1;
                continue;
            }

            let seq_str = core::str::from_utf8(ascii_slice).unwrap_or("");
            match parse_escape(seq_str) {
                Ok((rest, sequence)) => {
                    let consumed = seq_str.len().saturating_sub(rest.len());
                    if consumed == 0 {
                        pos += 1;
                        continue;
                    }
                    Self::apply_sequence(terminal, sequence);
                    pos = esc_pos + consumed;
                }
                Err(_) => {
                    if ascii_end == self.pending.len() {
                        break;
                    }
                    terminal.put('\u{1b}');
                    pos = esc_pos + 1;
                }
            }
        }

        if pos == self.pending_offset {
            return;
        }
        if pos >= self.pending.len() {
            self.pending.clear();
            self.pending_offset = 0;
            return;
        }
        self.pending_offset = pos;
        if self.pending_offset > 4096 || self.pending_offset * 2 >= self.pending.len() {
            self.pending.drain(0..self.pending_offset);
            self.pending_offset = 0;
        }
    }

    fn emit_text(terminal: &mut TerminalWrapper<'_>, bytes: &[u8]) {
        for c in String::from_utf8_lossy(bytes).chars() {
            match c {
                '\t' => terminal.tab(),
                '\r' => terminal.cr(),
                '\n' => terminal.lf(),
                _ => terminal.put(c),
            };
        }
    }

    fn apply_sequence(terminal: &mut TerminalWrapper<'_>, sequence: AnsiSequence) {
        match sequence {
            AnsiSequence::CursorPos(row, col) => {
                terminal.goto_line(row.saturating_sub(1) as i32);
                terminal.goto_col(col.saturating_sub(1));
            }
            AnsiSequence::CursorUp(rows) => {
                let row = terminal.cursor().row as i32;
                terminal.goto_line(row - rows as i32);
            }
            AnsiSequence::CursorDown(rows) => {
                let row = terminal.cursor().row as i32;
                terminal.goto_line(row + rows as i32);
            }
            AnsiSequence::CursorForward(cols) => {
                let col = terminal.cursor().col;
                terminal.goto_col(col.saturating_add(cols));
            }
            AnsiSequence::CursorBackward(cols) => {
                let col = terminal.cursor().col;
                terminal.goto_col(col.saturating_sub(cols));
            }
            AnsiSequence::CursorSave => terminal.save_cursor(),
            AnsiSequence::CursorRestore => terminal.restore_cursor(),
            AnsiSequence::EraseDisplay => terminal.clear_screen(2),
            AnsiSequence::EraseLine => terminal.clear_line(0),
            AnsiSequence::SetGraphicsMode(params) => Self::apply_sgr(terminal, &params),
            AnsiSequence::ShowCursor => terminal.buffer.show_cursor(),
            AnsiSequence::HideCursor => terminal.buffer.hide_cursor(),
            AnsiSequence::CursorToApp => {
                terminal.mode.insert(TerminalMode::APP_CURSOR);
                terminal.keyboard.set_app_cursor(true);
            }
            AnsiSequence::SetNewLineMode => terminal.mode.insert(TerminalMode::LINE_FEED_NEW_LINE),
            AnsiSequence::SetLineFeedMode => terminal.mode.remove(TerminalMode::LINE_FEED_NEW_LINE),
            AnsiSequence::SetAutoWrap => terminal.set_auto_wrap(AutoWrap::Delayed),
            AnsiSequence::ResetAutoWrap => terminal.set_auto_wrap(AutoWrap::Disabled),
            AnsiSequence::SetAlternateKeypad => terminal.mode.insert(TerminalMode::APP_KEYPAD),
            AnsiSequence::SetNumericKeypad => terminal.mode.remove(TerminalMode::APP_KEYPAD),
            AnsiSequence::SetTopAndBottom(top, bottom) if top <= bottom => {
                terminal.set_scroll_region(top.saturating_sub(1)..bottom);
                terminal.goto_line(0);
                terminal.goto_col(0);
            }
            _ => {}
        }
    }

    fn apply_sgr(terminal: &mut TerminalWrapper<'_>, params: &[u8]) {
        let mut i = 0;
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
                    if let Some(color) = AnsiNamedColor::from_named_fg(p) {
                        terminal.style.fg = AnsiColor::Named(color);
                    }
                }
                40..=47 | 100..=107 => {
                    if let Some(color) = AnsiNamedColor::from_named_bg(p) {
                        terminal.style.bg = AnsiColor::Named(color);
                    }
                }
                38 | 48 => {
                    let is_fg = p == 38;
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        let c = min(params[i + 2], 255);
                        if is_fg {
                            terminal.style.fg = AnsiColor::Indexed(c);
                        } else {
                            terminal.style.bg = AnsiColor::Indexed(c);
                        }
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        let rgb = AnsiRgb {
                            r: min(params[i + 2], 255),
                            g: min(params[i + 3], 255),
                            b: min(params[i + 4], 255),
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
}
