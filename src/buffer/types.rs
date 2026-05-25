use crate::CursorShape;

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tick(u32);

impl Tick {
    pub const fn default() -> Self {
        Self(0)
    }

    /// 初始 tick 设置为 1，这样调用 flush 时能立刻刷新
    pub const fn first() -> Self {
        Self(1)
    }

    pub const fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

enum_map! {
    /// ### A enum defining the mode of the screen buffer.
    ///
    /// 该结构体定义了屏幕缓冲区的模式。<br />
    /// 传入 Drawable 需符合设置的 BufMode，否则可能导致渲染错误。<br />
    ///
    /// - *根据目前的代码，应该是不会出现渲染错误的，只是可能会导致性能下降。注意这不是永远不出错的保证。*
    ///
    /// ---
    ///
    /// This struct defines the mode of the screen buffer.<br />
    /// When passing a Drawable, it must match the set BufMode, otherwise rendering errors may occur.<br />
    ///
    /// - *According to the current code, rendering errors should not occur, but it may lead to performance degradation. Note that this is not a guarantee that errors will never occur.*
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum BufMode : usize {
        /// 每次生成完整帧
        ///
        /// Generate complete frame every time
        None = 0,

        /// 仅重绘需要的部分 - 单缓冲模式
        ///
        /// Only redraw needed parts - single buffer mode
        Single = 1,

        /// 仅重绘需要的部分 - 双缓冲模式
        ///
        /// Only redraw needed parts - double buffer mode
        Double = 2,

        /// 仅重绘需要的部分 - 三缓冲模式
        ///
        /// Only redraw needed parts - triple buffer mode
        Triple = 3,
    }
}

enum_map! {
    /// 自动换行模式
    ///
    /// Auto-wrap mode
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum AutoWrap : usize {
        /// 禁止在光标达到行尾时自动换行，光标会在溢出时回到行首但保持在当前行。
        ///
        /// When the cursor reaches the end of the line, it will not automatically wrap to the next line. Instead, it will return to the beginning of the current line, but stay on the same line.
        Disabled = 0,
        /// 在光标达到行尾时立刻自动换行，不等待下一个字符输入。
        ///
        /// When the cursor reaches the end of the line, it will immediately wrap to the next line without waiting for the next character input.
        Immediate = 1,
        /// 在光标达到行尾时设置换行标志，等待下一个字符输入时自动换行。
        ///
        /// When the cursor reaches the end of the line, it will set a wrap flag and wait for the next character input to automatically wrap to the next line.
        #[default]
        Delayed = 2,
    }
}

/// 光标的位置从 0 开始计数，直到 width-1 和 height-1。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CursorPosition {
    pub col: u32,
    pub row: u32,
}

#[derive(Debug)]
pub struct Cursor {
    /// 光标位置 (在切换缓冲区时保存和恢复)
    /// - 宽度 0..width-1
    /// - 高度 0..height-1
    pub pos: CursorPosition,
    /// 光标形状
    pub shape: CursorShape,
    /// 是否显示光标
    pub visible: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            pos: CursorPosition::default(),
            shape: CursorShape::default(),
            visible: true,
        }
    }
}
