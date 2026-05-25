use ::alloc::borrow::ToOwned;
use ::alloc::boxed::Box;
use core::ffi::c_void;
use core::time::Duration;

use crate::callback::*;
use crate::font::FontRenderer;
use crate::*;

/// Create a new terminal instance.
/// - [`bufmode`](BufMode) is enum, not integer.
/// > See [`Terminal::new`] for more details.
#[unsafe(no_mangle)]
pub extern "C" fn shitty_new(width: u32, height: u32, bufmode: usize, font: *mut c_void) -> *mut Terminal {
    if width == 0 || height == 0 {
        return core::ptr::null_mut();
    }
    let Ok(bufmode) = BufMode::try_from(bufmode) else {
        return core::ptr::null_mut();
    };
    let terminal =
        Terminal::new(width, height, bufmode, vec![*unsafe { Box::from_raw(font as *mut Box<dyn FontRenderer>) }]);
    Box::into_raw(Box::new(terminal))
}

/// Delete a terminal instance.
/// > See [`Terminal`] for more details.
#[unsafe(no_mangle)]
pub extern "C" fn shitty_del(terminal: *mut Terminal) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(terminal);
    }
}

/// Resize the terminal instance.
/// > See [`Terminal::resize`] for more details.
#[unsafe(no_mangle)]
pub extern "C" fn shitty_resize(terminal: *mut Terminal, width: u32, height: u32) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).resize(width, height);
    }
}

/// Returns whether the terminal is dirty (i.e., needs to be redrawn).
#[unsafe(no_mangle)]
pub extern "C" fn shitty_isdirty(terminal: *mut Terminal) -> bool {
    if terminal.is_null() {
        return false;
    }
    unsafe { (*terminal).is_dirty() }
}

/// Returns the number of rows in the terminal.
#[unsafe(no_mangle)]
pub extern "C" fn shitty_rows(terminal: *mut Terminal) -> u32 {
    if terminal.is_null() {
        return 0;
    }
    unsafe { (*terminal).rows() }
}

/// Returns the number of columns in the terminal.
#[unsafe(no_mangle)]
pub extern "C" fn shitty_cols(terminal: *mut Terminal) -> u32 {
    if terminal.is_null() {
        return 0;
    }
    unsafe { (*terminal).cols() }
}

/// Flush the terminal's output.
/// > See [`Terminal::flush`] for more details.
#[unsafe(no_mangle)]
pub extern "C" fn shitty_flush(
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

#[unsafe(no_mangle)]
pub extern "C" fn shitty_process(terminal: *mut Terminal, input: *const u8, len: usize) {
    if terminal.is_null() || input.is_null() || len == 0 {
        return;
    }
    unsafe {
        let input = core::slice::from_raw_parts(input, len);
        (*terminal).process(input);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shitty_process_byte(terminal: *mut Terminal, byte: u8) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).process(byte);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shitty_put(terminal: *mut Terminal, c: u32) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).put(char::from_u32(c).unwrap_or('\u{FFFD}'));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shitty_input(terminal: *mut Terminal, input: *const u8, len: usize) {
    if terminal.is_null() || input.is_null() || len == 0 {
        return;
    }
    unsafe {
        let text = core::slice::from_raw_parts(input, len);
        (*terminal).user_input(text);
    }
}

// #[unsafe(no_mangle)]
// #[cfg(feature = "keyboard-scancode")]
// pub extern "C" fn shitty_handle_keyboard_scancode(terminal: *mut Terminal, scancode: u8) {
//     if terminal.is_null() {
//         return;
//     }
//     unsafe {
//         (*terminal).handle_keyboard_scancode(scancode);
//     }
// }

#[unsafe(no_mangle)]
pub extern "C" fn shitty_handle_mouse_move(terminal: *mut Terminal, x: i32, y: i32) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).handle_event(Event::PointerMove(x, y));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shitty_set_callback_ptywrite(terminal: *mut Terminal, callback: PtyWriteC) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        if callback as usize == 0 {
            (*terminal).callbacks().pty_write = None;
        } else {
            (*terminal).callbacks().pty_write = Some(Box::new(move |s: &[u8]| {
                callback(s.as_ptr(), s.len());
            }));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shitty_set_callback_clipboard(terminal: *mut Terminal, get: ClipboardGetC, set: ClipboardSetC) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        if get as usize == 0 {
            (*terminal).callbacks().clipboard_get = None;
        } else {
            (*terminal).callbacks().clipboard_get = Some(Box::new(move || {
                let ptr = get();
                if ptr.is_null() {
                    None
                } else {
                    let mut len = 0;
                    while *ptr.add(len) != 0 {
                        len += 1;
                    }
                    let slice = core::slice::from_raw_parts(ptr, len);
                    match core::str::from_utf8(slice) {
                        Ok(s) => Some(s.to_owned()),
                        Err(_) => None,
                    }
                }
            }));
        }
        if set as usize == 0 {
            (*terminal).callbacks().clipboard_set = None;
        } else {
            (*terminal).callbacks().clipboard_set = Some(Box::new(move |text| {
                set(text.as_ptr(), text.len());
            }));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shitty_set_callback_bell(terminal: *mut Terminal, callback: BellC) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        if callback as usize == 0 {
            (*terminal).callbacks().bell = None;
        } else {
            (*terminal).callbacks().bell = Some(Box::new(move || {
                callback();
            }));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shitty_set_callback_title(terminal: *mut Terminal, callback: TitleC) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        if callback as usize == 0 {
            (*terminal).callbacks().title = None;
        } else {
            (*terminal).callbacks().title = Some(Box::new(move |title| match title {
                Some(ref s) => callback(s.as_ptr(), s.len()),
                None => callback(core::ptr::null(), 0),
            }));
        }
    }
}
