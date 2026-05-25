use aligned_vec::{AVec, avec};
use alloc::collections::vec_deque::VecDeque;
use alloc::string::String;
use core::cmp::min;
use core::ops::Range;

use crate::buffer::{Buffer, Char, CursorPosition, LineSlice};
use crate::helper::{likely, unlikely};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Line {
    chars: AVec<Char>, // 当前行的字符
    endl: bool,        // 是否换行
    dirty: bool,       // 是否脏了
    all_dirty: bool,   // 是否整行都脏了
}

impl Line {
    pub fn new(ch: Char, width: usize) -> Self {
        debug_assert!(ch.width > 0, "Character width must be greater than 0");
        Self {
            chars: avec![ch; width],
            endl: false,
            dirty: true,
            all_dirty: true,
        }
    }

    pub fn clear(&mut self, ch: Char) {
        debug_assert!(ch.width > 0, "Character width must be greater than 0");
        self.chars.fill(ch);
        self.endl = false;
        self.dirty = true;
        self.all_dirty = true;
    }
}

pub struct Primary {
    /// 行历史
    lines: VecDeque<Line>,
    /// 当前屏幕
    buffer: VecDeque<Line>,
    /// 最多保留的行数
    max_lines: u32,
    /// 终端宽度
    width: u32,
    /// 终端高度
    height: u32,
    /// 可视区域开始处
    viewport: u32,
    /// 保存的光标位置
    /// - 只在当前缓冲区不活动时写入，在当前缓冲区活动时读取并存储在 [`TerminalBuffer`](super::TerminalBuffer) 中
    /// - 调整大小等操作也算做切换缓冲区的一种情况
    pub saved_cursor_pos: CursorPosition,
}

impl Primary {
    pub fn new(width: u32, height: u32, max_lines: u32) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_lines as usize),
            buffer: VecDeque::new(),
            max_lines,
            width,
            height,
            viewport: 0,
            saved_cursor_pos: CursorPosition::default(),
        }
    }

    /// 将行添加到历史记录
    fn push_line(&mut self, line: Line) {
        if self.lines.len() >= self.max_lines as usize {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    /// 将当前状态转换为历史记录
    fn as_history(&self) -> (VecDeque<VecDeque<Char>>, CursorPosition) {
        let mut strings = VecDeque::new();
        let mut string = VecDeque::new();
        for line in &self.lines {
            string.extend(line.chars.iter().filter(|&ch| ch.ch != '\0'));
            if line.endl {
                strings.push_back(string);
                string = VecDeque::new();
            }
        }
        let mut pos = self.saved_cursor_pos;
        for (i, line) in self.buffer.iter().enumerate() {
            if i == self.saved_cursor_pos.row as usize {
                pos.row = strings.len() as u32;
                for (j, ch) in line.chars.iter().enumerate() {
                    if j == self.saved_cursor_pos.col as usize {
                        pos.col = string.len() as u32;
                    }
                    if ch.ch != '\0' {
                        string.push_back(*ch);
                    }
                }
            } else {
                for ch in line.chars.iter() {
                    if ch.ch != '\0' {
                        string.push_back(*ch);
                    }
                }
            }
            if line.endl {
                strings.push_back(string);
                string = VecDeque::new();
            }
        }
        strings.push_back(string);
        (strings, pos)
    }

    /// 从历史记录恢复终端状态
    fn set_history(&mut self, strings: VecDeque<VecDeque<Char>>, pos: CursorPosition, fill: Char) {
        debug_assert!(!strings.is_empty(), "Strings must not be empty");

        let mut pos_in_lines = (0, 0);
        let mut lines: VecDeque<Line> = VecDeque::new();
        for (i, string) in strings.into_iter().enumerate() {
            let mut k = 0;
            let mut line = Line::new(fill, self.width as usize);
            let row_len = string.len();
            for (j, char) in string.into_iter().enumerate() {
                if k + char.width as usize > self.width as usize {
                    lines.push_back(line);
                    line = Line::new(fill, self.width as usize);
                    k = 0;
                }
                if i == pos.row as usize && j == pos.col as usize {
                    pos_in_lines = (lines.len() as u32, k as u32);
                }
                line.chars[k] = char;
                for j in 1..char.width as usize {
                    line.chars[k + j] = char.as_placeholder();
                }
                k += char.width as usize;
            }
            if pos.col >= row_len as u32 {
                pos_in_lines = (lines.len() as u32, row_len as u32);
            }
            line.endl = true;
            lines.push_back(line);
        }

        lines.iter_mut().last().unwrap().endl = false;

        while lines.len() > (self.max_lines + self.height) as usize {
            pos_in_lines = (pos_in_lines.0 - 1, pos_in_lines.1);
            lines.pop_front();
        }

        while lines.len() < self.height as usize {
            lines.push_back(Line::new(fill, self.width as usize));
        }

        if lines.len() > self.height as usize {
            self.buffer = lines.split_off(lines.len() - self.height as usize);
            self.lines = lines;
            self.saved_cursor_pos = CursorPosition {
                row: pos_in_lines.0.saturating_sub(self.lines.len() as u32),
                col: pos_in_lines.1,
            };
        } else {
            self.buffer = lines;
            self.lines = VecDeque::new();
            self.saved_cursor_pos = CursorPosition {
                row: pos_in_lines.0,
                col: pos_in_lines.1,
            };
        }
    }
}

impl Buffer for Primary {
    fn resize(&mut self, width: u32, height: u32, fill: Char) {
        let (strings, pos) = self.as_history();
        self.width = width;
        self.height = height;
        self.set_history(strings, pos, fill);
        self.viewport = self.lines.len() as u32;
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    #[inline(always)]
    fn view(&mut self, y: u32) {
        self.viewport = min(y, self.history_size());
    }

    /// 渲染用
    #[inline(always)]
    fn at(&self, x: u32, y: u32) -> Char {
        let y = self.viewport + y;
        if y < self.history_size() {
            self.lines[y as usize].chars[x as usize]
        } else {
            self.buffer[(y - self.history_size()) as usize].chars[x as usize]
        }
    }

    /// 渲染用
    #[inline(always)]
    fn at_line(&'_ mut self, y: u32) -> LineSlice<'_> {
        let y = self.viewport + y;
        let history_size = self.history_size();
        let line = if y < history_size {
            &mut self.lines[y as usize]
        } else {
            &mut self.buffer[(y - history_size) as usize]
        };
        LineSlice {
            chars: &mut line.chars,
            dirty: core::mem::replace(&mut line.dirty, false),
            all_dirty: core::mem::replace(&mut line.all_dirty, false),
        }
    }

    #[inline(always)]
    fn get(&self, x: u32, y: u32) -> Char {
        self.buffer[y as usize].chars[x as usize]
    }

    #[inline(always)]
    fn set(&mut self, x: u32, y: u32, cell: Char) -> Char {
        let line = &mut self.buffer[y as usize];
        line.dirty = true;
        let old = line.chars[x as usize];
        line.chars[x as usize] = cell;
        if unlikely(old.width > 1) {
            for i in 1..old.width as usize {
                line.chars[x as usize + i] = cell.as_empty();
            }
        }
        if unlikely(old.is_placeholder()) {
            let mut i = 1;
            let mut b = true;
            while b {
                b = line.chars[x as usize - i].is_placeholder();
                line.chars[x as usize - i] = cell.as_empty();
                i += 1;
            }
            let mut i = 1;
            while (x + i as u32) < self.width && line.chars[x as usize + i].is_placeholder() {
                line.chars[x as usize + i] = cell.as_empty();
                i += 1;
            }
        }
        if unlikely(cell.width > 1) {
            for i in 1..cell.width as usize {
                line.chars[x as usize + i] = cell.as_placeholder();
            }
        }
        old
    }

    #[inline(always)]
    fn endl(&mut self, y: u32) {
        self.buffer[y as usize].endl = true;
    }

    /// See [`TerminalBuffer::clear_line`](crate::buffer::TerminalBuffer::clear_line) for details.
    fn clear_line(&mut self, y: u32, fill: Char) {
        debug_assert!(fill.dirty, "Fill character must be dirty");
        self.buffer[y as usize].clear(fill);
    }

    /// See [`TerminalBuffer::clear`](crate::buffer::TerminalBuffer::clear) for details.
    fn clear(&mut self, fill: Char) {
        debug_assert!(fill.dirty, "Fill character must be dirty");
        self.buffer.iter_mut().for_each(|line| line.clear(fill));
    }

    /// See [`TerminalBuffer::clear_all`](crate::buffer::TerminalBuffer::clear_all) for details.
    fn clear_all(&mut self, fill: Char) {
        debug_assert!(fill.dirty, "Fill character must be dirty");
        self.lines.clear();
        self.buffer.iter_mut().for_each(|line| line.clear(fill));
    }

    fn tostr(&self, begin: (u32, u32), end: (u32, u32)) -> String {
        let mut result = String::new();
        for y in begin.1..=end.1 {
            let row = y as usize;
            if row >= self.buffer.len() {
                continue;
            }
            let line = &self.buffer[row];
            let start = if y == begin.1 { begin.0 as usize } else { 0 };
            let stop = if y == end.1 {
                (end.0 as usize + 1).min(line.chars.len())
            } else {
                line.chars.len()
            };
            for ch in &line.chars[start..stop] {
                if ch.ch != '\0' {
                    result.push(ch.ch);
                }
            }
            if y != end.1 && line.endl {
                result.push('\n');
            }
        }
        result
    }

    #[cfg(feature = "snapshot")]
    fn snapshot(&self, with_history: bool) -> String {
        let mut result = String::new();
        if with_history {
            for line in &self.lines {
                for ch in line.chars.iter() {
                    result.push(ch.ch);
                }
                if line.endl {
                    result.push('\n');
                }
            }
        }
        for line in &self.buffer {
            for ch in line.chars.iter() {
                result.push(ch.ch);
            }
            if line.endl {
                result.push('\n');
            }
        }
        result
    }

    fn scroll_without_clear(&mut self, count: i32) {
        let _ = count;
    }

    fn scroll_full(&mut self, count: i32, fill: Char) {
        let width = self.width;
        let height = self.height;
        if count == 0 {
            return;
        }
        if count < 0 {
            if !self.lines.is_empty() {
                let i = self.lines.len() - 1;
                self.lines[i].endl = true;
            }
            let mut new_line = Line::new(fill, width as usize);
            new_line.endl = true;
            for _ in 0..(-count) as usize {
                self.buffer.pop_back();
                self.buffer.push_front(new_line.clone());
            }
        } else {
            for _ in 0..count as usize {
                self.endl(height - 1);
                let line = self.buffer.pop_front().unwrap();
                self.push_line(line);
                self.buffer.push_back(Line::new(fill, width as usize));
            }
        }
    }

    fn scroll_partial(&mut self, count: i32, fill: Char, region: Range<u32>) {
        let start = region.start as usize;
        let end = region.end as usize;
        if count == 0 {
            return;
        }
        let amount = count.unsigned_abs() as usize;
        if amount >= end - start {
            for y in start..end {
                self.buffer[y].clear(fill);
            }
            return;
        }
        if count < 0 {
            for y in (start..(end - amount)).rev() {
                self.buffer.swap(y, y + amount);
            }
            for y in start..(start + amount) {
                self.buffer[y].clear(fill);
            }
        } else {
            for y in start..(end - amount) {
                self.buffer.swap(y, y + amount);
            }
            for y in (end - amount)..end {
                self.buffer[y].clear(fill);
            }
        }
    }

    #[inline(always)]
    fn history_size(&self) -> u32 {
        self.lines.len() as u32
    }

    #[inline(always)]
    fn set_history_size(&mut self, capacity: u32) {
        self.max_lines = capacity;
        while self.lines.len() > self.max_lines as usize {
            self.lines.pop_front();
        }
    }
}
