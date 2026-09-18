//! 对屎山 TTY 协议的简单实现：
//!
//! # 架构
//!
//! | 模块 | 职责 |
//! |------|------|
//! | [`Terminal`] | 终端状态机，协调各模块工作 |
//! | [`Parser`](ansi::Parser) | ANSI 转义序列解析器（支持多后端：vte / ansi-parser / shitty-parser / fast-parser / none） |
//! | [`TerminalBuffer`](buffer::TerminalBuffer) | 字符缓冲区，支持主屏/备用屏切换及滚动历史 |
//! | [`Screen`](buffer::Screen) | 渲染管线，支持脏区追踪和多缓冲（单/双/三缓冲） |
//! | [`FontRenderer`] | 可插拔的字体渲染接口 |
//! | [`Drawable`] | 像素输出抽象，支持多种颜色格式 |
//! | [`Callback`](callback) | PTY 读写、剪贴板、响铃、标题等外部回调 |
//!
//! # 基本用法
//!
//! ```rust
//! use shitty::{Terminal, BufMode, Drawable, EmptyFontRenderer};
//!
//! // 1. 创建字体渲染器和终端
//! let font = EmptyFontRenderer::new(8, 16);
//! let mut terminal = Terminal::new(1280, 720, BufMode::Double, vec![font]);
//!
//! // 2. 设置 PTY 写入回调（将用户输入发给子进程）
//! terminal.callbacks().pty_write = Some(Box::new(|data: &[u8]| {
//!     // 将 data 写入伪终端（PTY master）
//! }));
//!
//! // 3. 输入子进程的输出到终端
//! terminal.process(b"Hello, \x1b[31mworld\x1b[0m!\r\n");
//!
//! // 4. 创建可绘制表面并刷新
//! let mut buffer = vec![0u32; 1280 * 720];
//! let mut drawable = Drawable::new(
//!     shitty::Color::from_u32_mut_slice(&mut buffer),
//!     1280, 720, 0,
//! );
//! terminal.flush(&mut drawable, std::time::Duration::ZERO);
//!
//! // 5. 将用户输入发送给子进程
//! terminal.user_input(b"echo hello\r");
//! ```
//!
//! # Feature 分类
//!
//! - **ANSI 解析器**（五选一）：`vte`、`ansi-parser`、`shitty-parser`、`fast-parser`，默认 `shitty-parser`
//! - **字体渲染**（多选）：`font-unifont`、`font-abglyph`、`font-swash`、`font-woff2`、`font-bitmap`、`IBM_VGA_8x16`
//! - **颜色格式**（可选）：`color-formats`（开启 RGBA/BGRA/ARGB/ABGR/RGB565 等所有非默认格式）
//! - **平台**：`native`（PTY 进程支持）、`wasm`（WebAssembly）、`ffi`（C ABI）
//! - **其他**：`logging`（调试日志）、`keyboard-scancode`（原始扫描码输入）、`snapshot`（快照功能）
//!
//! 终端的大小被限制在 65535x65535 以内，光标位置从 0 开始计数，直到 width-1 和 height-1。
//!
//! 终端大小一律使用 `u32` 表示，光标位置也使用 `u32` 表示。
//!
//! ---
//!
//! A simple implementation of the shitty TTY protocol:
//!
//! # Architecture
//!
//! | Module | Responsibility |
//! |--------|---------------|
//! | [`Terminal`] | Terminal state machine, coordinates all modules |
//! | [`Parser`](ansi::Parser) | ANSI escape sequence parser (swappable backends: vte / ansi-parser / shitty-parser / fast-parser / none) |
//! | [`TerminalBuffer`](buffer::TerminalBuffer) | Character buffer with primary/alternate screen and scrollback history |
//! | [`Screen`](buffer::Screen) | Rendering pipeline with dirty region tracking and multi-buffering (single/double/triple) |
//! | [`FontRenderer`] | Pluggable font rendering interface |
//! | [`Drawable`] | Pixel output abstraction supporting multiple color formats |
//! | [`Callback`](callback) | External callbacks for PTY I/O, clipboard, bell, title |
//!
//! # Basic Usage
//!
//! ```rust
//! use shitty::{Terminal, BufMode, Drawable, EmptyFontRenderer};
//!
//! // 1. Create a font renderer and terminal
//! let font = EmptyFontRenderer::new(8, 16);
//! let mut terminal = Terminal::new(1280, 720, BufMode::Double, vec![font]);
//!
//! // 2. Set PTY write callback (forwards user input to child process)
//! terminal.callbacks().pty_write = Some(Box::new(|data: &[u8]| {
//!     // Write data to the PTY master
//! }));
//!
//! // 3. Feed program output into the terminal
//! terminal.process(b"Hello, \x1b[31mworld\x1b[0m!\r\n");
//!
//! // 4. Create a drawable surface and flush
//! let mut buffer = vec![0u32; 1280 * 720];
//! let mut drawable = Drawable::new(
//!     shitty::Color::from_u32_mut_slice(&mut buffer),
//!     1280, 720, 0,
//! );
//! terminal.flush(&mut drawable, std::time::Duration::ZERO);
//!
//! // 5. Send user input to the child process
//! terminal.user_input(b"echo hello\r");
//! ```
//!
//! # Feature Categories
//!
//! - **ANSI Parsers** (choose one): `vte`, `ansi-parser`, `shitty-parser`, `fast-parser`; default: `shitty-parser`
//! - **Font Renderers** (choose any): `font-unifont`, `font-abglyph`, `font-swash`, `font-woff2`, `font-bitmap`, `IBM_VGA_8x16`
//! - **Color Formats** (optional): `color-formats` (enables all non-default formats: RGBA/BGRA/ARGB/ABGR, RGB565, etc.)
//! - **Platforms**: `native` (PTY process support), `wasm` (WebAssembly), `ffi` (C ABI)
//! - **Misc**: `logging` (debug logging), `keyboard-scancode` (raw scancode input), `snapshot` (state snapshot)
//!
//! The terminal size is limited to 65535x65535, and the cursor position is counted from 0 to width-1 and height-1.
//!
//! The terminal size is represented by `u32`, and the cursor position is also represented by `u32`.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_default)]
#![cfg_attr(feature = "ffi", feature(linkage))]
#![cfg_attr(feature = "rustc-likely-unlikely", feature(likely_unlikely))]
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
pub use color::{ColorFormat, IColor, ansi::*, format::*, scheme::*};
pub use font::{FontRenderer, provider::*};
pub use input::{Event, KeyboardManager, PointerButton};
pub use logger::{get_log_level, set_log_level, set_logger};
pub use palette::Palette;
pub use terminal::Terminal;
