//! 输入处理。将键盘和指针（鼠标）事件转换为终端转义序列或内部操作。
//!
//! 核心类型：
//! - [`Event`]：统一输入事件枚举（键盘、鼠标、剪贴板等）
//! - [`KeyboardManager`]：键盘按键映射（支持 winit 和原始扫描码）
//! - [`Pointer`]：鼠标事件编码（标准/UTF-8/SGR 模式）
//!
//! ---
//!
//! Input handling. Converts keyboard and pointer (mouse) events into terminal
//! escape sequences or internal operations.
//!
//! Core types:
//! - [`Event`]: Unified input event enum (keyboard, mouse, clipboard, etc.)
//! - [`KeyboardManager`]: Keyboard key mapping (supports winit and raw scancode)
//! - [`Pointer`]: Mouse event encoding (standard/UTF-8/SGR mode)

mod event;
pub use event::*;

mod keyboard;
pub use keyboard::*;

mod pointer;
pub use pointer::*;
