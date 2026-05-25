use alloc::boxed::Box;
use core::ffi::c_void;
use wasm_bindgen::prelude::*;

use crate::font::FontRenderer;
use crate::*;

#[wasm_bindgen]
#[cfg(feature = "font-unifont")]
pub fn shitty_unifont_new() -> *mut c_void {
    let font = Unifont::new();
    Box::into_raw(Box::new(font)) as *mut c_void
}

#[wasm_bindgen]
#[cfg(feature = "font-unifont")]
pub fn shitty_unifont_del(font: *mut c_void) {
    if font.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(font as *mut Box<dyn FontRenderer>);
    }
}

#[wasm_bindgen]
#[cfg(feature = "font-bitmap")]
pub fn shitty_bitmapfont_new() -> *mut c_void {
    let font = BitmapFont::new();
    Box::into_raw(Box::new(font)) as *mut c_void
}

#[wasm_bindgen]
#[cfg(feature = "font-bitmap")]
pub fn shitty_bitmapfont_del(font: *mut c_void) {
    if font.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(font as *mut Box<dyn FontRenderer>);
    }
}

#[wasm_bindgen]
#[cfg(feature = "font-truetype")]
pub fn shitty_truetypefont_new(font_size: i32, font_bytes: *const u8, font_bytes_len: usize) -> *mut c_void {
    let font_bytes = unsafe { core::slice::from_raw_parts(font_bytes, font_bytes_len) };
    let font = TrueTypeFont::new(font_size, font_bytes);
    Box::into_raw(Box::new(font)) as *mut c_void
}

#[wasm_bindgen]
#[cfg(feature = "font-truetype")]
pub fn shitty_truetypefont_del(font: *mut c_void) {
    if font.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(font as *mut Box<dyn FontRenderer>);
    }
}
