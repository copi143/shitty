use alloc::boxed::Box;
use core::ffi::c_void;

use crate::font::FontRenderer;
use crate::*;

macro_rules! font {
    ($name:ident $args:tt => $body:block) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn ${ concat(shitty_, $name, _new) } $args -> *mut c_void {
            let font = $body;
            Box::into_raw(Box::new(font)) as *mut c_void
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn ${ concat(shitty_, $name, _del) }(font: *mut c_void) {
            if font.is_null() {
                return;
            }
            unsafe {
                let _ = Box::from_raw(font as *mut Box<dyn FontRenderer>);
            }
        }
    };
}

#[cfg(feature = "font-unifont")]
font! {
    unifont () => {
        Unifont::new()
    }
}

#[cfg(feature = "font-bitmap")]
font! {
    bitmapfont () => {
        BitmapFont::new()
    }
}

#[cfg(feature = "font-abglyph")]
font! {
    abglyphfont (font_size: i32, font_bytes: *const u8, font_bytes_len: usize) => {
        let font_bytes = unsafe { core::slice::from_raw_parts(font_bytes, font_bytes_len) };
        AbGlyphFont::new(font_size, font_bytes)
    }
}

#[cfg(feature = "font-swash")]
font! {
    swashfont (font_size: i32, font_bytes: *const u8, font_bytes_len: usize) => {
        let font_bytes = unsafe { core::slice::from_raw_parts(font_bytes, font_bytes_len) };
        SwashFont::new(font_size, font_bytes)
    }
}

#[cfg(feature = "font-woff2")]
font! {
    woff2font (font_size: i32, font_bytes: *const u8, font_bytes_len: usize) => {
        let font_bytes = unsafe { core::slice::from_raw_parts(font_bytes, font_bytes_len) };
        Woff2Font::new(font_size, font_bytes)
    }
}

#[cfg(feature = "font-truetype")]
font! {
    truetypefont (font_size: i32, font_bytes: *const u8, font_bytes_len: usize) => {
        let font_bytes = unsafe { core::slice::from_raw_parts(font_bytes, font_bytes_len) };
        TrueTypeFont::new(font_size, font_bytes)
    }
}
