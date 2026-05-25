use core::fmt::Display;

#[cfg(feature = "vte")]
pub use vte::ansi::{Color as VteColor, NamedColor as VteNamedColor, Rgb as VteRgb};

#[repr(align(4))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnsiColor {
    Named(AnsiNamedColor),
    Spec(AnsiRgb),
    Indexed(u8),
}

impl const From<AnsiNamedColor> for AnsiColor {
    fn from(color: AnsiNamedColor) -> Self {
        AnsiColor::Named(color)
    }
}

impl const From<AnsiRgb> for AnsiColor {
    fn from(rgb: AnsiRgb) -> Self {
        AnsiColor::Spec(rgb)
    }
}

impl const From<u8> for AnsiColor {
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
impl const From<AnsiColor> for VteColor {
    fn from(color: AnsiColor) -> Self {
        match color {
            AnsiColor::Named(named) => VteColor::Named(named.into()),
            AnsiColor::Spec(rgb) => VteColor::Spec(rgb.into()),
            AnsiColor::Indexed(index) => VteColor::Indexed(index),
        }
    }
}

#[cfg(feature = "vte")]
impl const From<VteColor> for AnsiColor {
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

#[rustfmt::skip]
impl AnsiNamedColor {
    pub const fn from_named_fg(code: u8) -> Option<Self> {
        Some(match code {
            30 => Self::Black,
            31 => Self::Red,
            32 => Self::Green,
            33 => Self::Yellow,
            34 => Self::Blue,
            35 => Self::Magenta,
            36 => Self::Cyan,
            37 => Self::White,
            90 => Self::BrightBlack,
            91 => Self::BrightRed,
            92 => Self::BrightGreen,
            93 => Self::BrightYellow,
            94 => Self::BrightBlue,
            95 => Self::BrightMagenta,
            96 => Self::BrightCyan,
            97 => Self::BrightWhite,
            _ => return None,
        })
    }

    pub const fn to_named_fg(self) -> Option<u8> {
        Some(match self {
            Self::Black => 30,
            Self::Red => 31,
            Self::Green => 32,
            Self::Yellow => 33,
            Self::Blue => 34,
            Self::Magenta => 35,
            Self::Cyan => 36,
            Self::White => 37,
            Self::BrightBlack => 90,
            Self::BrightRed => 91,
            Self::BrightGreen => 92,
            Self::BrightYellow => 93,
            Self::BrightBlue => 94,
            Self::BrightMagenta => 95,
            Self::BrightCyan => 96,
            Self::BrightWhite => 97,
            _ => return None,
        })
    }

    pub const fn from_named_bg(code: u8) -> Option<Self> {
        Some(match code {
            40 => Self::Black,
            41 => Self::Red,
            42 => Self::Green,
            43 => Self::Yellow,
            44 => Self::Blue,
            45 => Self::Magenta,
            46 => Self::Cyan,
            47 => Self::White,
            100 => Self::BrightBlack,
            101 => Self::BrightRed,
            102 => Self::BrightGreen,
            103 => Self::BrightYellow,
            104 => Self::BrightBlue,
            105 => Self::BrightMagenta,
            106 => Self::BrightCyan,
            107 => Self::BrightWhite,
            _ => return None,
        })
    }

    pub const fn to_named_bg(self) -> Option<u8> {
        Some(match self {
            Self::Black => 40,
            Self::Red => 41,
            Self::Green => 42,
            Self::Yellow => 43,
            Self::Blue => 44,
            Self::Magenta => 45,
            Self::Cyan => 46,
            Self::White => 47,
            Self::BrightBlack => 100,
            Self::BrightRed => 101,
            Self::BrightGreen => 102,
            Self::BrightYellow => 103,
            Self::BrightBlue => 104,
            Self::BrightMagenta => 105,
            Self::BrightCyan => 106,
            Self::BrightWhite => 107,
            _ => return None,
        })
    }

    pub const fn is_normal(self) -> bool {
        use AnsiNamedColor::*;
        matches!(self, Black | Red | Green | Yellow | Blue | Magenta | Cyan | White | Foreground | Background | Cursor)
    }

    pub const fn is_bright(self) -> bool {
        use AnsiNamedColor::*;
        matches!(self, BrightBlack | BrightRed | BrightGreen | BrightYellow | BrightBlue | BrightMagenta | BrightCyan | BrightWhite | BrightForeground)
    }

    pub const fn is_dim(self) -> bool {
        use AnsiNamedColor::*;
        matches!(self, DimBlack | DimRed | DimGreen | DimYellow | DimBlue | DimMagenta | DimCyan | DimWhite | DimForeground)
    }

    pub const fn brighter(self) -> Self {
        use AnsiNamedColor::*;
        match self {
            Black => BrightBlack,
            Red => BrightRed,
            Green => BrightGreen,
            Yellow => BrightYellow,
            Blue => BrightBlue,
            Magenta => BrightMagenta,
            Cyan => BrightCyan,
            White => BrightWhite,
            Foreground => BrightForeground,
            DimBlack => Black,
            DimRed => Red,
            DimGreen => Green,
            DimYellow => Yellow,
            DimBlue => Blue,
            DimMagenta => Magenta,
            DimCyan => Cyan,
            DimWhite => White,
            DimForeground => Foreground,
            other => other,
        }
    }

    pub const fn dimmer(self) -> Self {
        use AnsiNamedColor::*;
        match self {
            Black => DimBlack,
            Red => DimRed,
            Green => DimGreen,
            Yellow => DimYellow,
            Blue => DimBlue,
            Magenta => DimMagenta,
            Cyan => DimCyan,
            White => DimWhite,
            Foreground => DimForeground,
            BrightBlack => Black,
            BrightRed => Red,
            BrightGreen => Green,
            BrightYellow => Yellow,
            BrightBlue => Blue,
            BrightMagenta => Magenta,
            BrightCyan => Cyan,
            BrightWhite => White,
            BrightForeground => Foreground,
            other => other,
        }
    }

    pub const fn normal(self) -> Self {
        use AnsiNamedColor::*;
        match self {
            BrightBlack | DimBlack => Black,
            BrightRed | DimRed => Red,
            BrightGreen | DimGreen => Green,
            BrightYellow | DimYellow => Yellow,
            BrightBlue | DimBlue => Blue,
            BrightMagenta | DimMagenta => Magenta,
            BrightCyan | DimCyan => Cyan,
            BrightWhite | DimWhite => White,
            BrightForeground | DimForeground => Foreground,
            other => other,
        }
    }

    pub const fn bright(self) -> Self {
        use AnsiNamedColor::*;
        match self {
            Black | DimBlack => BrightBlack,
            Red | DimRed => BrightRed,
            Green | DimGreen => BrightGreen,
            Yellow | DimYellow => BrightYellow,
            Blue | DimBlue => BrightBlue,
            Magenta | DimMagenta => BrightMagenta,
            Cyan | DimCyan => BrightCyan,
            White | DimWhite => BrightWhite,
            Foreground | DimForeground => BrightForeground,
            other => other,
        }
    }

    pub const fn dim(self) -> Self {
        use AnsiNamedColor::*;
        match self {
            Black | BrightBlack => DimBlack,
            Red | BrightRed => DimRed,
            Green | BrightGreen => DimGreen,
            Yellow | BrightYellow => DimYellow,
            Blue | BrightBlue => DimBlue,
            Magenta | BrightMagenta => DimMagenta,
            Cyan | BrightCyan => DimCyan,
            White | BrightWhite => DimWhite,
            Foreground | BrightForeground => DimForeground,
            other => other,
        }
    }
}

impl Display for AnsiNamedColor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // `f.alternate()` selects background when true, foreground when false.
        // Handle special cases explicitly.
        match self {
            AnsiNamedColor::Cursor => return Ok(()), // No standard SGR code for cursor color; skip.
            AnsiNamedColor::BrightForeground => return write!(f, "\x1b[1m"),
            AnsiNamedColor::DimForeground => return write!(f, "\x1b[2m"),
            AnsiNamedColor::Foreground => {
                return if f.alternate() { write!(f, "\x1b[49m") } else { write!(f, "\x1b[39m") };
            }
            AnsiNamedColor::Background => return write!(f, "\x1b[49m"),
            _ => {}
        }

        // Map dim variants to their base (non-dim) color so we can reuse the
        // named FG/BG codes and prefix with the dim SGR when needed.
        let is_dim = self.is_dim();

        let base = match self {
            AnsiNamedColor::DimBlack => AnsiNamedColor::Black,
            AnsiNamedColor::DimRed => AnsiNamedColor::Red,
            AnsiNamedColor::DimGreen => AnsiNamedColor::Green,
            AnsiNamedColor::DimYellow => AnsiNamedColor::Yellow,
            AnsiNamedColor::DimBlue => AnsiNamedColor::Blue,
            AnsiNamedColor::DimMagenta => AnsiNamedColor::Magenta,
            AnsiNamedColor::DimCyan => AnsiNamedColor::Cyan,
            AnsiNamedColor::DimWhite => AnsiNamedColor::White,
            other => *other,
        };

        if let Some(code) = if f.alternate() { base.to_named_bg() } else { base.to_named_fg() } {
            if is_dim { write!(f, "\x1b[2;{}m", code) } else { write!(f, "\x1b[{}m", code) }
        } else {
            // Fallback: should not happen for known named colors; keep no-op.
            Ok(())
        }
    }
}

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnsiRgb {
    pub r: u8,
    pub g: u8,
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
impl const From<AnsiRgb> for VteRgb {
    fn from(rgb: AnsiRgb) -> Self {
        VteRgb {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }
}

#[cfg(feature = "vte")]
impl const From<VteRgb> for AnsiRgb {
    fn from(rgb: VteRgb) -> Self {
        AnsiRgb {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }
}

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //
