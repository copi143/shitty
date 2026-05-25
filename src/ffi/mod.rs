#[allow(unused_imports)]
use core::panic::PanicInfo;

use crate::*;

mod alloc;
mod callback;
mod font;
mod terminal;

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //

#[panic_handler]
#[cfg(not(feature = "shutup-rust-analyzer"))]
fn panic_handler(info: &PanicInfo) -> ! {
    let str = ::alloc::ffi::CString::new(format!("{}", info)).unwrap();
    unsafe { callback::shitty_callback_panic(str.as_ptr() as *const _) };
}

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //

macro_rules! ffi_consts {
    ($($name:ident => $value:expr),* $(,)?) => {
        $(
            ffi_const!($name, usize, $value as usize);
        )*
    };
}

ffi_consts! {
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
