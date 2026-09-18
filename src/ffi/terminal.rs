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
    if font.is_null() || width == 0 || height == 0 {
        return core::ptr::null_mut();
    }
    let Ok(bufmode) = BufMode::try_from(bufmode) else {
        drop(unsafe { Box::from_raw(font as *mut Box<dyn FontRenderer>) });
        return core::ptr::null_mut();
    };
    let font = unsafe { Box::from_raw(font as *mut Box<dyn FontRenderer>) };
    let terminal = Terminal::new(width, height, bufmode, vec![*font]);
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
/// - `color`: a [`ColorFormat`] discriminant value (see `SHITTY_COLORFMT_*` constants).
/// > See [`Terminal::flush`] for more details.
#[unsafe(no_mangle)]
pub extern "C" fn shitty_flush(
    terminal: *mut Terminal,
    width: usize,
    height: usize,
    pitch: usize,
    buf: *mut c_void,
    time_ms: u64,
    color: usize,
) {
    if terminal.is_null() {
        return;
    }
    let Ok(color) = ColorFormat::try_from(color) else {
        return;
    };
    unsafe {
        color.dispatch_flush(&mut *terminal, buf, width, height, pitch, Duration::from_millis(time_ms));
    }
}

/// 处理原始字节输入（通常来自 PTY）。
/// > See [`Terminal::process`] for more details.
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

/// 处理单个字节的终端输入。
/// > See [`Terminal::process`] for more details.
#[unsafe(no_mangle)]
pub extern "C" fn shitty_process_byte(terminal: *mut Terminal, byte: u8) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).process(byte);
    }
}

/// 向终端写入一个 Unicode 码点。
/// > See [`Terminal::put`] for more details.
#[unsafe(no_mangle)]
pub extern "C" fn shitty_put(terminal: *mut Terminal, c: u32) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).put(char::from_u32(c).unwrap_or('\u{FFFD}'));
    }
}

/// 处理用户文本输入（来自键盘输入法）。
/// > See [`Terminal::user_input`] for more details.
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

/// 处理鼠标移动事件。
/// > See [`Event::PointerMove`] for more details.
#[unsafe(no_mangle)]
pub extern "C" fn shitty_handle_mouse_move(terminal: *mut Terminal, x: i32, y: i32) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).handle_event(Event::PointerMove(x, y));
    }
}

/// 处理鼠标按下事件。
/// > See [`Event::PointerPress`] for more details.
#[unsafe(no_mangle)]
pub extern "C" fn shitty_handle_mouse_press(terminal: *mut Terminal, button: usize) {
    if terminal.is_null() {
        return;
    }
    let Ok(button) = PointerButton::try_from(button) else {
        return;
    };
    unsafe {
        (*terminal).handle_event(Event::PointerPress(button));
    }
}

/// 处理鼠标释放事件。
/// > See [`Event::PointerRelease`] for more details.
#[unsafe(no_mangle)]
pub extern "C" fn shitty_handle_mouse_release(terminal: *mut Terminal, button: usize) {
    if terminal.is_null() {
        return;
    }
    let Ok(button) = PointerButton::try_from(button) else {
        return;
    };
    unsafe {
        (*terminal).handle_event(Event::PointerRelease(button));
    }
}

/// 处理鼠标滚轮事件。正值向上，负值向下。
/// > See [`Event::scroll`] for more details.
#[unsafe(no_mangle)]
pub extern "C" fn shitty_handle_mouse_scroll(terminal: *mut Terminal, lines: i32) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).handle_event(Event::scroll(lines));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shitty_handle_mouse_scroll_xy(terminal: *mut Terminal, dx: i32, dy: i32) {
    if terminal.is_null() {
        return;
    }
    if let Some(event) = Event::scroll_xy(dx, dy) {
        unsafe {
            (*terminal).handle_event(event);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shitty_handle_modifiers(terminal: *mut Terminal, modifiers: u8) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).pointer.modifiers = modifiers;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shitty_handle_mouse_leave(terminal: *mut Terminal) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).handle_event(Event::PointerLeave);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn shitty_handle_focus(terminal: *mut Terminal, gained: bool) {
    if terminal.is_null() {
        return;
    }
    unsafe {
        (*terminal).handle_event(Event::Focus(gained));
    }
}

/// 设置 PTY 写入回调函数。
/// - 回调接收一个字节切片，用于将终端输出写入 PTY。
/// - 传入 `null` 以清除回调。
///
/// Set the PTY write callback function.
/// - The callback receives a byte slice for writing terminal output to the PTY.
/// - Pass `null` to clear the callback.
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

/// 设置剪贴板读写回调函数。
/// - `get`: 回调函数，返回以 null 结尾的字符串，或返回 `null` 表示无内容。
/// - `set`: 回调函数，接收一个字节切片作为剪贴板内容。
/// - 传入 `null` 以清除对应回调。
///
/// Set the clipboard read/write callback functions.
/// - `get`: Callback returning a null-terminated string, or `null` for no content.
/// - `set`: Callback receiving a byte slice as clipboard content.
/// - Pass `null` to clear the corresponding callback.
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

/// 设置终端响铃回调函数。
/// - 传入 `null` 以清除回调。
///
/// Set the terminal bell callback function.
/// - Pass `null` to clear the callback.
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

/// 设置终端标题变更回调函数。
/// - 回调接收一个字节切片作为标题，或接收 `null`/空切片表示清除标题。
/// - 传入 `null` 以清除回调。
///
/// Set the terminal title change callback function.
/// - The callback receives a byte slice as the title, or `null`/empty slice to clear.
/// - Pass `null` to clear the callback.
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
