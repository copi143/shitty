use core::fmt::Write as _;

use crate::helper::BufWriter;

enum_map! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum PointerButton : usize {
        Left = 0,
        Right = 1,
        Middle = 2,
        X1 = 3,
        X2 = 4,
    }
}

impl PointerButton {
    pub const fn to_xterm_code(self) -> u8 {
        match self {
            PointerButton::Left => 0,
            PointerButton::Middle => 1,
            PointerButton::Right => 2,
            PointerButton::X1 => 4,
            PointerButton::X2 => 5,
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PointerButtonState {
    flags: u32,
}

impl PointerButtonState {
    pub const fn new() -> Self {
        PointerButtonState { flags: 0 }
    }

    pub const fn clear(&mut self) {
        self.flags = 0;
    }

    pub const fn press(&mut self, button: PointerButton) {
        self.flags |= 1 << button as usize;
    }

    pub const fn release(&mut self, button: PointerButton) {
        self.flags &= !(1 << button as usize);
    }

    pub const fn is_pressed(&self, button: PointerButton) -> bool {
        (self.flags & (1 << button as usize)) != 0
    }

    pub const fn is_released(&self, button: PointerButton) -> bool {
        !self.is_pressed(button)
    }

    pub const fn num_pressed(&self) -> u32 {
        self.flags.count_ones()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PointerReportFilter {
    #[default]
    None, // 无鼠标
    X10,          // 9
    Button,       // 1000
    ButtonMotion, // 1002
    AnyMotion,    // 1003
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
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PointerReportEncoding {
    #[default]
    Legacy, // ESC [ M
    UTF8,  // 1005
    SGR,   // 1006
    URXVT, // 1015
}

pub struct Pointer {
    pub x: i32,
    pub y: i32,
    pub captured: bool,
    pub scroll_speed: i32,

    button_state: PointerButtonState,
    report_filter: PointerReportFilter,
    report_encoding: PointerReportEncoding,
    encode_buffer: [u8; 64],

    // 终端鼠标模式启用状态
    // \x1b[?1000h - 鼠标点击报告
    // \x1b[?1002h - 鼠标点击和拖动报告
    // \x1b[?1003h - 鼠标移动报告
    // \x1b[?1005h - UTF-8 编码坐标
    // \x1b[?1006h - SGR 编码坐标
    // \x1b[?1015h - URXVT 编码坐标
    pub enabled_1000: bool,
    pub enabled_1002: bool,
    pub enabled_1003: bool,
    pub enabled_1005: bool,
    pub enabled_1006: bool,
    pub enabled_1015: bool,
}

impl Pointer {
    pub const fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            captured: false,
            scroll_speed: 3,
            button_state: PointerButtonState::new(),
            report_filter: PointerReportFilter::None,
            report_encoding: PointerReportEncoding::Legacy,
            encode_buffer: [0; 64],

            enabled_1000: false,
            enabled_1002: false,
            enabled_1003: false,
            enabled_1005: false,
            enabled_1006: false,
            enabled_1015: false,
        }
    }

    /// 根据当前启用的终端鼠标模式计算报告过滤器和编码方式
    ///
    /// Calculates the report filter and encoding based on the currently enabled terminal mouse modes.
    pub fn calc_mode(&mut self) {
        self.report_filter = PointerReportFilter::None;
        if self.enabled_1006 {
            self.report_encoding = PointerReportEncoding::SGR;
            self.report_filter = PointerReportFilter::Button;
        } else if self.enabled_1015 {
            self.report_encoding = PointerReportEncoding::URXVT;
            self.report_filter = PointerReportFilter::Button;
        } else if self.enabled_1005 {
            self.report_encoding = PointerReportEncoding::UTF8;
            self.report_filter = PointerReportFilter::Button;
        } else {
            self.report_encoding = PointerReportEncoding::Legacy;
        }
        if self.enabled_1003 {
            self.report_filter = PointerReportFilter::AnyMotion;
        } else if self.enabled_1002 {
            self.report_filter = PointerReportFilter::ButtonMotion;
        } else if self.enabled_1000 {
            self.report_filter = PointerReportFilter::Button;
        }
        info!("Pointer tracing mode: {:?}, encoding: {:?}", self.report_filter, self.report_encoding,);
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

    pub fn encode_move(&mut self, font_size: (u32, u32)) -> &[u8] {
        if !self.report_filter.report_motion(self.button_state.num_pressed() > 0) {
            return &[];
        }
        let x = self.x / font_size.0 as i32;
        let y = self.y / font_size.1 as i32;
        let b = 32; // motion flag
        match self.report_encoding {
            PointerReportEncoding::Legacy => self.encode_legacy(b, x, y),
            PointerReportEncoding::UTF8 => self.encode_utf8(b, x, y),
            PointerReportEncoding::SGR => self.encode_sgr(b, x, y, true),
            PointerReportEncoding::URXVT => self.encode_urxvt(b, x, y),
        }
    }

    pub fn encode_scroll(&mut self, delta: i32, font_size: (u32, u32)) -> &[u8] {
        if !self.report_filter.report_press() {
            return &[];
        }
        let x = self.x / font_size.0 as i32;
        let y = self.y / font_size.1 as i32;
        let b = if delta > 0 { 64 } else { 65 };
        match self.report_encoding {
            PointerReportEncoding::Legacy => self.encode_legacy(b, x, y),
            PointerReportEncoding::UTF8 => self.encode_utf8(b, x, y),
            PointerReportEncoding::SGR => self.encode_sgr(b, x, y, false),
            PointerReportEncoding::URXVT => self.encode_urxvt(b, x, y),
        }
    }

    pub fn encode_press(&mut self, button: PointerButton, font_size: (u32, u32)) -> &[u8] {
        self.button_state.press(button);
        if !self.report_filter.report_press() {
            return &[];
        }
        let x = self.x / font_size.0 as i32;
        let y = self.y / font_size.1 as i32;
        let b = button.to_xterm_code();
        match self.report_encoding {
            PointerReportEncoding::Legacy => self.encode_legacy(b, x, y),
            PointerReportEncoding::UTF8 => self.encode_utf8(b, x, y),
            PointerReportEncoding::SGR => self.encode_sgr(b, x, y, false),
            PointerReportEncoding::URXVT => self.encode_urxvt(b, x, y),
        }
    }

    pub fn encode_release(&mut self, button: PointerButton, font_size: (u32, u32)) -> &[u8] {
        self.button_state.release(button);
        if !self.report_filter.report_release() {
            return &[];
        }
        let x = self.x / font_size.0 as i32;
        let y = self.y / font_size.1 as i32;
        match self.report_encoding {
            PointerReportEncoding::SGR => {
                let b = button.to_xterm_code();
                self.encode_sgr(b, x, y, true)
            }
            PointerReportEncoding::UTF8 => self.encode_utf8(3, x, y),
            _ => self.encode_legacy(3, x, y), // legacy / urxvt: release = button 3
        }
    }

    pub fn encode_enter(&mut self, _font_size: (u32, u32)) -> &[u8] {
        self.button_state.clear();
        self.captured = true;
        &[]
    }

    pub fn encode_leave(&mut self, _font_size: (u32, u32)) -> &[u8] {
        self.button_state.clear();
        self.captured = false;
        &[]
    }
}

// -------------------------------------------------- Testing -------------------------------------------------- //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_mouse_encoding_uses_utf8_bytes_for_coordinates() {
        let mut pointer = Pointer::new();
        pointer.report_filter = PointerReportFilter::Button;
        pointer.report_encoding = PointerReportEncoding::UTF8;
        pointer.x = 95;
        pointer.y = 0;

        let bytes = pointer.encode_press(PointerButton::Left, (1, 1));

        assert_eq!(bytes, b"\x1b[M \xc2\x80!");
    }

    #[test]
    fn utf8_mouse_encoding_matches_legacy_for_small_coordinates() {
        let mut pointer = Pointer::new();
        pointer.report_filter = PointerReportFilter::Button;
        pointer.report_encoding = PointerReportEncoding::UTF8;
        pointer.x = 0;
        pointer.y = 0;

        let bytes = pointer.encode_press(PointerButton::Left, (1, 1));

        assert_eq!(bytes, b"\x1b[M !!");
    }
}
