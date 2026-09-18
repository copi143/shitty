//! 平台相关的终端包装（PTY 进程管理）。
//!
//! - Unix：通过 `spawn_pty_process` 创建伪终端，返回 master fd 和子进程 pid
//! - Windows：通过 `spawn_pty_process` 创建伪终端，返回 `PtyProcess` 句柄
//!
//! ---
//!
//! Platform-specific terminal wrappers (PTY process management).
//!
//! - Unix: Creates a pseudo-terminal via `spawn_pty_process`, returns master fd and child pid
//! - Windows: Creates a pseudo-terminal via `spawn_pty_process`, returns a `PtyProcess` handle

macro_rules! m {
    ($ident:ident) => {
        mod $ident;
        pub use $ident::*;
    };
}

#[cfg(unix)]
m!(unix);

#[cfg(windows)]
m!(windows);
