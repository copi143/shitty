use alloc::boxed::Box;
use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_void;
#[allow(unused_imports)]
use core::panic::PanicInfo;
use core::time::Duration;
use js_sys::Function;
use wasm_bindgen::prelude::*;
use wee_alloc::WeeAlloc;

use crate::CursorShape;
use crate::font::FontRenderer;
use crate::*;

mod font;

#[panic_handler]
#[cfg(not(feature = "shutup-rust-analyzer"))]
fn panic_handler(info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //

macro_rules! wasm_consts {
    ($($name:ident => $value:expr),* $(,)?) => {
        $(
            wasm_const!($name, usize, $value as usize);
        )*
    };
}

wasm_consts! {
    // ++ shitty::buffer::screen::BufMode ++ //
    SHITTY_BUFMODE_NONE => BufMode::None,
    SHITTY_BUFMODE_SINGLE => BufMode::Single,
    SHITTY_BUFMODE_DOUBLE => BufMode::Double,
    SHITTY_BUFMODE_TRIPLE => BufMode::Triple,

    // ++ shitty::ansi::CursorShape ++ //
    SHITTY_CURSOR_SHAPE_BLOCK => CursorShape::Block,
    SHITTY_CURSOR_SHAPE_UNDERLINE => CursorShape::Underline,
    SHITTY_CURSOR_SHAPE_BEAM => CursorShape::Beam,
    SHITTY_CURSOR_SHAPE_HOLLOW_BLOCK => CursorShape::HollowBlock,
    SHITTY_CURSOR_SHAPE_HIDDEN => CursorShape::Hidden,

    // ++ shitty::MouseButton ++ //
    SHITTY_MOUSE_BUTTON_LEFT => PointerButton::Left,
    SHITTY_MOUSE_BUTTON_RIGHT => PointerButton::Right,
    SHITTY_MOUSE_BUTTON_MIDDLE => PointerButton::Middle,
    SHITTY_MOUSE_BUTTON_X1 => PointerButton::X1,
    SHITTY_MOUSE_BUTTON_X2 => PointerButton::X2,

    // ++ shitty::AutoWrap ++ //
    SHITTY_AUTOWRAP_DISABLED => AutoWrap::Disabled,
    SHITTY_AUTOWRAP_IMMEDIATE => AutoWrap::Immediate,
    SHITTY_AUTOWRAP_DELAYED => AutoWrap::Delayed,
}

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //

#[wasm_bindgen]
pub fn shitty_alloc_display_buffer(width: usize, height: usize) -> *mut c_void {
    let layout = Layout::array::<Color>(width * height).unwrap();
    unsafe { ALLOC.alloc(layout) as *mut c_void }
}

#[wasm_bindgen]
pub fn shitty_dealloc_display_buffer(ptr: *mut c_void, width: usize, height: usize) {
    if ptr.is_null() {
        return;
    }
    let layout = Layout::array::<Color>(width * height).unwrap();
    unsafe { ALLOC.dealloc(ptr as *mut u8, layout) };
}

#[wasm_bindgen]
pub fn shitty_alloc_string_buffer(len: usize) -> *mut c_void {
    let layout = Layout::array::<u8>(len).unwrap();
    unsafe { ALLOC.alloc(layout) as *mut c_void }
}

#[wasm_bindgen]
pub fn shitty_dealloc_string_buffer(ptr: *mut c_void, len: usize) {
    if ptr.is_null() {
        return;
    }
    let layout = Layout::array::<u8>(len).unwrap();
    unsafe { ALLOC.dealloc(ptr as *mut u8, layout) };
}

/// Create a new terminal instance.
/// > See [`Terminal::new`] for more details.
#[wasm_bindgen]
pub fn shitty_new(width: u32, height: u32, bufmode: usize, font: *mut c_void) -> *mut Terminal {
    if width == 0 || height == 0 {
        return core::ptr::null_mut();
    }
    let Ok(bufmode) = BufMode::try_from(bufmode) else {
        return core::ptr::null_mut();
    };
    let font = *unsafe { Box::from_raw(font as *mut Box<dyn FontRenderer>) };
    let terminal = Terminal::new(width, height, bufmode, vec![font]);
    Box::into_raw(Box::new(terminal))
}

/// Delete a terminal instance.
/// > See [`Terminal`] for more details.
#[wasm_bindgen]
pub fn shitty_del(terminal: *mut Terminal) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(terminal);
    }
}

/// Resize the terminal instance.
/// > See [`Terminal::resize`] for more details.
#[wasm_bindgen]
pub fn shitty_resize(terminal: *mut Terminal, width: u32, height: u32) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).resize(width, height);
    }
}

/// Returns whether the terminal is dirty (i.e., needs to be redrawn).
#[wasm_bindgen]
pub fn shitty_isdirty(terminal: *mut Terminal) -> bool {
    if terminal.is_null() {
        return false;
    }
    unsafe { (*terminal).is_dirty() }
}

/// Returns the number of rows in the terminal.
#[wasm_bindgen]
pub fn shitty_rows(terminal: *mut Terminal) -> u32 {
    if terminal.is_null() {
        return 0;
    }
    unsafe { (*terminal).rows() }
}

/// Returns the number of columns in the terminal.
#[wasm_bindgen]
pub fn shitty_cols(terminal: *mut Terminal) -> u32 {
    if terminal.is_null() {
        return 0;
    }
    unsafe { (*terminal).cols() }
}

/// Flush the terminal's output.
/// > See [`Terminal::flush`] for more details.
#[wasm_bindgen]
pub fn shitty_flush(
    terminal: *mut Terminal,
    width: usize,
    height: usize,
    pitch: usize,
    buf: *mut c_void,
    time_ms: u64,
) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        let mut drawable = Drawable::from_raw_parts(buf as *mut Color, width, height, pitch);
        (*terminal).flush(&mut drawable, Duration::from_millis(time_ms));
    }
}

#[wasm_bindgen]
pub fn shitty_process(terminal: *mut Terminal, input: *const u8, len: usize) {
    if terminal.is_null() || input.is_null() || len == 0 {
        return;
    }
    unsafe {
        let input = core::slice::from_raw_parts(input, len);
        (*terminal).process(input);
    }
}

#[wasm_bindgen]
pub fn shitty_process_byte(terminal: *mut Terminal, byte: u8) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).process(byte);
    }
}

#[wasm_bindgen]
pub fn shitty_process_char(terminal: *mut Terminal, char: char) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).process(char);
    }
}

#[wasm_bindgen]
pub fn shitty_input(terminal: *mut Terminal, input: *const u8, len: usize) {
    if terminal.is_null() || input.is_null() || len == 0 {
        return;
    }
    unsafe {
        let text = core::slice::from_raw_parts(input, len);
        (*terminal).user_input(text);
    }
}

// #[wasm_bindgen]
// #[cfg(feature = "keyboard-scancode")]
// pub fn shitty_handle_keyboard_scancode(terminal: *mut Terminal, scancode: u8) {
//     if terminal.is_null() {
//         return;
//     }
//     unsafe {
//         (*terminal).handle_keyboard_scancode(scancode);
//     }
// }

#[wasm_bindgen]
pub fn shitty_set_callback_ptywrite(terminal: *mut Terminal, callback: Function) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).callbacks().pty_write = Some(Box::new(move |s: &[u8]| {
            let _ = callback.call1(&JsValue::NULL, &JsValue::from_str(core::str::from_utf8_unchecked(s)));
        }));
    }
}

#[wasm_bindgen]
pub fn shitty_set_callback_clipboard(terminal: *mut Terminal, get: Function, set: Function) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).callbacks().clipboard_get =
            Some(Box::new(move || get.call0(&JsValue::NULL).map(|v| v.as_string()).ok().flatten()));
        (*terminal).callbacks().clipboard_set = Some(Box::new(move |text| {
            let _ = set.call1(&JsValue::NULL, &JsValue::from_str(&text));
        }));
    }
}

#[wasm_bindgen]
pub fn shitty_set_callback_bell(terminal: *mut Terminal, callback: Function) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).callbacks().bell = Some(Box::new(move || {
            let _ = callback.call0(&JsValue::NULL);
        }));
    }
}

#[wasm_bindgen]
pub fn shitty_set_callback_title(terminal: *mut Terminal, callback: Function) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).callbacks().title = Some(Box::new(move |title| {
            let js_title = title.as_deref().map(JsValue::from_str).unwrap_or(JsValue::NULL);
            let _ = callback.call1(&JsValue::NULL, &js_title);
        }));
    }
}
