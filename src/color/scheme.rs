use aligned_vec::{ABox, avec};
use core::ops::{Deref, DerefMut};

use crate::color::{AnsiColor, AnsiNamedColor, Color, IColor};
use crate::palette::Palette;

static_assert!(core::mem::size_of::<ColorScheme<Color>>() == core::mem::size_of::<usize>());

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorScheme<T: IColor>(ABox<[T; 512]>);

impl<T: IColor> Default for ColorScheme<T> {
    fn default() -> Self {
        Self::from(Palette::default())
    }
}

impl<T: IColor> Deref for ColorScheme<T> {
    type Target = [T; 512];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: IColor> DerefMut for ColorScheme<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: IColor> ColorScheme<T> {
    pub fn new(palette: &Palette) -> Self {
        Self::from(palette)
    }

    pub fn get(&self, color: AnsiColor) -> T {
        match color {
            AnsiColor::Spec(rgb) => T::from(Color::from(rgb)),
            AnsiColor::Named(named) => self[named as usize],
            AnsiColor::Indexed(index) => self[index as usize],
        }
    }

    pub fn black(&self) -> T {
        self[AnsiNamedColor::Black as usize]
    }

    pub fn red(&self) -> T {
        self[AnsiNamedColor::Red as usize]
    }

    pub fn green(&self) -> T {
        self[AnsiNamedColor::Green as usize]
    }

    pub fn yellow(&self) -> T {
        self[AnsiNamedColor::Yellow as usize]
    }

    pub fn blue(&self) -> T {
        self[AnsiNamedColor::Blue as usize]
    }

    pub fn magenta(&self) -> T {
        self[AnsiNamedColor::Magenta as usize]
    }

    pub fn cyan(&self) -> T {
        self[AnsiNamedColor::Cyan as usize]
    }

    pub fn white(&self) -> T {
        self[AnsiNamedColor::White as usize]
    }

    pub fn foreground(&self) -> T {
        self[AnsiNamedColor::Foreground as usize]
    }

    pub fn background(&self) -> T {
        self[AnsiNamedColor::Background as usize]
    }

    pub fn cursor(&self) -> T {
        self[AnsiNamedColor::Cursor as usize]
    }
}

impl<T: IColor> From<&Palette<'_>> for ColorScheme<T> {
    fn from(palette: &Palette) -> Self {
        const fn palette256_scale(c: usize) -> u8 {
            if c == 0 { 0 } else { (c * 40 + 55) as u8 }
        }

        let mut colors = ABox::new(0, [T::default(); 512]);

        for index in 0..16 {
            colors[index] = palette.ansi_colors[index].into();
        }

        for index in 0..216 {
            let r = palette256_scale(index / 36);
            let g = palette256_scale(index % 36 / 6);
            let b = palette256_scale(index % 6);
            colors[index + 16] = Color::rgb(r, g, b).into();
        }

        for index in 0..24 {
            let luminance = (index * 10 + 8) as u8;
            colors[index + 16 + 216] = Color::rgb(luminance, luminance, luminance).into();
        }

        colors[AnsiNamedColor::Foreground as usize] = palette.foreground.into();
        colors[AnsiNamedColor::Background as usize] = palette.background.into();
        colors[AnsiNamedColor::Cursor as usize] = palette.cursor.into();

        Self(colors)
    }
}
