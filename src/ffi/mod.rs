//! C FFI（外部函数接口）。允许从 C、C++、Kotlin、Python 等语言调用 shitty。
//!
//! 按功能分为 4 个子模块：
//! - `terminal`：终端生命周期和操作（`shitty_new`、`shitty_flush` 等）
//! - `font`：字体提供者创建（`shitty_unifont_new` 等）
//! - `callback`：外部回调 typedef（`PtyWriteC`、`ClipboardGetC` 等）
//! - `alloc`：分配器回调（当不使用 `ffi-std` 时需要外部提供）
//!
//! 不带 `ffi-std` feature 构建时，调用方必须提供 `shitty_callback_*` 分配器实现。
//! 参见 `examples/c-x11/` 获取完整示例。
//!
//! ---
//!
//! C FFI (Foreign Function Interface). Allows calling shitty from C, C++, Kotlin,
//! Python, and other languages.
//!
//! Divided into 4 submodules:
//! - `terminal`: Terminal lifecycle and operations (`shitty_new`, `shitty_flush`, etc.)
//! - `font`: Font provider creation (`shitty_unifont_new`, etc.)
//! - `callback`: External callback typedefs (`PtyWriteC`, `ClipboardGetC`, etc.)
//! - `alloc`: Allocator callbacks (required when building without `ffi-std`)
//!
//! When built without `ffi-std`, the caller must provide `shitty_callback_*` allocator
//! implementations. See `examples/c-x11/` for a complete working example.

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

    // ++ shitty::ColorFormat ++ //
    SHITTY_COLORFMT_DEFAULT => ColorFormat::Default,
}

#[cfg(feature = "color-formats")]
ffi_consts! {
    // ++ shitty::ColorFormat ++ //
    SHITTY_COLORFMT_RGBA => ColorFormat::RGBA,
    SHITTY_COLORFMT_BGRA => ColorFormat::BGRA,
    SHITTY_COLORFMT_ARGB => ColorFormat::ARGB,
    SHITTY_COLORFMT_ABGR => ColorFormat::ABGR,
    SHITTY_COLORFMT_RGBA16 => ColorFormat::RGBA16,
    SHITTY_COLORFMT_BGRA16 => ColorFormat::BGRA16,
    SHITTY_COLORFMT_ARGB16 => ColorFormat::ARGB16,
    SHITTY_COLORFMT_ABGR16 => ColorFormat::ABGR16,
    SHITTY_COLORFMT_RGB => ColorFormat::RGB,
    SHITTY_COLORFMT_BGR => ColorFormat::BGR,
    SHITTY_COLORFMT_RGBA_F32 => ColorFormat::RGBA_F32,
    SHITTY_COLORFMT_BGRA_F32 => ColorFormat::BGRA_F32,
    SHITTY_COLORFMT_ARGB_F32 => ColorFormat::ARGB_F32,
    SHITTY_COLORFMT_ABGR_F32 => ColorFormat::ABGR_F32,
    SHITTY_COLORFMT_RGB_F32 => ColorFormat::RGB_F32,
    SHITTY_COLORFMT_BGR_F32 => ColorFormat::BGR_F32,
    SHITTY_COLORFMT_RGB565 => ColorFormat::RGB565,
    SHITTY_COLORFMT_BGR565 => ColorFormat::BGR565,
}

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //
