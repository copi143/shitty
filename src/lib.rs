//! 对屎山 TTY 协议的简单实现：
//!
//! 其中终端的大小被限制在 65536x65536 以内，光标位置从 0 开始计数，直到 width-1 和 height-1。
//!
//! 终端大小一律使用 `u32` 表示，光标位置也使用 `u32` 表示。
//!
//! ---
//!
//! A simple implementation of the shitty TTY protocol:
//!
//! The terminal size is limited to 65536x65536, and the cursor position is counted from 0 to width-1 and height-1.
//!
//! The terminal size is represented by `u32`, and the cursor position is also represented by `u32`.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_default)]
#![cfg_attr(feature = "ffi", feature(linkage))]
#![cfg_attr(feature = "rustc-likely-unlikely", feature(likely_unlikely))]
// #![feature(unboxed_closures)]
#![feature(macro_metavar_expr_concat)]
#![forbid(unused_must_use)]
#![deny(unsafe_code)]
#![expect(clippy::missing_transmute_annotations)]

#[macro_use]
extern crate alloc;

#[macro_use]
mod helper;

#[macro_use]
mod logger;

#[cfg(feature = "ffi")]
#[expect(unsafe_code)]
mod ffi;

#[cfg(feature = "wasm")]
#[expect(unsafe_code)]
mod wasm;

#[cfg(feature = "native")]
#[expect(unsafe_code)]
pub mod native;

#[macro_use]
mod color;

mod ansi;
mod buffer;
mod font;
mod input;
mod palette;
mod terminal;

pub mod callback;

pub use ansi::CursorShape;
pub use buffer::{AutoWrap, BufMode, Drawable, OwnedDrawable};
pub use color::{AnsiColor, AnsiNamedColor, AnsiRgb, BGRA, Color, RGBA};
pub use font::{FontRenderer, provider::*};
pub use input::{Event, KeyboardManager, PointerButton};
pub use logger::{get_log_level, set_log_level, set_logger};
pub use palette::Palette;
pub use terminal::Terminal;
