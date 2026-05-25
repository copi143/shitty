use aligned_vec::{ABox, avec};

use crate::buffer::cell::LineSlice;
use crate::buffer::{BufMode, Buffer, Cell, CellSlice, Cursor, CursorPosition, Tick};
use crate::color::{ColorScheme, IColor, WrappedDrawable, WrappedScreen};
use crate::font::FontBuffer;
use crate::{Drawable, Palette};

/// This struct represents a single frame in the terminal.
///
/// See [`Screen`] and [`Drawable`] for more details on how frames are used in rendering.
#[derive(Debug)]
struct Frame {
    /// 当前帧的缓冲区地址
    /// - 相同地址和大小的缓冲区被默认不会在 shitty 外写入
    /// - **请务必保证未被外部覆写，否则可能导致屏幕闪烁**
    ///
    /// ---
    ///
    /// The buffer address of the current frame.
    /// - Buffers with the same address and size are by default not written outside shitty.
    /// - **Please ensure that it is not overwritten externally, as this may cause screen flickering.**
    bufaddr: usize,

    /// 当前帧的缓冲区大小 (字节)
    /// - 相同地址和大小的缓冲区被默认不会在 shitty 外写入
    /// - **请务必保证未被外部覆写，否则可能导致屏幕闪烁**
    ///
    /// ---
    ///
    /// The buffer size of the current frame (in bytes).
    /// - Buffers with the same address and size are by default not written outside shitty.
    /// - **Please ensure that it is not overwritten externally, as this may cause screen flickering.**
    bufsize: usize,

    /// 当前帧的渲染计数器
    ///
    /// The rendering counter of the current frame.
    tick: Tick,

    /// 渲染时的光标位置
    ///
    /// The cursor position during rendering.
    cursor: Option<CursorPosition>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            bufaddr: usize::MAX,
            bufsize: 0,
            tick: Tick::default(),
            cursor: None,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn invalidate(&mut self) {
        self.bufaddr = usize::MAX;
        self.bufsize = 0;
        self.cursor = None;
    }

    pub fn is_valid(&self) -> bool {
        self.bufaddr != usize::MAX
    }

    pub fn is_for<T: IColor>(&self, drawable: &Drawable<T>) -> bool {
        self.bufaddr == drawable.buf.as_ptr() as usize && self.bufsize == core::mem::size_of_val(drawable.buf)
    }

    pub fn update<T: IColor>(&mut self, drawable: &Drawable<T>, tick: Tick) {
        self.bufaddr = drawable.buf.as_ptr() as usize;
        self.bufsize = core::mem::size_of_val(drawable.buf);
        self.tick = tick;
    }
}

/// This struct defines the screen buffer.
#[derive(Debug)]
pub struct Screen<T: IColor> {
    /// 屏幕宽度 (字符数)
    ///
    /// Screen width (in characters)
    pub width: u32,

    /// 屏幕高度 (字符数)
    ///
    /// Screen height (in characters)
    pub height: u32,

    /// 缓冲区模式
    pub buf_mode: BufMode,

    /// 上几帧渲染内容
    ///
    /// Previous frame rendering content
    prev: [Option<Frame>; 3],

    /// 颜色方案
    scheme: ColorScheme<T>,

    /// 当前的渲染单元缓存是否还有效
    ///
    /// Whether the current rendering cell cache is still valid
    cells_valid: bool,

    cells: ABox<[Cell<T>]>,

    /// 每行的渲染计数
    /// - 如果行内的渲染单元被更新，则该行的计数器会被更新为当前的 tick
    /// - 这可以用来快速判断渲染时是否可以直接跳过整行
    ///
    /// The rendering count for each line.
    /// - If the rendering cell in the line is updated, the counter of the line will be updated to the current tick.
    /// - This can be used to quickly determine whether the entire line can be skipped during rendering.
    line_ticks: ABox<[Tick]>,

    /// 渲染计数器
    tick: Tick,
}

impl<T: IColor> Screen<T> {
    pub fn set_color_scheme(&mut self, palette: &Palette) {
        self.scheme = ColorScheme::new(palette);
        self.cells_valid = false;
    }
}

impl<T: IColor> Screen<T> {
    pub fn new(width: u32, height: u32, buf_mode: BufMode) -> Self {
        let scheme = ColorScheme::default();
        let prev = match buf_mode {
            BufMode::None => [Some(Frame::new()), None, None],
            BufMode::Single => [Some(Frame::new()), None, None],
            BufMode::Double => [Some(Frame::new()), Some(Frame::new()), None],
            BufMode::Triple => [Some(Frame::new()), Some(Frame::new()), Some(Frame::new())],
        };

        Self {
            width,
            height,
            prev,
            buf_mode,
            cells_valid: false,
            cells: avec![Cell::new(&scheme); (width * height) as usize].into_boxed_slice(),
            line_ticks: avec![Tick::default(); height as usize].into_boxed_slice(),
            tick: Tick::first(),
            scheme,
        }
    }

    /// 重置屏幕缓冲区
    pub fn reinit(&mut self, width: u32, height: u32, font: &mut FontBuffer) {
        self.clear(font);
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.cells = avec![Cell::new(&self.scheme); (width * height) as usize].into_boxed_slice();
            self.line_ticks = avec![Tick::default(); height as usize].into_boxed_slice();
        }
    }

    pub fn clear(&mut self, font: &mut FontBuffer) {
        for cell in self.cells.iter_mut() {
            cell.clear(&self.scheme, font, true);
        }
        self.cells_valid = false;
        for frame in self.prev.iter_mut().flatten() {
            frame.reset();
        }
    }

    /// 使外部缓存全部失效
    /// - 之后传入的 Drawable 即使地址相同也认为内容可能被覆写
    pub fn invalidate_outer_cache(&mut self) {
        for frame in self.prev.iter_mut().flatten() {
            frame.invalidate();
        }
    }

    /// 使内部缓存全部失效
    /// - 无论如何需要刷新内部的缓存，如颜色主题已经更改
    pub fn invalidate_inner_cache(&mut self) {
        for frame in self.prev.iter_mut().flatten() {
            frame.invalidate();
        }
    }
}

impl<T: IColor> Screen<T> {
    /// 获取一个任意的缓存，优先返回一个失效的缓存
    fn any_cache(&mut self) -> Frame {
        for frame in self.prev.iter_mut() {
            if frame.is_some() && !frame.as_ref().unwrap().is_valid() {
                return frame.take().unwrap();
            }
        }
        for frame in self.prev.iter_mut() {
            if frame.is_some() {
                let mut frame = frame.take().unwrap();
                frame.invalidate();
                return frame;
            }
        }
        panic!("No cache available, that's unexpected");
    }

    /// 根据传入的 Drawable 获取一个可用的缓存
    /// - 如果 buf_mode 是 None 则直接返回一个任意的缓存
    /// - 否则优先返回一个地址和大小都匹配的缓存，如果没有则返回一个任意的缓存
    fn get_cache(&mut self, drawable: &Drawable<T>) -> Frame {
        if self.buf_mode != BufMode::None {
            for frame in self.prev.iter_mut() {
                if frame.is_some() && frame.as_ref().unwrap().is_for(drawable) {
                    return frame.take().unwrap();
                }
            }
        }
        self.any_cache()
    }

    /// 将一个缓存放回缓存池
    fn put_cache(&mut self, drawable: &Drawable<T>, mut frame: Frame, tick: Tick) {
        frame.update(drawable, tick);
        for f in self.prev.iter_mut() {
            if f.is_none() {
                *f = Some(frame);
                return;
            }
        }
        panic!("No empty slot to put cache");
    }
}

impl<T: IColor> Screen<T> {
    fn flush_cursor(
        &mut self,
        drawable: &mut Drawable<T>,
        font: &mut FontBuffer,
        frame: &mut Frame,
        cursor: &mut Cursor,
    ) {
        if let Some(pos) = frame.cursor {
            if pos == cursor.pos && cursor.visible {
                return;
            }
            let cell = &self.cells[(pos.row * self.width + pos.col) as usize];
            drawable.write(font, pos.col, pos.row, cell);
        }
        if !cursor.visible {
            frame.cursor = None;
            return;
        }
        frame.cursor = Some(cursor.pos);
        let mut cell = self.cells[(cursor.pos.row * self.width + cursor.pos.col) as usize].clone();
        core::mem::swap(&mut cell.fg, &mut cell.bg);
        if cell.width > 0 {
            drawable.write(font, cursor.pos.col, cursor.pos.row, &cell);
        }
        cell.drop(font);
    }
}

// 刷新渲染单元缓存
impl<T: IColor> Screen<T> {
    #[expect(unsafe_code)]
    fn at_line<'cells>(&mut self, y: u32) -> CellSlice<'cells, T> {
        debug_assert!(y < self.height, "Y coordinate out of bounds");
        let start = (y * self.width) as usize;
        let end = start + self.width as usize;
        CellSlice {
            cells: unsafe { core::mem::transmute(&mut self.cells[start..end]) },
        }
    }

    #[inline(always)]
    fn flush_line_all(&mut self, font: &mut FontBuffer, mut line: LineSlice<'_>, mut cells: CellSlice<'_, T>) {
        for x in 0..self.width {
            let ch = line.get_mut(x);
            let cell = cells.get_mut(x);
            cell.update(&self.scheme, font, *ch, self.tick);
            ch.dirty = false;
        }
    }

    #[inline(always)]
    fn flush_line(&mut self, font: &mut FontBuffer, mut line: LineSlice<'_>, mut cells: CellSlice<'_, T>) {
        for x in 0..self.width {
            let ch = line.get_mut(x);
            if ch.dirty {
                let cell = cells.get_mut(x);
                cell.update(&self.scheme, font, *ch, self.tick);
                ch.dirty = false;
            }
        }
    }

    #[inline(always)]
    fn flush_cells_all(&mut self, buffer: &mut impl Buffer, font: &mut FontBuffer) {
        for y in 0..self.height {
            let line = buffer.at_line(y);
            let cells = self.at_line(y);
            self.flush_line_all(font, line, cells);
        }
        self.line_ticks.fill(self.tick);
        self.cells_valid = true;
    }

    #[inline(always)]
    fn flush_cells(&mut self, buffer: &mut impl Buffer, font: &mut FontBuffer) {
        if !self.cells_valid {
            return self.flush_cells_all(buffer, font);
        }
        for y in 0..self.height {
            let line = buffer.at_line(y);
            if !line.dirty {
                continue;
            }
            let cells = self.at_line(y);
            if line.all_dirty {
                self.flush_line_all(font, line, cells);
            } else {
                self.flush_line(font, line, cells);
            }
            self.line_ticks[y as usize] = self.tick;
        }
        self.cells_valid = true;
    }
}

impl<T: IColor> Screen<T> {
    #[inline(always)]
    pub fn flush(
        &mut self,
        drawable: &mut Drawable<T>,
        buffer: &mut impl Buffer,
        font: &mut FontBuffer,
        cursor: &mut Cursor,
    ) {
        assert!(self.width == buffer.width(), "Buffer width does not match screen width");
        assert!(self.height == buffer.height(), "Buffer height does not match screen height");
        self.flush_cells(buffer, font);
        let mut frame = self.get_cache(drawable);
        if !frame.is_valid() {
            let bg = self.scheme.background();
            for y in 0..self.height {
                let cells = self.at_line(y);
                for x in 0..self.width {
                    let cell = cells.get(x);
                    if cell.width > 0 {
                        drawable.write(font, x, y, cell);
                    }
                }
                for y in (y * font.height) as usize..((y + 1) * font.height) as usize {
                    for x in (self.width * font.width) as usize..drawable.width {
                        drawable.set(x, y, bg);
                    }
                }
            }
            for y in (self.height * font.height) as usize..drawable.height {
                for x in 0..drawable.width {
                    drawable.set(x, y, bg);
                }
            }
        } else {
            for y in 0..self.height {
                if self.line_ticks[y as usize] <= frame.tick {
                    continue; // Skip the whole line if it hasn't been updated since the last flush
                }
                let cells = self.at_line(y);
                for x in 0..self.width {
                    let cell = cells.get(x);
                    if cell.width > 0 && cell.tick > frame.tick {
                        drawable.write(font, x, y, cell);
                    }
                }
            }
        }
        self.flush_cursor(drawable, font, &mut frame, cursor);
        self.put_cache(drawable, frame, self.tick);
        self.tick.increment();
    }
}

#[derive(Debug)]
pub struct TerminalScreen {
    /// Terminal screen width (in characters)
    pub width: u32,
    /// Terminal screen height (in characters)
    pub height: u32,
    /// Buffer mode
    pub buf_mode: BufMode,
    pub wrapped: WrappedScreen,
}

impl TerminalScreen {
    pub fn set_color_scheme(&mut self, palette: &Palette) {
        screen_dispatch!(self, |scr: &mut Screen<_>| scr.set_color_scheme(palette));
    }
}

impl TerminalScreen {
    pub fn new(width: u32, height: u32, buf_mode: BufMode) -> Self {
        Self {
            wrapped: WrappedScreen::None,
            width,
            height,
            buf_mode,
        }
    }

    pub fn init_if_needed(&mut self, dr: &WrappedDrawable, font: &mut FontBuffer) {
        if self.wrapped.is_same_variant(dr) {
            return;
        }
        screen_dispatch!(self, |scr: &mut Screen<_>| scr.clear(font));
        self.wrapped = drawable_dispatch_new_screen!(dr, self.width, self.height, self.buf_mode);
    }

    pub fn reinit(&mut self, width: u32, height: u32, font: &mut FontBuffer) {
        self.width = width;
        self.height = height;
        screen_dispatch!(self, |scr: &mut Screen<_>| scr.reinit(width, height, font));
    }

    pub fn clear(&mut self, font: &mut FontBuffer) {
        screen_dispatch!(self, |scr: &mut Screen<_>| scr.clear(font));
    }

    pub fn invalidate_outer_cache(&mut self) {
        screen_dispatch!(self, |scr: &mut Screen<_>| scr.invalidate_outer_cache());
    }

    pub fn invalidate_inner_cache(&mut self) {
        screen_dispatch!(self, |scr: &mut Screen<_>| scr.invalidate_inner_cache());
    }

    #[inline(always)]
    pub fn flush(
        &mut self,
        drawable: WrappedDrawable,
        buffer: &mut impl Buffer,
        font: &mut FontBuffer,
        cursor: &mut Cursor,
    ) {
        self.init_if_needed(&drawable, font);
        screen_drawable_dispatch!(
            self,
            drawable,
            |scr: &mut Screen<_>, dr: &mut Drawable<_>| scr.flush(dr, buffer, font, cursor),
            panic!("Drawable type does not match Screen type")
        );
    }
}
