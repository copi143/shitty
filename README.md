# shitty

<img align="left" width="192" src="docs/assets/shitty.svg" alt="logo" />

A simple implementation of the shitty TTY protocol.

A cross-platform terminal emulator library written in Rust.

A modular terminal emulator supporting swappable ANSI parsers, font renderers, and pixel formats.

A no_std terminal core suitable for embedded systems and bare-metal environments.

A WebAssembly-compatible terminal emulator that runs in the browser.

A terminal emulator with C FFI bindings for integration with non-Rust applications.

A terminal featuring configurable multi-buffered rendering with dirty region tracking for efficient redraws.

**The example program can currently only be compiled on amd64 Linux and Windows, but the library itself should be portable to other platforms.**

***WIP!!!***

## Quick Start

Add shitty to your `Cargo.toml`:

```toml
[dependencies]
shitty = "0.0.1"
```

Basic usage:

```rust
use shitty::{Terminal, BufMode, Drawable, EmptyFontRenderer};

// Create a font renderer and terminal
let font = EmptyFontRenderer::new(8, 16);
let mut terminal = Terminal::new(1280, 720, BufMode::Double, vec![font]);

// Set PTY write callback
terminal.callbacks().pty_write = Some(Box::new(|data: &[u8]| {
    // Forward data to the PTY master
}));

// Feed program output into the terminal
terminal.process(b"Hello, \x1b[31mworld\x1b[0m!\r\n");

// Create a drawable surface and flush
let mut buffer = vec![0u32; 1280 * 720];
let mut drawable = Drawable::new(
    shitty::Color::from_u32_mut_slice(&mut buffer),
    1280, 720, 0,
);
terminal.flush(&mut drawable, std::time::Duration::ZERO);

// Send user input to the child process
terminal.user_input(b"echo hello\r");
```

## Feature Flags

### ANSI Parsers (choose one)

| Feature         | Description                                     |
|-----------------|-------------------------------------------------|
| `vte`           | Full-featured parser using the `vte` crate      |
| `ansi-parser`   | Parser using the `ansi-parser` crate            |
| `shitty-parser` | Simple built-in parser **(default)**            |
| `fast-parser`   | Simple but faster built-in parser               |
| *(none)*        | Raw pass-through, no escape sequence processing |

### Font Renderers (choose any)

| Feature        | Backend                 | Description                                     |
|----------------|-------------------------|-------------------------------------------------|
| `font-unifont` | `libbaremetal-unifont`  | Unicode bitmap font, no_std compatible          |
| `font-abglyph` | `ab_glyph`              | TrueType/OpenType via ab_glyph                  |
| `font-swash`   | `swash`                 | TrueType/OpenType via swash (variable fonts)    |
| `font-woff2`   | `woff2-no-std`          | WOFF2 decompression (requires abglyph or swash) |
| `font-bitmap`  | `noto-sans-mono-bitmap` | Embedded Noto Sans Mono bitmap                  |
| `IBM_VGA_8x16` | built-in                | Classic VGA hardware font                       |

### Color Formats

| Feature | Description |
|---------|-------------|
| `color-formats` | Enables all additional color types (RGBA/BGRA/ARGB/ABGR, 16-bit & f32 variants, RGB/BGR, RGB565/BGR565) |

The in-memory channel order for `Color` is controlled by:

| Features                             | Channel Order | Description                                    |
|--------------------------------------|---------------|------------------------------------------------|
| *(none)*                             | R G B A       | Red-first 32-bit **(default for WASM)**        |
| `color-b-before-r`                   | B G R A       | Blue-first 32-bit **(default for native/FFI)** |
| `color-a-first`                      | A R G B       | Alpha-first 32-bit                             |
| `color-a-first` + `color-b-before-r` | A B G R       | Alpha-blue-green-red 32-bit                    |

### Platforms

| Feature  | Description                           |
|----------|---------------------------------------|
| `native` | PTY process support (unix/windows)    |
| `wasm`   | WebAssembly bindings via wasm-bindgen |
| `ffi`    | C ABI bindings for non-Rust callers   |

### Misc

| Feature             | Description                                      |
|---------------------|--------------------------------------------------|
| `logging`           | Debug logging macros                             |
| `keyboard-scancode` | Raw PS/2 scancode keyboard support               |
| `snapshot`          | Terminal state snapshot (escape sequence export) |

## Build

### Desktop (Linux x86-64)

```bash
# Debug build with unifont
make run-example

# Release build with TrueType font support
make run-example-release
```

### no_std (Bare-metal / Embedded)

```bash
# Build the C library (no_std, FFI, unifont)
make lib

# This produces: target/x86_64-unknown-none/release/libshitty.a
```

### WebAssembly

```bash
# Debug build
make wasm-debug

# Release build
make wasm-release

# Serve locally
make web
```

### Cross-compilation

```bash
# Linux cross
make cross-linux

# Windows cross (MinGW)
make cross-windows
```

## Examples

| Directory              | Language | Platform  | Status                                                                     |
|------------------------|----------|-----------|----------------------------------------------------------------------------|
| `examples/c-x11/`      | C        | X11       | Working — full terminal with keyboard input, resize, framebuffer rendering |
| `examples/c-wayland/`  | C        | Wayland   | Skeleton — display connection only                                         |
| `examples/kotlin-ffm/` | Kotlin   | JVM FFM   | Working — Java Foreign Function & Memory API, no JNI needed                |
| `examples/macroquad/`  | Rust     | Macroquad | Planned                                                                    |
| `examples/stm32f1/`    | C        | STM32F1   | Planned                                                                    |

## Architecture

```
┌─────────────────────────────────────────────────┐
│                   Terminal                        │
│  ┌──────────┐  ┌──────────────────────────────┐  │
│  │  Parser   │  │      TerminalBuffer          │  │
│  │ (ANSI    │  │  ┌───────┐  ┌──────────────┐ │  │
│  │  escape) │  │  │Primary│  │  Alternate   │ │  │
│  │          │  │  │(hist.)│  │  (alt screen)│ │  │
│  └──────────┘  │  └───────┘  └──────────────┘ │  │
│                └──────────┬───────────────────┘  │
│                           │                       │
│                ┌──────────▼───────────────────┐  │
│                │         Screen                │  │
│                │  (dirty tracking, multi-buf)  │  │
│                └──────────┬───────────────────┘  │
│                           │                       │
│                ┌──────────▼───────────────────┐  │
│                │         Drawable              │  │
│                │  (pixel buffer output)        │  │
│                └──────────────────────────────┘  │
│                                                   │
│  Callbacks: PTY Write ─ Clipboard ─ Bell ─ Title │
└─────────────────────────────────────────────────┘
```

All components are modular and swappable via feature flags.

## WebAssembly

The WASM target provides a `shitty` global object with functions for creating terminals, processing input, flushing to a pixel buffer, and setting callbacks.

Build with `make wasm-release`, then open `www/index.html` in a browser.

The WASM API is declared in `www/pkg/shitty.d.ts` and includes:

- `shitty_new` / `shitty_del` — Terminal lifecycle
- `shitty_process` / `shitty_process_byte` / `shitty_process_char` — Feed program output
- `shitty_input` — Send user input to the child process
- `shitty_flush` — Render to a pixel buffer
- `shitty_set_callback_*` — Set JS callbacks for PTY write, clipboard, bell, title

## FFI (C ABI)

The library exposes a C ABI for non-Rust consumers. Enable with `ffi` feature:

```bash
cargo build --features ffi-default,font-unifont
```

The C API includes functions prefixed with `shitty_*` and constants prefixed with `SHITTY_*`. See `examples/c-x11/` for a complete working example.

When building without `ffi-std`, the caller must provide allocator callbacks:

- `shitty_callback_alloc`
- `shitty_callback_free`
- `shitty_callback_aligned_alloc`
- `shitty_callback_aligned_free` (or `realloc`)

## Embedded / no_std

The library can run without the Rust standard library. Use:

```bash
cargo build -Z build-std=core,alloc \
  --target <target> \
  --no-default-features \
  --features ffi-default,font-unifont
```

Supported targets include `x86_64-unknown-none`, `wasm32-unknown-unknown`, and any ARM Cortex-M target.

## Testing

```bash
# Run all unit and integration tests
cargo test

# Run CI build matrix (includes cross-compilation checks)
make ci-test

# Fuzz testing
make fuzzing
```

## Documentation

Crate-level documentation is available via `cargo doc --open` or on the [module-level docs](src/lib.rs).

## Contributing

Contributions are welcome! Let's make a shitty terminal emulator together!

## License

MIT OR Apache-2.0
