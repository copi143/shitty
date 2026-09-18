use core::fmt::Write as _;

use super::ScrollDirection;
use crate::helper::BufWriter;

enum_map! {
    /// 鼠标按钮枚举。
    ///
    /// Mouse button enumeration.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum PointerButton : usize {
        /// 左键。
        ///
        /// Left button.
        Left = 0,
        /// 右键。
        ///
        /// Right button.
        Right = 1,
        /// 中键。
        ///
        /// Middle button.
        Middle = 2,
        /// 后退键。
        ///
        /// Back (X1) button.
        X1 = 3,
        /// 前进键。
        ///
        /// Forward (X2) button.
        X2 = 4,
    }
}

impl PointerButton {
    /// 转换为 xterm 鼠标编码的按钮代码。
    ///
    /// Convert to the xterm mouse encoding button code.
    pub const fn to_xterm_code(self) -> u8 {
        match self {
            PointerButton::Left => 0,
            PointerButton::Middle => 1,
            PointerButton::Right => 2,
            PointerButton::X1 => 128,
            PointerButton::X2 => 129,
        }
    }

    pub const ALL: [Self; 5] = [Self::Left, Self::Right, Self::Middle, Self::X1, Self::X2];
}

#[cfg(feature = "winit")]
use winit::event::MouseButton as WinitMouseButton;

#[cfg(feature = "winit")]
impl TryFrom<WinitMouseButton> for PointerButton {
    type Error = ();

    fn try_from(button: WinitMouseButton) -> Result<Self, Self::Error> {
        match button {
            WinitMouseButton::Left => Ok(PointerButton::Left),
            WinitMouseButton::Right => Ok(PointerButton::Right),
            WinitMouseButton::Middle => Ok(PointerButton::Middle),
            WinitMouseButton::Back => Ok(PointerButton::X1),
            WinitMouseButton::Forward => Ok(PointerButton::X2),
            _ => Err(()),
        }
    }
}

/// 鼠标按钮状态（位掩码）。
///
/// Mouse button state (bitmask).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PointerButtonState {
    flags: u32,
}

impl PointerButtonState {
    /// 创建一个新的空状态。
    ///
    /// Create a new empty state.
    pub const fn new() -> Self {
        PointerButtonState { flags: 0 }
    }

    /// 清除所有按钮状态。
    ///
    /// Clear all button states.
    pub const fn clear(&mut self) {
        self.flags = 0;
    }

    /// 标记指定按钮为按下状态。
    ///
    /// Mark the specified button as pressed.
    pub const fn press(&mut self, button: PointerButton) {
        self.flags |= 1 << button as usize;
    }

    /// 标记指定按钮为释放状态。
    ///
    /// Mark the specified button as released.
    pub const fn release(&mut self, button: PointerButton) {
        self.flags &= !(1 << button as usize);
    }

    /// 检查指定按钮是否被按下。
    ///
    /// Check if the specified button is pressed.
    pub const fn is_pressed(&self, button: PointerButton) -> bool {
        (self.flags & (1 << button as usize)) != 0
    }

    /// 检查指定按钮是否被释放。
    ///
    /// Check if the specified button is released.
    pub const fn is_released(&self, button: PointerButton) -> bool {
        !self.is_pressed(button)
    }

    /// 获取当前按下的按钮数量。
    ///
    /// Get the number of currently pressed buttons.
    pub const fn num_pressed(&self) -> u32 {
        self.flags.count_ones()
    }

    const fn first_pressed(self) -> Option<PointerButton> {
        if self.flags == 0 {
            return None;
        }
        match self.flags.trailing_zeros() {
            0 => Some(PointerButton::Left),
            1 => Some(PointerButton::Right),
            2 => Some(PointerButton::Middle),
            3 => Some(PointerButton::X1),
            4 => Some(PointerButton::X2),
            _ => None,
        }
    }
}

/// 鼠标报告模式过滤器（对应 DECSET 模式码）。
///
/// Mouse report mode filter (corresponds to DECSET mode codes).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PointerReportFilter {
    /// 不报告鼠标事件（默认）。
    ///
    /// No mouse event reporting (default).
    #[default]
    None,
    /// X10 报告模式（仅按钮按下，模式 9）。
    ///
    /// X10 report mode (button press only, mode 9).
    X10,
    /// 报告按钮点击（模式 1000）。
    ///
    /// Report button clicks (mode 1000).
    Button,
    /// 报告按钮点击和拖动（模式 1002）。
    ///
    /// Report button clicks and drags (mode 1002).
    ButtonMotion,
    /// 报告所有鼠标移动（模式 1003）。
    ///
    /// Report all mouse motion (mode 1003).
    AnyMotion,
}

impl PointerReportFilter {
    pub const fn report_press(&self) -> bool {
        !matches!(self, PointerReportFilter::None)
    }

    pub const fn report_release(&self) -> bool {
        !matches!(self, PointerReportFilter::None | PointerReportFilter::X10)
    }

    pub const fn report_motion(&self, button_down: bool) -> bool {
        matches!(self, PointerReportFilter::AnyMotion)
            || (matches!(self, PointerReportFilter::ButtonMotion) && button_down)
    }

    pub const fn reporting(&self) -> bool {
        !matches!(self, PointerReportFilter::None)
    }
}

/// 鼠标坐标编码方式。
///
/// Mouse coordinate encoding method.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PointerReportEncoding {
    /// 传统编码（`ESC [ M`，坐标限于 0-223）。
    ///
    /// Legacy encoding (`ESC [ M`, coordinates limited to 0-223).
    #[default]
    Legacy,
    /// UTF-8 编码（模式 1005，支持更大坐标范围）。
    ///
    /// UTF-8 encoding (mode 1005, supports larger coordinate range).
    UTF8,
    /// SGR 编码（模式 1006，支持超大坐标）。
    ///
    /// SGR encoding (mode 1006, supports very large coordinates).
    SGR,
    /// URXVT 编码（模式 1015）。
    ///
    /// URXVT encoding (mode 1015).
    URXVT,
}

/// 鼠标指针状态管理器。处理鼠标事件编码为终端转义序列。
///
/// Mouse pointer state manager. Handles encoding of mouse events into terminal escape sequences.
pub struct Pointer {
    /// 指针在窗口中的 X 坐标（像素）。
    ///
    /// Pointer X coordinate in the window (pixels).
    pub x: i32,
    /// 指针在窗口中的 Y 坐标（像素）。
    ///
    /// Pointer Y coordinate in the window (pixels).
    pub y: i32,
    /// 指针是否已进入终端区域。
    ///
    /// Whether the pointer has entered the terminal area.
    pub captured: bool,
    /// 滚轮滚动速度倍率。
    ///
    /// Scroll wheel speed multiplier.
    pub scroll_speed: i32,
    /// 修饰键位：Shift=4, Alt=8, Ctrl=16。
    ///
    /// Modifier bits: Shift=4, Alt=8, Ctrl=16.
    pub modifiers: u8,

    button_state: PointerButtonState,
    last_pressed: Option<PointerButton>,
    last_cell: Option<(i32, i32)>,
    report_filter: PointerReportFilter,
    report_encoding: PointerReportEncoding,
    encode_buffer: [u8; 64],

    /// 启用 X10 鼠标报告（DECSET 9）。
    ///
    /// Enable X10 mouse reporting (DECSET 9).
    pub enabled_9: bool,
    /// 启用鼠标点击报告（DECSET 1000）。
    ///
    /// Enable mouse click reporting (DECSET 1000).
    pub enabled_1000: bool,
    /// 启用 highlight tracking（DECSET 1001，按 1000 处理）。
    ///
    /// Enable highlight tracking (DECSET 1001, treated as 1000).
    pub enabled_1001: bool,
    /// 启用鼠标点击和拖动报告（DECSET 1002）。
    ///
    /// Enable mouse click and drag reporting (DECSET 1002).
    pub enabled_1002: bool,
    /// 启用所有鼠标移动报告（DECSET 1003）。
    ///
    /// Enable all mouse motion reporting (DECSET 1003).
    pub enabled_1003: bool,
    /// 启用焦点进出报告（DECSET 1004）。
    ///
    /// Enable focus in/out reporting (DECSET 1004).
    pub enabled_1004: bool,
    /// 启用 UTF-8 坐标编码（DECSET 1005）。
    ///
    /// Enable UTF-8 coordinate encoding (DECSET 1005).
    pub enabled_1005: bool,
    /// 启用 SGR 坐标编码（DECSET 1006）。
    ///
    /// Enable SGR coordinate encoding (DECSET 1006).
    pub enabled_1006: bool,
    /// 启用备用屏滚轮映射为光标键（DECSET 1007）。
    ///
    /// Enable alternate-screen scroll as cursor keys (DECSET 1007).
    pub enabled_1007: bool,
    /// 启用 URXVT 坐标编码（DECSET 1015）。
    ///
    /// Enable URXVT coordinate encoding (DECSET 1015).
    pub enabled_1015: bool,
    /// 启用 SGR 像素坐标编码（DECSET 1016）。
    ///
    /// Enable SGR pixel coordinate encoding (DECSET 1016).
    pub enabled_1016: bool,
}

impl Pointer {
    /// 创建一个新的指针状态管理器。
    ///
    /// Create a new pointer state manager.
    pub const fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            captured: false,
            scroll_speed: 3,
            modifiers: 0,
            button_state: PointerButtonState::new(),
            last_pressed: None,
            last_cell: None,
            report_filter: PointerReportFilter::None,
            report_encoding: PointerReportEncoding::Legacy,
            encode_buffer: [0; 64],

            enabled_9: false,
            enabled_1000: false,
            enabled_1001: false,
            enabled_1002: false,
            enabled_1003: false,
            enabled_1004: false,
            enabled_1005: false,
            enabled_1006: false,
            enabled_1007: false,
            enabled_1015: false,
            enabled_1016: false,
        }
    }

    /// 当前是否正在向程序报告鼠标事件。
    ///
    /// Whether mouse events are currently reported to the program.
    pub const fn reporting(&self) -> bool {
        self.report_filter.reporting()
    }

    /// 设置或清除 DEC 私有鼠标模式，并重新计算报告方式。
    ///
    /// Set or clear a DEC private mouse mode and recompute reporting.
    pub fn set_dec_mode(&mut self, mode: u16, enable: bool) {
        match mode {
            9 => self.enabled_9 = enable,
            1000 => self.enabled_1000 = enable,
            1001 => self.enabled_1001 = enable,
            1002 => self.enabled_1002 = enable,
            1003 => self.enabled_1003 = enable,
            1004 => self.enabled_1004 = enable,
            1005 => self.enabled_1005 = enable,
            1006 => self.enabled_1006 = enable,
            1007 => self.enabled_1007 = enable,
            1015 => self.enabled_1015 = enable,
            1016 => self.enabled_1016 = enable,
            _ => return,
        }
        self.calc_mode();
    }

    /// 重置所有鼠标模式和瞬时状态，保留指针坐标。
    ///
    /// Reset all mouse modes and transient state, keeping pointer coordinates.
    pub fn reset_modes(&mut self) {
        let x = self.x;
        let y = self.y;
        let scroll_speed = self.scroll_speed;
        *self = Self::new();
        self.x = x;
        self.y = y;
        self.scroll_speed = scroll_speed;
    }

    /// 根据当前启用的终端鼠标模式计算报告过滤器和编码方式
    ///
    /// Calculates the report filter and encoding based on the currently enabled terminal mouse modes.
    pub fn calc_mode(&mut self) {
        self.report_encoding = if self.enabled_1016 || self.enabled_1006 {
            PointerReportEncoding::SGR
        } else if self.enabled_1015 {
            PointerReportEncoding::URXVT
        } else if self.enabled_1005 {
            PointerReportEncoding::UTF8
        } else {
            PointerReportEncoding::Legacy
        };
        self.report_filter = if self.enabled_1003 {
            PointerReportFilter::AnyMotion
        } else if self.enabled_1002 {
            PointerReportFilter::ButtonMotion
        } else if self.enabled_1000 || self.enabled_1001 {
            PointerReportFilter::Button
        } else if self.enabled_9 {
            PointerReportFilter::X10
        } else {
            PointerReportFilter::None
        };
        info!("Pointer tracing mode: {:?}, encoding: {:?}", self.report_filter, self.report_encoding,);
    }

    fn cell_pos(&self, font_size: (u32, u32)) -> (i32, i32) {
        let width = font_size.0.max(1) as i32;
        let height = font_size.1.max(1) as i32;
        (self.x.div_euclid(width), self.y.div_euclid(height))
    }

    fn cb(&self, button: u8, motion: bool) -> u8 {
        let mut code = button;
        if motion {
            code = code.saturating_add(32);
        }
        if !matches!(self.report_filter, PointerReportFilter::X10) {
            code = code.saturating_add(self.modifiers);
        }
        code
    }

    fn encode_legacy(&mut self, b: u8, x: i32, y: i32) -> &[u8] {
        let x = (x + 1).clamp(1, 223) as u8;
        let y = (y + 1).clamp(1, 223) as u8;
        self.encode_buffer[0] = 0x1b;
        self.encode_buffer[1] = b'[';
        self.encode_buffer[2] = b'M';
        self.encode_buffer[3] = b + 32;
        self.encode_buffer[4] = x + 32;
        self.encode_buffer[5] = y + 32;
        &self.encode_buffer[..6]
    }

    fn encode_utf8(&mut self, b: u8, x: i32, y: i32) -> &[u8] {
        let b = char::from_u32(u32::from(b) + 32).unwrap();
        let x = char::from_u32(((x + 1).clamp(1, 2015) + 32) as u32).unwrap();
        let y = char::from_u32(((y + 1).clamp(1, 2015) + 32) as u32).unwrap();
        let mut buf = BufWriter::new(&mut self.encode_buffer);
        write!(buf, "\x1b[M{b}{x}{y}").unwrap();
        buf.into_slice()
    }

    fn encode_sgr(&mut self, b: u8, x: i32, y: i32, release: bool) -> &[u8] {
        let x = (x + 1).clamp(1, 99999);
        let y = (y + 1).clamp(1, 99999);
        let mut buf = BufWriter::new(&mut self.encode_buffer);
        write!(buf, "\x1b[<{b};{x};{y}{}", if release { 'm' } else { 'M' }).unwrap();
        buf.into_slice()
    }

    fn encode_urxvt(&mut self, b: u8, x: i32, y: i32) -> &[u8] {
        let x = (x + 1).clamp(1, 99999);
        let y = (y + 1).clamp(1, 99999);
        let mut buf = BufWriter::new(&mut self.encode_buffer);
        write!(buf, "\x1b[{};{x};{y}M", b + 32).unwrap();
        buf.into_slice()
    }

    pub fn is_pressed(&self, button: PointerButton) -> bool {
        self.button_state.is_pressed(button)
    }

    pub fn dec_mode_status(&self, mode: u16) -> u8 {
        let set = match mode {
            9 => self.enabled_9,
            1000 => self.enabled_1000,
            1001 => self.enabled_1001,
            1002 => self.enabled_1002,
            1003 => self.enabled_1003,
            1004 => self.enabled_1004,
            1005 => self.enabled_1005,
            1006 => self.enabled_1006,
            1007 => self.enabled_1007,
            1015 => self.enabled_1015,
            1016 => self.enabled_1016,
            _ => return 0,
        };
        if set { 1 } else { 2 }
    }

    fn encode_report(&mut self, button: u8, motion: bool, release: bool, font_size: (u32, u32)) -> &[u8] {
        let (x, y) = if self.enabled_1016 { (self.x, self.y) } else { self.cell_pos(font_size) };
        let b = self.cb(button, motion);
        match self.report_encoding {
            PointerReportEncoding::Legacy => self.encode_legacy(b, x, y),
            PointerReportEncoding::UTF8 => self.encode_utf8(b, x, y),
            PointerReportEncoding::SGR => self.encode_sgr(b, x, y, release),
            PointerReportEncoding::URXVT => self.encode_urxvt(b, x, y),
        }
    }

    /// 将指针移动事件编码为转义序列。
    ///
    /// Encode a pointer move event as an escape sequence.
    pub fn encode_move(&mut self, font_size: (u32, u32)) -> &[u8] {
        if !self.report_filter.report_motion(self.button_state.num_pressed() > 0) {
            return &[];
        }
        let cell = self.cell_pos(font_size);
        if self.last_cell == Some(cell) {
            return &[];
        }
        self.last_cell = Some(cell);
        let button = self.last_pressed.map(PointerButton::to_xterm_code).unwrap_or(3);
        self.encode_report(button, true, false, font_size)
    }

    /// 将滚轮滚动事件编码为转义序列。
    ///
    /// Encode a scroll wheel event as an escape sequence.
    pub fn encode_scroll(&mut self, direction: ScrollDirection, font_size: (u32, u32)) -> &[u8] {
        if !self.report_filter.report_press() {
            return &[];
        }
        let button = match direction {
            ScrollDirection::Up => 64,
            ScrollDirection::Down => 65,
            ScrollDirection::Left => 66,
            ScrollDirection::Right => 67,
        };
        self.last_cell = Some(self.cell_pos(font_size));
        self.encode_report(button, false, false, font_size)
    }

    /// 将鼠标按下事件编码为转义序列。
    ///
    /// Encode a mouse button press event as an escape sequence.
    pub fn encode_press(&mut self, button: PointerButton, font_size: (u32, u32)) -> &[u8] {
        self.button_state.press(button);
        self.last_pressed = Some(button);
        if !self.report_filter.report_press() {
            return &[];
        }
        self.last_cell = Some(self.cell_pos(font_size));
        self.encode_report(button.to_xterm_code(), false, false, font_size)
    }

    /// 将鼠标释放事件编码为转义序列。
    ///
    /// Encode a mouse button release event as an escape sequence.
    pub fn encode_release(&mut self, button: PointerButton, font_size: (u32, u32)) -> &[u8] {
        self.button_state.release(button);
        if self.last_pressed == Some(button) {
            self.last_pressed = self.button_state.first_pressed();
        }
        if !self.report_filter.report_release() {
            return &[];
        }
        self.last_cell = Some(self.cell_pos(font_size));
        match self.report_encoding {
            PointerReportEncoding::SGR => self.encode_report(button.to_xterm_code(), false, true, font_size),
            _ => self.encode_report(3, false, false, font_size),
        }
    }

    /// 处理指针进入终端区域事件。
    ///
    /// Handle the pointer entering the terminal area event.
    pub fn encode_enter(&mut self, _font_size: (u32, u32)) -> &[u8] {
        self.captured = true;
        &[]
    }

    /// 处理指针离开终端区域事件。
    ///
    /// Handle the pointer leaving the terminal area event.
    pub fn encode_leave(&mut self, _font_size: (u32, u32)) -> &[u8] {
        self.captured = false;
        &[]
    }

    pub fn forget_cell(&mut self) {
        self.last_cell = None;
    }

    /// 将焦点变化编码为转义序列（DECSET 1004）。
    ///
    /// Encode a focus change as an escape sequence (DECSET 1004).
    pub fn encode_focus(&mut self, gained: bool) -> &[u8] {
        if !self.enabled_1004 {
            return &[];
        }
        self.encode_buffer[0] = 0x1b;
        self.encode_buffer[1] = b'[';
        self.encode_buffer[2] = if gained { b'I' } else { b'O' };
        &self.encode_buffer[..3]
    }
}

// -------------------------------------------------- Testing -------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

    fn pointer_with(filter: PointerReportFilter, encoding: PointerReportEncoding) -> Pointer {
        let mut pointer = Pointer::new();
        pointer.report_filter = filter;
        pointer.report_encoding = encoding;
        pointer
    }

    #[test]
    fn utf8_mouse_encoding_uses_utf8_bytes_for_coordinates() {
        let mut pointer = pointer_with(PointerReportFilter::Button, PointerReportEncoding::UTF8);
        pointer.x = 95;
        pointer.y = 0;

        let bytes = pointer.encode_press(PointerButton::Left, (1, 1));

        assert_eq!(bytes, b"\x1b[M \xc2\x80!");
    }

    #[test]
    fn utf8_mouse_encoding_matches_legacy_for_small_coordinates() {
        let mut pointer = pointer_with(PointerReportFilter::Button, PointerReportEncoding::UTF8);
        pointer.x = 0;
        pointer.y = 0;

        let bytes = pointer.encode_press(PointerButton::Left, (1, 1));

        assert_eq!(bytes, b"\x1b[M !!");
    }

    #[test]
    fn legacy_press_and_release() {
        let mut pointer = pointer_with(PointerReportFilter::Button, PointerReportEncoding::Legacy);
        assert_eq!(pointer.encode_press(PointerButton::Left, (1, 1)), b"\x1b[M !!");
        assert_eq!(pointer.encode_release(PointerButton::Left, (1, 1)), b"\x1b[M#!!");
    }

    #[test]
    fn sgr_press_release_and_motion() {
        let mut pointer = pointer_with(PointerReportFilter::ButtonMotion, PointerReportEncoding::SGR);
        assert_eq!(pointer.encode_press(PointerButton::Left, (8, 16)), b"\x1b[<0;1;1M");
        pointer.x = 16;
        assert_eq!(pointer.encode_move((8, 16)), b"\x1b[<32;3;1M");
        pointer.x = 17;
        assert_eq!(pointer.encode_move((8, 16)), b"");
        assert_eq!(pointer.encode_release(PointerButton::Left, (8, 16)), b"\x1b[<0;3;1m");
    }

    #[test]
    fn any_motion_without_button_uses_release_code() {
        let mut pointer = pointer_with(PointerReportFilter::AnyMotion, PointerReportEncoding::SGR);
        pointer.x = 8;
        assert_eq!(pointer.encode_move((8, 16)), b"\x1b[<35;2;1M");
    }

    #[test]
    fn x10_reports_press_but_not_release() {
        let mut pointer = pointer_with(PointerReportFilter::X10, PointerReportEncoding::Legacy);
        assert_eq!(pointer.encode_press(PointerButton::Middle, (1, 1)), b"\x1b[M!!!");
        assert_eq!(pointer.encode_release(PointerButton::Middle, (1, 1)), b"");
    }

    #[test]
    fn urxvt_press_and_release() {
        let mut pointer = pointer_with(PointerReportFilter::Button, PointerReportEncoding::URXVT);
        assert_eq!(pointer.encode_press(PointerButton::Right, (1, 1)), b"\x1b[34;1;1M");
        assert_eq!(pointer.encode_release(PointerButton::Right, (1, 1)), b"\x1b[35;1;1M");
    }

    #[test]
    fn wheel_and_extra_buttons() {
        let mut pointer = pointer_with(PointerReportFilter::Button, PointerReportEncoding::SGR);
        assert_eq!(pointer.encode_scroll(ScrollDirection::Up, (1, 1)), b"\x1b[<64;1;1M");
        assert_eq!(pointer.encode_scroll(ScrollDirection::Down, (1, 1)), b"\x1b[<65;1;1M");
        assert_eq!(pointer.encode_press(PointerButton::X1, (1, 1)), b"\x1b[<128;1;1M");
        assert_eq!(pointer.encode_press(PointerButton::X2, (1, 1)), b"\x1b[<129;1;1M");
    }

    #[test]
    fn modifiers_are_added_except_in_x10() {
        let mut pointer = pointer_with(PointerReportFilter::Button, PointerReportEncoding::SGR);
        pointer.modifiers = 4 | 16;
        assert_eq!(pointer.encode_press(PointerButton::Left, (1, 1)), b"\x1b[<20;1;1M");

        let mut x10 = pointer_with(PointerReportFilter::X10, PointerReportEncoding::Legacy);
        x10.modifiers = 4 | 16;
        assert_eq!(x10.encode_press(PointerButton::Left, (1, 1)), b"\x1b[M !!");
    }

    #[test]
    fn calc_mode_keeps_encoding_independent_of_reporting() {
        let mut pointer = Pointer::new();
        pointer.set_dec_mode(1006, true);
        assert!(!pointer.reporting());
        assert_eq!(pointer.report_encoding, PointerReportEncoding::SGR);

        pointer.set_dec_mode(1000, true);
        assert_eq!(pointer.report_filter, PointerReportFilter::Button);

        pointer.set_dec_mode(1002, true);
        assert_eq!(pointer.report_filter, PointerReportFilter::ButtonMotion);

        pointer.set_dec_mode(9, true);
        assert_eq!(pointer.report_filter, PointerReportFilter::ButtonMotion);

        pointer.set_dec_mode(1002, false);
        pointer.set_dec_mode(1000, false);
        assert_eq!(pointer.report_filter, PointerReportFilter::X10);
    }

    #[test]
    fn focus_reporting() {
        let mut pointer = Pointer::new();
        assert_eq!(pointer.encode_focus(true), b"");
        pointer.set_dec_mode(1004, true);
        assert_eq!(pointer.encode_focus(true), b"\x1b[I");
        assert_eq!(pointer.encode_focus(false), b"\x1b[O");
    }
}
