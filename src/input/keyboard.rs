use alloc::vec::Vec;

#[cfg(feature = "keyboard-scancode")]
use pc_keyboard::{
    DecodedKey as PcDecodedKey, HandleControl, KeyCode, PS2Keyboard as PcKeyboard, ScancodeSet1, layouts::Us104Key,
};
#[cfg(feature = "winit")]
pub use winit::{
    event::{ElementState as WinitElementState, Modifiers as WinitModifiers},
    keyboard::{Key as WinitKey, NamedKey as WinitNamedKey},
};

use crate::input::{Event, ScrollDirection};

/// 键盘输入管理器。负责将按键事件转换为终端转义序列。
///
/// 支持两种输入后端：
/// - `keyboard-scancode`：原始 PS/2 扫描码（裸机/无窗口环境）
/// - `winit`：winit 窗口事件（桌面环境）
///
/// Keyboard input manager. Converts key events into terminal escape sequences.
///
/// Supports two input backends:
/// - `keyboard-scancode`: Raw PS/2 scancodes (bare-metal / no-window environments)
/// - `winit`: Winit window events (desktop environments)
pub struct KeyboardManager {
    pub app_cursor_mode: bool,
    /// 是否将回车（CR）映射为回车换行（CR+LF）。
    ///
    /// Whether to map carriage return (CR) to carriage return + line feed (CR+LF).
    pub crnl_mapping: bool,
    #[cfg(feature = "keyboard-scancode")]
    keyboard: PcKeyboard<Us104Key, ScancodeSet1>,
    /// 当前修饰键状态（仅 winit 模式下使用）。
    ///
    /// Current modifier key state (only used in winit mode).
    #[cfg(feature = "winit")]
    pub modifiers: WinitModifiers,
}

impl Default for KeyboardManager {
    fn default() -> Self {
        Self {
            app_cursor_mode: false,
            crnl_mapping: false,
            #[cfg(feature = "keyboard-scancode")]
            keyboard: PcKeyboard::new(ScancodeSet1::new(), Us104Key, HandleControl::MapLettersToUnicode),
            #[cfg(feature = "winit")]
            modifiers: WinitModifiers::default(),
        }
    }
}

impl KeyboardManager {
    pub fn modifier_bits(&self) -> u8 {
        #[cfg(feature = "winit")]
        {
            let state = self.modifiers.state();
            let mut bits = 0u8;
            if state.shift_key() {
                bits |= 4;
            }
            if state.alt_key() {
                bits |= 8;
            }
            if state.control_key() {
                bits |= 16;
            }
            bits
        }
        #[cfg(not(feature = "winit"))]
        {
            let _ = self;
            0
        }
    }

    fn char_to_event(&self, c: char) -> Event {
        match c {
            '\x08' => Event::String("\x7f"),
            '\x7f' => Event::String("\x1b[3~"),
            '\n' | '\r' if !self.crnl_mapping => Event::String("\r"),
            _ => Event::Char(c),
        }
    }

    fn text_to_event(&self, text: &str) -> Event {
        let mut chars = text.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) => self.char_to_event(ch),
            _ => Event::Bytes(text.as_bytes().to_vec()),
        }
    }
}

macro_rules! app_cursor {
    ($self:expr, $normal:expr, $app:expr) => {
        if $self.app_cursor_mode { $app } else { $normal }
    };
}

#[cfg(feature = "keyboard-scancode")]
impl KeyboardManager {
    /// 处理原始 PS/2 扫描码输入（仅 `keyboard-scancode` feature）。
    ///
    /// Handle raw PS/2 scancode input (only with `keyboard-scancode` feature).
    pub fn handle_pckb_key(&mut self, scancode: u8) -> Option<Event> {
        self.keyboard
            .add_byte(scancode)
            .ok()
            .flatten()
            .and_then(|event| self.keyboard.process_keyevent(event))
            .and_then(|key| self.key_to_event(key))
    }

    fn key_to_event(&self, key: PcDecodedKey) -> Option<Event> {
        let modifiers = self.keyboard.get_modifiers();

        if modifiers.is_ctrl() && modifiers.is_shifted() {
            let raw_key = match key {
                PcDecodedKey::RawKey(k) => Some(k),
                PcDecodedKey::Unicode('\x03') => Some(KeyCode::C),
                PcDecodedKey::Unicode('\x16') => Some(KeyCode::V),
                _ => None,
            };

            if let Some(k) = raw_key {
                return self.handle_pckb_function(k);
            }
        }

        match key {
            PcDecodedKey::RawKey(k) => self.generate_ansi_sequence(k).map(Event::String),
            PcDecodedKey::Unicode(c) => Some(self.char_to_event(c)),
        }
    }

    fn handle_pckb_function(&self, key: KeyCode) -> Option<Event> {
        use KeyCode::*;
        if let Some(index) = match key {
            F1 => Some(0),
            F2 => Some(1),
            F3 => Some(2),
            F4 => Some(3),
            F5 => Some(4),
            F6 => Some(5),
            F7 => Some(6),
            F8 => Some(7),
            _ => None,
        } {
            return Some(Event::SetColorScheme(index));
        }

        match key {
            C => Some(Event::Copy),
            V => Some(Event::Paste),
            ArrowUp | PageUp => Some(Event::Scroll {
                direction: ScrollDirection::Up,
                count: 1,
                page: matches!(key, PageUp),
            }),
            ArrowDown | PageDown => Some(Event::Scroll {
                direction: ScrollDirection::Down,
                count: 1,
                page: matches!(key, PageDown),
            }),
            _ => None,
        }
    }

    fn generate_ansi_sequence(&self, key: KeyCode) -> Option<&'static str> {
        use KeyCode::*;
        let sequence = match key {
            F1 => "\x1bOP",
            F2 => "\x1bOQ",
            F3 => "\x1bOR",
            F4 => "\x1bOS",
            F5 => "\x1b[15~",
            F6 => "\x1b[17~",
            F7 => "\x1b[18~",
            F8 => "\x1b[19~",
            F9 => "\x1b[20~",
            F10 => "\x1b[21~",
            F11 => "\x1b[23~",
            F12 => "\x1b[24~",
            ArrowUp => app_cursor!(self, "\x1b[A", "\x1bOA"),
            ArrowDown => app_cursor!(self, "\x1b[B", "\x1bOB"),
            ArrowRight => app_cursor!(self, "\x1b[C", "\x1bOC"),
            ArrowLeft => app_cursor!(self, "\x1b[D", "\x1bOD"),
            Home => "\x1b[H",
            End => "\x1b[F",
            PageUp => "\x1b[5~",
            PageDown => "\x1b[6~",
            _ => return None,
        };
        Some(sequence)
    }
}

#[cfg(feature = "winit")]
impl KeyboardManager {
    /// 处理 winit 键盘事件（仅 `winit` feature）。
    ///
    /// Handle winit keyboard event (only with `winit` feature).
    pub fn handle_winit_key(
        &self,
        state: WinitElementState,
        logical_key: &WinitKey,
        text: Option<&str>,
    ) -> Option<Event> {
        if !matches!(state, WinitElementState::Pressed) {
            return None;
        }

        if self.modifiers.state().control_key() && self.modifiers.state().shift_key() {
            return self.handle_winit_function(logical_key);
        }

        if let Some(text) = text.filter(|s| !s.is_empty()) {
            if self.modifiers.state().alt_key()
                && !self.modifiers.state().control_key()
                && !self.modifiers.state().super_key()
            {
                let mut input = Vec::with_capacity(1 + text.len());
                input.push(0x1b);
                input.extend_from_slice(text.as_bytes());
                return Some(Event::Bytes(input));
            }
            return Some(self.text_to_event(text));
        }

        match logical_key {
            WinitKey::Named(key) => self.named_key_to_event(*key),
            WinitKey::Character(ch) => Some(self.text_to_event(ch.as_ref())),
            _ => None,
        }
    }

    fn handle_winit_function(&self, key: &WinitKey) -> Option<Event> {
        use WinitNamedKey::*;
        if let WinitKey::Named(named) = key {
            if let Some(index) = match named {
                F1 => Some(0),
                F2 => Some(1),
                F3 => Some(2),
                F4 => Some(3),
                F5 => Some(4),
                F6 => Some(5),
                F7 => Some(6),
                F8 => Some(7),
                _ => None,
            } {
                return Some(Event::SetColorScheme(index));
            }

            return match named {
                Copy => Some(Event::Copy),
                Paste => Some(Event::Paste),
                ArrowUp | PageUp => Some(Event::Scroll {
                    direction: ScrollDirection::Up,
                    count: 1,
                    page: matches!(named, PageUp),
                }),
                ArrowDown | PageDown => Some(Event::Scroll {
                    direction: ScrollDirection::Down,
                    count: 1,
                    page: matches!(named, PageDown),
                }),
                _ => None,
            };
        }

        if let WinitKey::Character(ch) = key {
            if ch.eq_ignore_ascii_case("c") {
                return Some(Event::Copy);
            }
            if ch.eq_ignore_ascii_case("v") {
                return Some(Event::Paste);
            }
        }
        None
    }

    fn named_key_to_event(&self, key: WinitNamedKey) -> Option<Event> {
        use WinitNamedKey::*;
        self.named_key_to_ansi_sequence(key).map(Event::String).or_else(|| {
            Some(match key {
                Enter => Event::String("\r"),
                Tab => Event::String("\t"),
                Space => Event::Char(' '),
                Backspace => Event::String("\x7f"),
                Escape => Event::String("\x1b"),
                Copy => Event::Copy,
                Paste => Event::Paste,
                _ => return None,
            })
        })
    }

    fn named_key_to_ansi_sequence(&self, key: WinitNamedKey) -> Option<&'static str> {
        use WinitNamedKey::*;
        let sequence = match key {
            F1 => "\x1bOP",
            F2 => "\x1bOQ",
            F3 => "\x1bOR",
            F4 => "\x1bOS",
            F5 => "\x1b[15~",
            F6 => "\x1b[17~",
            F7 => "\x1b[18~",
            F8 => "\x1b[19~",
            F9 => "\x1b[20~",
            F10 => "\x1b[21~",
            F11 => "\x1b[23~",
            F12 => "\x1b[24~",
            F13 => "\x1b[25~",
            F14 => "\x1b[26~",
            F15 => "\x1b[28~",
            F16 => "\x1b[29~",
            F17 => "\x1b[31~",
            F18 => "\x1b[32~",
            F19 => "\x1b[33~",
            F20 => "\x1b[34~",
            F21 => "\x1b[42~",
            F22 => "\x1b[43~",
            F23 => "\x1b[44~",
            F24 => "\x1b[45~",
            F25 => "\x1b[46~",
            F26 => "\x1b[47~",
            F27 => "\x1b[48~",
            F28 => "\x1b[49~",
            F29 => "\x1b[50~",
            F30 => "\x1b[51~",
            F31 => "\x1b[52~",
            F32 => "\x1b[53~",
            F33 => "\x1b[54~",
            F34 => "\x1b[55~",
            F35 => "\x1b[56~",
            ArrowUp => app_cursor!(self, "\x1b[A", "\x1bOA"),
            ArrowDown => app_cursor!(self, "\x1b[B", "\x1bOB"),
            ArrowRight => app_cursor!(self, "\x1b[C", "\x1bOC"),
            ArrowLeft => app_cursor!(self, "\x1b[D", "\x1bOD"),
            Home => "\x1b[H",
            End => "\x1b[F",
            Insert => "\x1b[2~",
            Delete => "\x1b[3~",
            PageUp => "\x1b[5~",
            PageDown => "\x1b[6~",
            _ => return None,
        };
        Some(sequence)
    }
}
