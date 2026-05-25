//! Font providers for shitty.
#![allow(unused_imports)]

mod empty;
pub use empty::EmptyFontRenderer;

#[allow(non_camel_case_types)]
#[cfg(feature = "IBM_VGA_8x16")]
mod ibm_vga_8x16;

#[cfg(feature = "IBM_VGA_8x16")]
pub use ibm_vga_8x16::IBM_VGA_8x16;

#[cfg(feature = "font-abglyph")]
mod abglyph;
#[cfg(feature = "font-bitmap")]
mod bitmap;
#[cfg(feature = "font-swash")]
mod swash;
#[cfg(feature = "font-truetype")]
mod truetype;
#[cfg(feature = "font-unifont")]
mod unifont;
#[cfg(feature = "font-woff2")]
mod woff2;

#[cfg(feature = "font-abglyph")]
pub use abglyph::AbGlyphFont;
#[cfg(feature = "font-bitmap")]
pub use bitmap::BitmapFont;
#[cfg(feature = "font-swash")]
pub use swash::SwashFont;
#[cfg(feature = "font-truetype")]
pub use truetype::TrueTypeFont;
#[cfg(feature = "font-unifont")]
pub use unifont::Unifont;
#[cfg(feature = "font-woff2")]
pub use woff2::Woff2Font;
