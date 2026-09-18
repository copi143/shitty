use aligned_vec::{AVec, avec};
use alloc::string::String;
use bitvec::vec::BitVec;
use core::cmp::min;
use core::ops::Range;

use crate::buffer::{Buffer, Char, CursorPosition, LineSlice};
use crate::helper::{likely, unlikely};

/// 备用缓冲区
pub struct Alternate {
    /// 备用缓冲区内容
    /// - 以 padded_width 为行宽，padded_height 为行数的环形缓冲区
    buffer: AVec<Char>,
    /// 行脏标记
    line_dirty: BitVec,
    /// 行全脏标记
    line_all_dirty: BitVec,
    /// 备用缓冲区起始位置
    /// - 起始的 index，如果有全屏滚动直接修改这个值而不是移动数据
    begin: u32,
    /// 备用缓冲区宽度
    width: u32,
    /// 备用缓冲区高度
    height: u32,
    /// 对齐的备用缓冲区宽度 (必须是 2 的幂)
    padded_width: u32,
    /// 对齐的备用缓冲区高度 (必须是 2 的幂)
    padded_height: u32,
    /// padded_width - 1
    mask_width: u32,
    /// padded_height - 1
    mask_height: u32,
    /// log(padded_width)
    shift: u32,
    /// 保存的光标位置
    /// - 只在当前缓冲区不活动时写入，在当前缓冲区活动时读取并存储在 [`TerminalBuffer`](super::TerminalBuffer) 中
    /// - 调整大小等操作也算做切换缓冲区的一种情况
    pub saved_cursor_pos: CursorPosition,
}

const _: usize = core::mem::size_of::<Alternate>();

impl Alternate {
    /// 创建一个未初始化的备用缓冲区。
    /// 使用前必须先调用 [`init`](Self::init)。
    ///
    /// Create an uninitialized alternate buffer.
    /// [`init`](Self::init) must be called before use.
    pub fn new() -> Self {
        Self {
            buffer: avec![],
            line_dirty: BitVec::new(),
            line_all_dirty: BitVec::new(),
            begin: 0,
            width: 0,
            height: 0,
            padded_width: 0,
            padded_height: 0,
            mask_width: 0,
            mask_height: 0,
            shift: 0,
            saved_cursor_pos: CursorPosition::default(),
        }
    }

    /// 反初始化备用缓冲区，释放所有资源。
    ///
    /// Deinitialize the alternate buffer, releasing all resources.
    pub fn deinit(&mut self) {
        self.buffer.clear();
        self.line_dirty.clear();
        self.line_all_dirty.clear();
        self.begin = 0;
        self.width = 0;
        self.height = 0;
        self.padded_width = 0;
        self.padded_height = 0;
        self.mask_width = 0;
        self.mask_height = 0;
        self.shift = 0;
        self.saved_cursor_pos = CursorPosition::default();
    }

    /// 初始化备用缓冲区，分配指定大小的环形缓冲。
    /// - `fill`: 用于填充新行的字符
    ///
    /// Initialize the alternate buffer with the specified size as a ring buffer.
    /// - `fill`: The character used to fill new lines
    pub fn init(&mut self, width: u32, height: u32, fill: Char) {
        debug_assert!(self.buffer.is_empty(), "Alternate buffer already initialized");
        self.begin = 0;
        self.width = width;
        self.height = height;
        self.padded_width = width.next_power_of_two();
        self.padded_height = height.next_power_of_two();
        self.mask_width = self.padded_width - 1;
        self.mask_height = self.padded_height - 1;
        self.shift = self.padded_width.trailing_zeros();
        self.buffer.resize((self.padded_width * self.padded_height) as usize, fill);
        self.line_dirty.resize(self.padded_height as usize, false);
        self.line_all_dirty.resize(self.padded_height as usize, false);
        self.saved_cursor_pos = CursorPosition::default();
    }

    fn copy_line(&mut self, dy: u32, sy: u32) {
        let src_index = self.index(0, sy);
        let dst_index = self.index(0, dy);
        self.buffer.copy_within(src_index..(src_index + self.width as usize), dst_index);
    }

    #[inline(always)]
    fn index(&self, x: u32, y: u32) -> usize {
        debug_assert!(x < self.width, "x ({x}) out of bounds (0..{})", self.width);
        debug_assert!(y < self.height, "y ({y}) out of bounds (0..{})", self.height);
        ((((self.begin + y) & self.mask_height) << self.shift) | x) as usize
    }
}

impl Buffer for Alternate {
    fn resize(&mut self, width: u32, height: u32, fill: Char) {
        assert!(width > 0 && height > 0, "Width and height must be greater than 0");
        if self.buffer.is_empty() {
            return self.init(width, height, fill);
        }
        if self.width == width && self.height == height {
            return;
        }
        let padded_width = width.next_power_of_two();
        let padded_height = height.next_power_of_two();
        let shift = padded_width.trailing_zeros() as usize;
        // 滚动当前缓冲区
        let nscrolled = if height < self.height {
            let mut dh = self.height as i32 - height as i32;
            for y in (height..self.height).rev() {
                let line_beg = self.index(0, y);
                let line_end = line_beg + self.width as usize;
                if self.buffer[line_beg..line_end].iter().any(|ch| ch.ch != '\0') {
                    break;
                }
                dh -= 1;
            }
            if dh > 0 {
                self.scroll_without_clear(dh);
            }
            dh as u32
        } else {
            0
        };
        // 重新分配缓冲区
        let mut buffer = avec![fill; (padded_width * padded_height) as usize];
        let copy_width = min(width, self.width) as usize;
        for y in 0..min(height, self.height) {
            let src_index = self.index(0, y);
            let dst_index = (y << shift) as usize;
            buffer[dst_index..(dst_index + copy_width)]
                .copy_from_slice(&self.buffer[src_index..(src_index + copy_width)]);
        }
        // 更新结构
        self.buffer = buffer;
        self.begin = 0;
        self.width = width;
        self.height = height;
        self.padded_width = padded_width;
        self.padded_height = padded_height;
        self.mask_width = padded_width - 1;
        self.mask_height = padded_height - 1;
        self.shift = padded_width.trailing_zeros();
        self.saved_cursor_pos.col = min(self.saved_cursor_pos.col, width - 1);
        self.saved_cursor_pos.row = self.saved_cursor_pos.row.saturating_sub(nscrolled);
        self.line_dirty.resize(padded_height as usize, true);
        self.line_all_dirty.resize(padded_height as usize, true);
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn view(&mut self, y: u32) {
        // Alternate buffer does not support history.
        assert!(y == 0, "Alternate buffer does not support viewing history");
    }

    #[inline(always)]
    #[expect(unsafe_code)]
    fn at(&self, x: u32, y: u32) -> Char {
        unsafe { *self.buffer.get_unchecked(self.index(x, y)) }
    }

    #[inline(always)]
    #[expect(unsafe_code)]
    fn at_line(&'_ mut self, y: u32) -> LineSlice<'_> {
        let index = self.index(0, y);
        let chars = unsafe { self.buffer.get_unchecked_mut(index..(index + self.width as usize)) };
        let dirty = self.line_dirty[y as usize];
        let all_dirty = self.line_all_dirty[y as usize];
        self.line_dirty.set(y as usize, false);
        self.line_all_dirty.set(y as usize, false);
        LineSlice {
            chars,
            dirty,
            all_dirty,
        }
    }

    #[inline(always)]
    #[expect(unsafe_code)]
    fn get(&self, x: u32, y: u32) -> Char {
        unsafe { *self.buffer.get_unchecked(self.index(x, y)) }
    }

    #[inline(always)]
    #[expect(unsafe_code)]
    fn set(&mut self, x: u32, y: u32, cell: Char) -> Char {
        self.line_dirty.set(y as usize, true);
        let idx = self.index(x, y);
        let old = unsafe { *self.buffer.get_unchecked(idx) };
        unsafe { *self.buffer.get_unchecked_mut(idx) = cell };
        if unlikely(old.width > 1) {
            for i in 1..old.width as usize {
                unsafe { *self.buffer.get_unchecked_mut(idx + i) = cell.as_empty() };
            }
        }
        if unlikely(old.is_placeholder()) {
            let mut i = 1;
            let mut b = true;
            while b {
                b = unsafe { *self.buffer.get_unchecked(idx - i) }.is_placeholder();
                unsafe { *self.buffer.get_unchecked_mut(idx - i) = cell.as_empty() };
                i += 1;
            }
            let mut i = 1;
            while (x + i as u32) < self.width && unsafe { *self.buffer.get_unchecked(idx + i) }.is_placeholder() {
                unsafe { *self.buffer.get_unchecked_mut(idx + i) = cell.as_empty() };
                i += 1;
            }
        }
        if unlikely(cell.width > 1) {
            for i in 1..cell.width as usize {
                unsafe { *self.buffer.get_unchecked_mut(idx + i) = cell.as_placeholder() };
            }
        }
        old
    }

    #[inline(always)]
    fn endl(&mut self, _y: u32) {
        // Do nothing, alternate buffer does not support history.
    }

    /// See [`TerminalBuffer::clear_line`](crate::buffer::TerminalBuffer::clear_line) for details.
    fn clear_line(&mut self, y: u32, cell: Char) {
        debug_assert!(cell.width > 0, "Cell must have a width greater than 0");
        let index = self.index(0, y);
        self.buffer[index..(index + self.width as usize)].fill(cell);
        self.line_all_dirty.set(y as usize, true);
    }

    /// See [`TerminalBuffer::clear`](crate::buffer::TerminalBuffer::clear) for details.
    fn clear(&mut self, cell: Char) {
        debug_assert!(cell.width > 0, "Cell must have a width greater than 0");
        self.buffer.fill(cell);
        self.begin = 0;
        self.line_all_dirty.fill(true);
    }

    /// See [`TerminalBuffer::clear_all`](crate::buffer::TerminalBuffer::clear_all) for details.
    fn clear_all(&mut self, cell: Char) {
        debug_assert!(cell.width > 0, "Cell must have a width greater than 0");
        self.buffer.fill(cell);
        self.begin = 0;
        self.line_all_dirty.fill(true);
    }

    fn tostr(&self, begin: (u32, u32), end: (u32, u32)) -> String {
        let mut result = String::new();
        for y in begin.1..=end.1 {
            for x in begin.0..=end.0 {
                let ch = self.get(x, y);
                if ch.ch != '\0' {
                    result.push(ch.ch);
                }
            }
            if y != end.1 {
                result.push('\n');
            }
        }
        result
    }

    #[cfg(feature = "snapshot")]
    fn snapshot(&self, with_history: bool) -> String {
        let mut result = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let ch = self.get(x, y);
                if ch.ch != '\0' {
                    result.push(ch.ch);
                }
            }
            if with_history || y != self.height - 1 {
                result.push('\n');
            }
        }
        result
    }

    fn scroll_without_clear(&mut self, count: i32) {
        if count == 0 {
            return;
        }
        if count < 0 {
            if count <= -(self.height as i32) {
                return;
            }
            let delta_begin = (self.padded_height as i32 + count) as u32;
            self.begin = (self.begin + delta_begin) & self.mask_height;
        } else {
            if count >= self.height as i32 {
                return;
            }
            let count = count as u32;
            self.begin = (self.begin + count) & self.mask_height;
        }
    }

    fn scroll_full(&mut self, count: i32, fill: Char) {
        debug_assert!(fill.width > 0, "FillCell must have a width greater than 0");
        if count == 0 {
            return;
        }
        if count < 0 {
            if count <= -(self.height as i32) {
                self.clear(fill);
                return;
            }
            let delta_begin = (self.padded_height as i32 + count) as u32;
            self.begin = (self.begin + delta_begin) & self.mask_height;
            for i in 0..(-count) as u32 {
                self.clear_line(i, fill);
            }
        } else {
            if count >= self.height as i32 {
                self.clear(fill);
                return;
            }
            let count = count as u32;
            self.begin = (self.begin + count) & self.mask_height;
            for i in (self.height - count)..self.height {
                self.clear_line(i, fill);
            }
        }
        self.line_all_dirty.fill(true);
    }

    fn scroll_partial(&mut self, count: i32, fill: Char, region: Range<u32>) {
        debug_assert!(fill.width > 0, "Cell must have a width greater than 0");
        debug_assert!(region.start < region.end, "Invalid region: {:?}", region);
        debug_assert!(region.end <= self.height, "Region end out of bounds: {:?}", region);
        if count == 0 {
            return;
        }
        let nscroll = region.end - region.start;
        if count < 0 {
            if count <= -(nscroll as i32) {
                for y in region.start..region.end {
                    self.clear_line(y, fill);
                }
                return;
            }
            let count = (-count) as u32;
            for i in (region.start..(region.end - count)).rev() {
                self.copy_line(i + count, i);
            }
            for i in (region.start..(region.start + count)).rev() {
                self.clear_line(i, fill);
            }
        } else {
            if count >= nscroll as i32 {
                for y in region.start..region.end {
                    self.clear_line(y, fill);
                }
                return;
            }
            let count = count as u32;
            for i in region.start..(region.end - count) {
                self.copy_line(i, i + count);
            }
            for i in (region.end - count)..region.end {
                self.clear_line(i, fill);
            }
        }
        self.line_all_dirty.fill(true);
    }

    fn history_size(&self) -> u32 {
        0
    }

    fn set_history_size(&mut self, capacity: u32) {
        debug_assert!(capacity == 0, "Alternate buffer does not support history, capacity must be 0");
    }
}
