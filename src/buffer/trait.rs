use alloc::string::String;
use core::ops::Range;

use crate::buffer::{Char, LineSlice};

pub trait Buffer {
    /// 调整终端大小并保持内容
    /// - `fill`: 用于填充空余内容的字符
    fn resize(&mut self, width: u32, height: u32, fill: Char);

    /// Get the width of the buffer.
    fn width(&self) -> u32;

    /// Get the height of the buffer.
    fn height(&self) -> u32;

    /// 将当前视图的首行设置为指定行。
    ///
    /// Set the top line of the current view to the specified line.
    fn view(&mut self, y: u32);

    /// 获取当前 `(x, y)` 位置的字符。
    /// - `at` 以当前视图首行为基准。
    /// - `get` 以历史记录开始处为基准。
    ///
    /// Get the character at position `(x, y)` in the buffer.
    /// - `at` is based on the current view's top line.
    /// - `get` is based on the start of the history.
    fn at(&self, x: u32, y: u32) -> Char;

    fn at_line(&'_ mut self, y: u32) -> LineSlice<'_>;

    /// 获取当前 `(x, y)` 位置的字符。
    /// - `at` 以当前视图首行为基准。
    /// - `get` 以历史记录开始处为基准。
    ///
    /// Get the character at position `(x, y)` in the buffer.
    /// - `at` is based on the current view's top line.
    /// - `get` is based on the start of the history.
    fn get(&self, x: u32, y: u32) -> Char;

    /// Set the character at position `(x, y)` in the buffer.
    fn set(&mut self, x: u32, y: u32, cell: Char) -> Char;

    /// Mark whether a line is a new line (i.e., the previous line ends with a newline).
    fn endl(&mut self, y: u32);

    fn clear_line(&mut self, y: u32, cell: Char);

    /// Clear the buffer with a specific character.
    /// This method is used to clear the visible area of the buffer, not the entire buffer.
    fn clear(&mut self, cell: Char);

    /// Clear the buffer with a specific character.
    /// This method is used to clear the entire buffer, not just the visible area.
    fn clear_all(&mut self, cell: Char);

    /// Convert the buffer to a string representation.
    fn tostr(&self, begin: (u32, u32), end: (u32, u32)) -> String;

    /// Only for debug purposes.
    #[cfg(feature = "snapshot")]
    fn snapshot(&self, with_history: bool) -> String;

    /// 滚动但是不清理新增的行
    /// 新增的行中可能有脏数据
    /// 在 resize 的时候调用下，反正脏数据都会被切除
    ///
    /// TODO ***这不应该是公开API***
    fn scroll_without_clear(&mut self, count: i32);

    /// Scroll the buffer by `count` lines.
    ///
    /// If `count` is positive, scroll down; if negative, scroll up.
    /// The `cell` parameter is used to fill the scrolled lines.
    ///
    /// If scroll up, the top lines will be filled with `cell`.
    /// If scroll down, the bottom lines will be filled with `cell`.
    fn scroll_full(&mut self, count: i32, fill: Char);

    /// Scroll the buffer by `count` lines.
    ///
    /// If `count` is positive, scroll down; if negative, scroll up.
    /// The `cell` parameter is used to fill the scrolled lines.
    /// The `region` parameter specifies the range of rows to scroll.
    ///
    /// If scroll up, the top lines will be filled with `cell`.
    /// If scroll down, the bottom lines will be filled with `cell`.
    fn scroll_partial(&mut self, count: i32, fill: Char, region: Range<u32>);

    /// Get the current history size.
    /// This is the number of lines that can be scrolled back in the buffer.
    fn history_size(&self) -> u32;

    fn set_history_size(&mut self, capacity: u32);
}
