use alloc::string::String;
use core::cmp::min;

use crate::buffer::Char;

mod parse_state;
mod processor;

pub use processor::Processor;

terminal_wrapper!();

impl TerminalWrapper<'_> {
    fn set_title(&mut self, title: Option<String>) {
        self.title = title.clone();
        call!(self, title, title);
    }

    fn bell(&mut self) {
        call!(self, bell);
    }

    fn backspace(&mut self) {
        self.cursor().col = self.cursor().col.saturating_sub(1);
    }

    fn insert_blank(&mut self, count: usize) {
        let (row, columns) = (self.cursor().row, self.buffer.width());
        let count = min(count as u32, columns.saturating_sub(self.cursor().col));
        let end = columns.saturating_sub(count);

        let template = Char::empty().with_color(self.style.fg, self.style.bg);
        for column in (self.cursor().col..end).rev() {
            let buffer = &mut self.buffer;
            buffer.set(column + count, row, buffer.get(column, row), template);
            buffer.set(column, row, template, template);
        }
    }

    fn erase_chars(&mut self, count: usize) {
        let start = self.cursor().col;
        let end = min(start + count as u32, self.buffer.width());
        let row = self.cursor().row;
        let template = Char::empty().with_color(self.style.fg, self.style.bg);
        for column in start..end {
            self.buffer.set(column, row, template, template);
        }
    }

    fn delete_chars(&mut self, count: usize) {
        let (row, width) = (self.cursor().row, self.buffer.width());
        let count = min(count as u32, width.saturating_sub(self.cursor().col));

        let template = Char::empty().with_color(self.style.fg, self.style.bg);
        for i in self.cursor().col..width - count {
            let buffer = &mut self.buffer;
            buffer.set(i, row, buffer.get(i + count, row), template);
        }

        let template = Char::empty().with_color(self.style.fg, self.style.bg);
        for i in width - count..width {
            self.buffer.set(i, row, template, template);
        }
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

    fn reset_state(&mut self) {
        self.terminal.reset();
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
