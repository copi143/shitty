use core::fmt::Debug;

use crate::color::Color;

/// A color palette for the terminal, containing a name, foreground color, background color, cursor color, and ANSI colors.
///
/// Example usage:
///
/// ```rust
/// use shitty::Palette;
///
/// let palette = Palette::new(
///     "Example Palette",
///     (0x000000, 0xffffff, 0xff0000),
///     Palette::xterm_colors(),
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Palette<'name> {
    /// The name of the palette, used for display and selection purposes.
    pub name: &'name str,
    /// The foreground color for the terminal.
    pub foreground: Color,
    /// The background color for the terminal.
    pub background: Color,
    /// The cursor color for the terminal.
    pub cursor: Color,
    /// The ANSI color palette, containing 16 colors.
    pub ansi_colors: [Color; 16],
}

impl<'name> Palette<'name> {
    /// Create a new palette with the given name, color pair, and ANSI colors.
    pub const fn new(name: &'name str, pair: (u32, u32, u32), colors: [u32; 16]) -> Self {
        Self {
            name,
            foreground: Color::from_readable_u32(pair.0),
            background: Color::from_readable_u32(pair.1),
            cursor: Color::from_readable_u32(pair.2),
            ansi_colors: {
                let mut ansi_colors = [Color::default(); 16];
                let mut i = 0;
                while i < 16 {
                    ansi_colors[i] = Color::from_readable_u32(colors[i]);
                    i += 1;
                }
                ansi_colors
            },
        }
    }

    pub const fn builtin() -> &'static [Palette<'static>] {
        &PALETTES
    }

    pub const fn default() -> &'static Palette<'static> {
        &PALETTES[DEFAULT_PALETTE_INDEX]
    }

    pub const fn default_index() -> usize {
        DEFAULT_PALETTE_INDEX
    }

    pub const fn get(index: usize) -> &'static Palette<'static> {
        if index < PALETTES.len() { &PALETTES[index] } else { Self::default() }
    }

    pub const fn xterm_colors() -> [u32; 16] {
        XTERM_COLORS
    }
}

static_assert!(!PALETTES.is_empty(), "There must be at least one palette");

include!(concat!(env!("OUT_DIR"), "/palettes.rs"));

#[rustfmt::skip]
const XTERM_COLORS: [u32; 16] = [
    0x000000, 0xcd0000, 0x00cd00, 0xcdcd00, 0x0000ee, 0xcd00cd, 0x00cdcd, 0xe5e5e5,
    0x7f7f7f, 0xff0000, 0x00ff00, 0xffff00, 0x5c5cff, 0xff00ff, 0x00ffff, 0xffffff,
];
