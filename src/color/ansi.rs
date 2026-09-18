use core::fmt::Display;

#[cfg(feature = "vte")]
pub use vte::ansi::{Color as VteColor, NamedColor as VteNamedColor, Rgb as VteRgb};

/// ANSI 颜色值。可以是命名颜色、RGB 指定色或 256 色调色板索引。
///
/// ANSI color value. Can be a named color, an RGB spec, or a 256-color palette index.
#[repr(align(4))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnsiColor {
    /// 命名颜色（16 ANSI 色 + 特殊色）。
    ///
    /// Named color (16 ANSI colors + special colors).
    Named(AnsiNamedColor),
    /// 24 位 RGB 指定色。
    ///
    /// 24-bit RGB specified color.
    Spec(AnsiRgb),
    /// 256 色调色板索引。
    ///
    /// 256-color palette index.
    Indexed(u8),
}

const impl From<AnsiNamedColor> for AnsiColor {
    fn from(color: AnsiNamedColor) -> Self {
        AnsiColor::Named(color)
    }
}

const impl From<AnsiRgb> for AnsiColor {
    fn from(rgb: AnsiRgb) -> Self {
        AnsiColor::Spec(rgb)
    }
}

const impl From<u8> for AnsiColor {
    fn from(index: u8) -> Self {
        AnsiColor::Indexed(index)
    }
}

impl Display for AnsiColor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AnsiColor::Named(named) => named.fmt(f),
            AnsiColor::Spec(rgb) => rgb.fmt(f),
            AnsiColor::Indexed(index) => {
                write!(f, "\x1b[{};5;{index}m", if f.alternate() { "48" } else { "38" })
            }
        }
    }
}

#[cfg(feature = "vte")]
const impl From<AnsiColor> for VteColor {
    fn from(color: AnsiColor) -> Self {
        match color {
            AnsiColor::Named(named) => VteColor::Named(named.into()),
            AnsiColor::Spec(rgb) => VteColor::Spec(rgb.into()),
            AnsiColor::Indexed(index) => VteColor::Indexed(index),
        }
    }
}

#[cfg(feature = "vte")]
const impl From<VteColor> for AnsiColor {
    fn from(color: VteColor) -> Self {
        match color {
            VteColor::Named(named) => AnsiColor::Named(named.into()),
            VteColor::Spec(rgb) => AnsiColor::Spec(rgb.into()),
            VteColor::Indexed(index) => AnsiColor::Indexed(index),
        }
    }
}

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //

enum_map! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum AnsiNamedColor : usize {
        /// Black.
        Black = 0,
        /// Red.
        Red = 1,
        /// Green.
        Green = 2,
        /// Yellow.
        Yellow = 3,
        /// Blue.
        Blue = 4,
        /// Magenta.
        Magenta = 5,
        /// Cyan.
        Cyan = 6,
        /// White.
        White = 7,
        /// Bright black.
        BrightBlack = 8,
        /// Bright red.
        BrightRed = 9,
        /// Bright green.
        BrightGreen = 10,
        /// Bright yellow.
        BrightYellow = 11,
        /// Bright blue.
        BrightBlue = 12,
        /// Bright magenta.
        BrightMagenta = 13,
        /// Bright cyan.
        BrightCyan = 14,
        /// Bright white.
        BrightWhite = 15,
        /// The foreground color.
        Foreground = 256,
        /// The background color.
        Background = 257,
        /// Color for the cursor itself.
        Cursor = 258,
        /// Dim black.
        DimBlack = 259,
        /// Dim red.
        DimRed = 260,
        /// Dim green.
        DimGreen = 261,
        /// Dim yellow.
        DimYellow = 262,
        /// Dim blue.
        DimBlue = 263,
        /// Dim magenta.
        DimMagenta = 264,
        /// Dim cyan.
        DimCyan = 265,
        /// Dim white.
        DimWhite = 266,
        /// The bright foreground color.
        BrightForeground = 267,
        /// Dim foreground.
        DimForeground = 268,
    }
    --- SAME AS ---
    #[cfg(feature = "vte")]
    VteNamedColor,
}

/// Intensity ladder: normal → bright → (no brighter), dim → normal → bright.
const INTENSITY_NORMAL: u8 = 0;
const INTENSITY_BRIGHT: u8 = 1;
const INTENSITY_DIM: u8 = 2;

/// Index of the special foreground family in [`AnsiNamedColor::family`].
const FAMILY_FOREGROUND: u8 = 8;

impl AnsiNamedColor {
    /// `(family, intensity)` for colors that participate in the bright/dim ladder.
    ///
    /// - `family` 0..=7: the 8 ANSI colors; 8: Foreground
    /// - `intensity`: 0 normal, 1 bright, 2 dim
    ///
    /// Background / Cursor return `None`.
    const fn family(self) -> Option<(u8, u8)> {
        match self as usize {
            n @ 0..=7 => Some((n as u8, INTENSITY_NORMAL)),
            n @ 8..=15 => Some(((n - 8) as u8, INTENSITY_BRIGHT)),
            n @ 259..=266 => Some(((n - 259) as u8, INTENSITY_DIM)),
            x if x == Self::Foreground as usize => Some((FAMILY_FOREGROUND, INTENSITY_NORMAL)),
            x if x == Self::BrightForeground as usize => Some((FAMILY_FOREGROUND, INTENSITY_BRIGHT)),
            x if x == Self::DimForeground as usize => Some((FAMILY_FOREGROUND, INTENSITY_DIM)),
            _ => None,
        }
    }

    const fn from_family(family: u8, intensity: u8) -> Self {
        if family == FAMILY_FOREGROUND {
            return match intensity {
                INTENSITY_BRIGHT => Self::BrightForeground,
                INTENSITY_DIM => Self::DimForeground,
                _ => Self::Foreground,
            };
        }
        let n = match intensity {
            INTENSITY_BRIGHT => 8 + family as usize,
            INTENSITY_DIM => 259 + family as usize,
            _ => family as usize,
        };
        match Self::try_from(n) {
            Ok(c) => c,
            Err(()) => Self::Black,
        }
    }

    const fn with_intensity(self, intensity: u8) -> Self {
        match self.family() {
            Some((family, _)) => Self::from_family(family, intensity),
            None => self,
        }
    }

    /// 从前景色 ANSI 代码解析为命名颜色。
    ///
    /// Parse an ANSI foreground color code into a named color.
    pub const fn from_named_fg(code: u8) -> Option<Self> {
        let idx = match code {
            30..=37 => (code - 30) as usize,
            90..=97 => (code - 90 + 8) as usize,
            _ => return None,
        };
        match Self::try_from(idx) {
            Ok(c) => Some(c),
            Err(()) => None,
        }
    }

    /// 将命名颜色转换为前景色 ANSI 代码。
    ///
    /// Convert a named color to an ANSI foreground color code.
    pub const fn to_named_fg(self) -> Option<u8> {
        match self as usize {
            n @ 0..=7 => Some(30 + n as u8),
            n @ 8..=15 => Some(90 + (n as u8 - 8)),
            _ => None,
        }
    }

    /// 从背景色 ANSI 代码解析为命名颜色。
    ///
    /// Parse an ANSI background color code into a named color.
    pub const fn from_named_bg(code: u8) -> Option<Self> {
        // BG codes are FG codes + 10 (40..=47 / 100..=107).
        Self::from_named_fg(code.wrapping_sub(10))
    }

    /// 将命名颜色转换为背景色 ANSI 代码。
    ///
    /// Convert a named color to an ANSI background color code.
    pub const fn to_named_bg(self) -> Option<u8> {
        match self.to_named_fg() {
            Some(c) => Some(c + 10),
            None => None,
        }
    }

    /// 检查是否为普通（非亮色/非暗色）颜色。
    ///
    /// Check if this is a normal (not bright, not dim) color.
    pub const fn is_normal(self) -> bool {
        matches!(self as usize, 0..=7 | 256..=258)
    }

    /// 检查是否为亮色变体。
    ///
    /// Check if this is a bright variant.
    pub const fn is_bright(self) -> bool {
        matches!(self as usize, 8..=15 | 267)
    }

    /// 检查是否为暗色变体。
    ///
    /// Check if this is a dim variant.
    pub const fn is_dim(self) -> bool {
        matches!(self as usize, 259..=266 | 268)
    }

    /// 获取更亮的变体（normal → bright，dim → normal，bright → bright）。
    ///
    /// Get the brighter variant (normal → bright, dim → normal, bright → bright).
    pub const fn brighter(self) -> Self {
        match self.family() {
            Some((family, INTENSITY_NORMAL)) => Self::from_family(family, INTENSITY_BRIGHT),
            Some((family, INTENSITY_DIM)) => Self::from_family(family, INTENSITY_NORMAL),
            _ => self,
        }
    }

    /// 获取更暗的变体（normal → dim，bright → normal，dim → dim）。
    ///
    /// Get the dimmer variant (normal → dim, bright → normal, dim → dim).
    pub const fn dimmer(self) -> Self {
        match self.family() {
            Some((family, INTENSITY_NORMAL)) => Self::from_family(family, INTENSITY_DIM),
            Some((family, INTENSITY_BRIGHT)) => Self::from_family(family, INTENSITY_NORMAL),
            _ => self,
        }
    }

    /// 标准化为普通变体（bright/dim → normal）。
    ///
    /// Normalize to the normal variant (bright/dim → normal).
    pub const fn normal(self) -> Self {
        self.with_intensity(INTENSITY_NORMAL)
    }

    /// 强制转换为亮色变体。
    ///
    /// Force conversion to the bright variant.
    pub const fn bright(self) -> Self {
        self.with_intensity(INTENSITY_BRIGHT)
    }

    /// 强制转换为暗色变体。
    ///
    /// Force conversion to the dim variant.
    pub const fn dim(self) -> Self {
        self.with_intensity(INTENSITY_DIM)
    }
}

impl Display for AnsiNamedColor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // `f.alternate()` selects background when true, foreground when false.
        match self {
            AnsiNamedColor::Cursor => return Ok(()),
            AnsiNamedColor::BrightForeground => return write!(f, "\x1b[1m"),
            AnsiNamedColor::DimForeground => return write!(f, "\x1b[2m"),
            AnsiNamedColor::Foreground => {
                return if f.alternate() { write!(f, "\x1b[49m") } else { write!(f, "\x1b[39m") };
            }
            AnsiNamedColor::Background => return write!(f, "\x1b[49m"),
            _ => {}
        }

        let is_dim = self.is_dim();
        let base = self.normal();
        if let Some(code) = if f.alternate() { base.to_named_bg() } else { base.to_named_fg() } {
            if is_dim { write!(f, "\x1b[2;{}m", code) } else { write!(f, "\x1b[{}m", code) }
        } else {
            Ok(())
        }
    }
}

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //

/// 24 位 RGB 颜色值（用于 ANSI 转义序列中的 `\x1b[38;2;R;G;Bm`）。
///
/// 24-bit RGB color value (used in ANSI escape sequences like `\x1b[38;2;R;G;Bm`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnsiRgb {
    /// 红色分量 (0-255)。
    ///
    /// Red component (0-255).
    pub r: u8,
    /// 绿色分量 (0-255)。
    ///
    /// Green component (0-255).
    pub g: u8,
    /// 蓝色分量 (0-255)。
    ///
    /// Blue component (0-255).
    pub b: u8,
}

impl Display for AnsiRgb {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            write!(f, "\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
        } else {
            write!(f, "\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
        }
    }
}

#[cfg(feature = "vte")]
const impl From<AnsiRgb> for VteRgb {
    fn from(rgb: AnsiRgb) -> Self {
        VteRgb {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }
}

#[cfg(feature = "vte")]
const impl From<VteRgb> for AnsiRgb {
    fn from(rgb: VteRgb) -> Self {
        AnsiRgb {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }
}

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //
