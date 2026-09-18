use alloc::vec::Vec;

use super::PointerButton;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// 统一输入事件。将外部输入转换为终端可处理的事件。
///
/// Unified input event. Converts external input into events processable by the terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// 单个字符输入（如普通按键）。
    ///
    /// A single character input (e.g., a regular key press).
    Char(char),
    /// 多字节输入（无法映射为单个字符时使用）。
    ///
    /// Multi-byte input (used when the input cannot be mapped to a single character).
    Bytes(Vec<u8>),
    /// 预定义的静态字符串（如在函数键映射中使用）。
    ///
    /// A predefined static string (e.g., used in function key mappings).
    String(&'static str),
    /// 拷贝请求（Ctrl+C 或自定义快捷键）。
    ///
    /// Copy request (Ctrl+C or custom shortcut).
    Copy,
    /// 粘贴请求（Ctrl+V 或自定义快捷键）。
    ///
    /// Paste request (Ctrl+V or custom shortcut).
    Paste,

    /// 切换颜色主题（通过 F1-F8 快捷键切换内置主题）。
    ///
    /// Switch color scheme (switch built-in themes via F1-F8 shortcuts).
    SetColorScheme(usize),

    /// 原始 winit 键盘事件（仅当启用 `winit` feature）。
    ///
    /// Raw winit keyboard event (only available with `winit` feature).
    #[cfg(feature = "winit")]
    WinitKeyEvent(WinitKeyEvent),
    /// 原始 winit 修饰键状态（仅当启用 `winit` feature）。
    ///
    /// Raw winit modifier state (only available with `winit` feature).
    #[cfg(feature = "winit")]
    WinitModifiers(WinitModifiers),

    /// 滚动，并未指定来源，可以不是鼠标。
    ///
    /// Scrolling; the source is not specified, so it does not necessarily have to be the mouse.
    Scroll {
        direction: ScrollDirection,
        count: u32,
        /// 如果为 true 整页滚动，否则逐行滚动。
        ///
        /// Page scroll if true, otherwise line scroll.
        page: bool,
    },

    /// 指针移动到新坐标。
    ///
    /// Pointer moved to new coordinates.
    PointerMove(i32, i32),
    /// 指针按钮按下。
    ///
    /// Pointer button pressed.
    PointerPress(PointerButton),
    /// 指针按钮释放。
    ///
    /// Pointer button released.
    PointerRelease(PointerButton),
    /// 指针进入终端区域。
    ///
    /// Pointer entered the terminal area.
    PointerEnter,
    /// 指针离开终端区域。
    ///
    /// Pointer left the terminal area.
    PointerLeave,
    /// 窗口焦点变化（DECSET 1004）。
    ///
    /// Window focus change (DECSET 1004).
    Focus(bool),
}

impl Event {
    pub fn scroll(lines: i32) -> Self {
        Event::Scroll {
            direction: if lines < 0 { ScrollDirection::Down } else { ScrollDirection::Up },
            count: lines.unsigned_abs(),
            page: false,
        }
    }

    pub fn scroll_xy(dx: i32, dy: i32) -> Option<Self> {
        if dx == 0 && dy == 0 {
            return None;
        }
        if dy.abs() >= dx.abs() {
            Some(Self::scroll(dy))
        } else {
            Some(Event::Scroll {
                direction: if dx < 0 { ScrollDirection::Left } else { ScrollDirection::Right },
                count: dx.unsigned_abs(),
                page: false,
            })
        }
    }
}

#[cfg(feature = "winit")]
use winit::event::{
    ElementState as WinitElementState, KeyEvent as WinitKeyEvent, Modifiers as WinitModifiers,
    MouseButton as WinitMouseButton,
};

#[cfg(feature = "winit")]
impl TryFrom<(WinitElementState, WinitMouseButton)> for Event {
    type Error = ();

    fn try_from(pair: (WinitElementState, WinitMouseButton)) -> Result<Self, Self::Error> {
        let (state, button) = pair;
        let button = PointerButton::try_from(button)?;
        match state {
            WinitElementState::Pressed => Ok(Event::PointerPress(button)),
            WinitElementState::Released => Ok(Event::PointerRelease(button)),
        }
    }
}

#[cfg(feature = "winit")]
impl From<WinitKeyEvent> for Event {
    fn from(event: WinitKeyEvent) -> Self {
        Event::WinitKeyEvent(event)
    }
}

#[cfg(feature = "winit")]
impl From<WinitModifiers> for Event {
    fn from(modifiers: WinitModifiers) -> Self {
        Event::WinitModifiers(modifiers)
    }
}
