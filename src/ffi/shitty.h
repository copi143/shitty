#pragma once

#define SHITTY_FEATURE_UNIFONT 1

#ifndef SHITTY_FEATURE_UNIFONT
#define SHITTY_FEATURE_UNIFONT 0
#endif

#ifndef SHITTY_FEATURE_BITMAP
#define SHITTY_FEATURE_BITMAP 0
#endif

#ifndef SHITTY_FEATURE_TRUETYPE
#define SHITTY_FEATURE_TRUETYPE 0
#endif

#ifndef SHITTY_FEATURE_COLOR_FORMATS
#define SHITTY_FEATURE_COLOR_FORMATS 0
#endif

#ifndef SHITTY_NO_LIBC
#define SHITTY_NO_LIBC 0
#endif

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //

#if !defined(__cplusplus) && __STDC_VERSION__ < 202311L
typedef _Bool shitty_bool;
#else
typedef bool shitty_bool;
#endif

typedef __INT8_TYPE__ shitty_i8;
typedef __INT16_TYPE__ shitty_i16;
typedef __INT32_TYPE__ shitty_i32;
typedef __INT64_TYPE__ shitty_i64;

typedef __UINT8_TYPE__ shitty_u8;
typedef __UINT16_TYPE__ shitty_u16;
typedef __UINT32_TYPE__ shitty_u32;
typedef __UINT64_TYPE__ shitty_u64;

typedef __PTRDIFF_TYPE__ shitty_isize;
typedef __SIZE_TYPE__ shitty_usize;

typedef const char *shitty_str;

typedef void (*shitty_callback_pty_write)(shitty_str data, shitty_usize len);
typedef shitty_str (*shitty_callback_clipboard_get)();
typedef void (*shitty_callback_clipboard_set)(shitty_str data,
                                              shitty_usize len);
typedef void (*shitty_callback_bell)();
typedef void (*shitty_callback_title_set)(shitty_str title, shitty_usize len);

typedef struct shitty_terminal_struct *shitty_terminal;

typedef struct shitty_fontrenderer_struct *shitty_fontrenderer;

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //

// ++ shitty::buffer::screen::BufMode ++ //

/// Generate complete frame every time
extern const shitty_usize SHITTY_BUFMODE_NONE;
/// Only redraw needed parts - single buffer mode
extern const shitty_usize SHITTY_BUFMODE_SINGLE;
/// Only redraw needed parts - double buffer mode
extern const shitty_usize SHITTY_BUFMODE_DOUBLE;
/// Only redraw needed parts - triple buffer mode
extern const shitty_usize SHITTY_BUFMODE_TRIPLE;

// ++ shitty::cursor_shape::CursorShape ++ //

/// Cursor is a block like `▒`.
extern const shitty_usize SHITTY_CURSOR_SHAPE_BLOCK;
/// Cursor is an underscore like `_`.
extern const shitty_usize SHITTY_CURSOR_SHAPE_UNDERLINE;
/// Cursor is a vertical bar `⎸`.
extern const shitty_usize SHITTY_CURSOR_SHAPE_BEAM;
/// Cursor is a box like `☐`.
extern const shitty_usize SHITTY_CURSOR_SHAPE_HOLLOW_BLOCK;
/// Invisible cursor.
extern const shitty_usize SHITTY_CURSOR_SHAPE_HIDDEN;

// ++ shitty::MouseButton ++ //

extern const shitty_usize SHITTY_MOUSE_BUTTON_LEFT;
extern const shitty_usize SHITTY_MOUSE_BUTTON_RIGHT;
extern const shitty_usize SHITTY_MOUSE_BUTTON_MIDDLE;
extern const shitty_usize SHITTY_MOUSE_BUTTON_X1;
extern const shitty_usize SHITTY_MOUSE_BUTTON_X2;

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //
// ----- Shitty Requested Callbacks                                  ----- //

/**
 *\brief Panic callback
 *\note This callback is optional.
 *      If not provided, the panic handler will enter an infinite loop.
 *
 *\param info     Panic information
 */
void shitty_callback_panic(shitty_str info);

/**
 *\brief Memory allocation callback
 *
 *\param size    The size of the memory to allocate
 *\return A pointer to the allocated memory, or NULL on failure
 */
void *shitty_callback_alloc(shitty_usize size);

/**
 *\brief Memory deallocation callback
 *
 *\param ptr    A pointer to the memory to deallocate
 */
void shitty_callback_free(void *ptr);

/**
 *\brief Aligned memory allocation callback
 *
 *\param size   The size of the memory to allocate
 *\param align  The alignment of the memory to allocate
 *\return A pointer to the allocated memory, or NULL on failure
 */
void *shitty_callback_aligned_alloc(shitty_usize size, shitty_usize align);

/**
 *\brief Memory reallocation callback
 *\note This callback is optional.
 *      If not provided, the reallocation handler will fall back to an
 *      implementation using alloc and free.
 *
 *\param ptr      A pointer to the memory to reallocate
 *\param size     The new size of the memory
 *\return A pointer to the reallocated memory, or NULL on failure
 */
void *shitty_callback_realloc(void *ptr, shitty_usize size);

/**
 *\brief Aligned memory reallocation callback
 *\note This callback is optional.
 *      If not provided, the reallocation handler will fall back to an
 *      implementation using aligned_alloc and free.
 *
 *\param ptr      A pointer to the memory to reallocate
 *\param size     The new size of the memory
 *\param align    The alignment of the memory to reallocate
 *\return A pointer to the reallocated memory, or NULL on failure
 */
void *shitty_callback_aligned_realloc(void *ptr, shitty_usize size,
                                      shitty_usize align);

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //

/**
 *\brief Create a new terminal instance.
 *
 *\param width    The width of the terminal.
 *\param height   The height of the terminal.
 *\param bufmode  The buffer mode of the terminal.
 *\param font     A pointer to the font renderer to use.
 *\return A pointer to the new terminal instance.
 */
shitty_terminal shitty_new(shitty_u32 width, shitty_u32 height,
                           shitty_usize bufmode, shitty_fontrenderer font);

/**
 *\brief Delete a terminal instance.
 *
 *\param terminal A pointer to the terminal instance to delete.
 */
void shitty_del(shitty_terminal terminal);

/**
 *\brief Resize the terminal.
 *
 *\param terminal A pointer to the terminal instance.
 *\param width    The new width.
 *\param height   The new height.
 */
void shitty_resize(shitty_terminal terminal, shitty_u32 width,
                   shitty_u32 height);

/**
 *\brief Check if the terminal is dirty.
 *
 *\param terminal A pointer to the terminal instance.
 *\return true if dirty, false otherwise.
 */
shitty_bool shitty_isdirty(shitty_terminal terminal);

/**
 *\brief Get the number of rows in the terminal.
 *
 *\param terminal A pointer to the terminal instance.
 *\return The number of rows.
 */
shitty_u32 shitty_rows(shitty_terminal terminal);

/**
 *\brief Get the number of columns in the terminal.
 *
 *\param terminal A pointer to the terminal instance.
 *\return The number of columns.
 */
shitty_u32 shitty_cols(shitty_terminal terminal);

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //

void shitty_set_callback_ptywrite(shitty_terminal terminal,
                                  shitty_callback_pty_write callback);

void shitty_set_callback_clipboard(shitty_terminal terminal,
                                   shitty_callback_clipboard_get get,
                                   shitty_callback_clipboard_set set);

void shitty_set_callback_bell(shitty_terminal terminal,
                              shitty_callback_bell callback);

void shitty_set_callback_title(shitty_terminal terminal,
                               shitty_callback_title_set callback);

// ++ shitty::ColorFormat ++ //

/// Default color format (platform RGBA).
extern const shitty_usize SHITTY_COLORFMT_DEFAULT;

#ifdef SHITTY_FEATURE_COLOR_FORMATS
/// 32-bit RGBA pixel format.
extern const shitty_usize SHITTY_COLORFMT_RGBA;
/// 32-bit BGRA pixel format.
extern const shitty_usize SHITTY_COLORFMT_BGRA;
/// 32-bit ARGB pixel format.
extern const shitty_usize SHITTY_COLORFMT_ARGB;
/// 32-bit ABGR pixel format.
extern const shitty_usize SHITTY_COLORFMT_ABGR;
/// 64-bit RGBA pixel format (16 bits per channel).
extern const shitty_usize SHITTY_COLORFMT_RGBA16;
/// 64-bit BGRA pixel format (16 bits per channel).
extern const shitty_usize SHITTY_COLORFMT_BGRA16;
/// 64-bit ARGB pixel format (16 bits per channel).
extern const shitty_usize SHITTY_COLORFMT_ARGB16;
/// 64-bit ABGR pixel format (16 bits per channel).
extern const shitty_usize SHITTY_COLORFMT_ABGR16;
/// 24-bit RGB pixel format.
extern const shitty_usize SHITTY_COLORFMT_RGB;
/// 24-bit BGR pixel format.
extern const shitty_usize SHITTY_COLORFMT_BGR;
/// 128-bit RGBA pixel format (f32 per channel).
extern const shitty_usize SHITTY_COLORFMT_RGBA_F32;
/// 128-bit BGRA pixel format (f32 per channel).
extern const shitty_usize SHITTY_COLORFMT_BGRA_F32;
/// 128-bit ARGB pixel format (f32 per channel).
extern const shitty_usize SHITTY_COLORFMT_ARGB_F32;
/// 128-bit ABGR pixel format (f32 per channel).
extern const shitty_usize SHITTY_COLORFMT_ABGR_F32;
/// 96-bit RGB pixel format (f32 per channel).
extern const shitty_usize SHITTY_COLORFMT_RGB_F32;
/// 96-bit BGR pixel format (f32 per channel).
extern const shitty_usize SHITTY_COLORFMT_BGR_F32;
/// 16-bit RGB pixel format (5-6-5).
extern const shitty_usize SHITTY_COLORFMT_RGB565;
/// 16-bit BGR pixel format (5-6-5).
extern const shitty_usize SHITTY_COLORFMT_BGR565;
#endif

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //

/**
 *\brief Flush the terminal buffer to an output buffer.
 *
 *\param terminal A pointer to the terminal instance.
 *\param width    The width of the output buffer.
 *\param height   The height of the output buffer.
 *\param pitch    The pitch (stride) of the output buffer.
 *\param buf      A pointer to the output buffer.
 *\param time_ms  The current time in milliseconds.
 *\param color    A `ColorFormat` discriminant (see `SHITTY_COLORFMT_*`
 * constants).
 */
void shitty_flush(shitty_terminal terminal, shitty_usize width,
                  shitty_usize height, shitty_usize pitch, void *buf,
                  shitty_u64 time_ms, shitty_usize color);

void shitty_process(shitty_terminal terminal, const shitty_u8 *input,
                    shitty_usize len);

void shitty_input(shitty_terminal terminal, const shitty_u8 *input,
                  shitty_usize len);

void shitty_handle_mouse_move(shitty_terminal terminal, shitty_i32 x,
                              shitty_i32 y);
void shitty_handle_mouse_press(shitty_terminal terminal, shitty_usize button);
void shitty_handle_mouse_release(shitty_terminal terminal, shitty_usize button);
void shitty_handle_mouse_scroll(shitty_terminal terminal, shitty_i32 lines);
void shitty_handle_mouse_scroll_xy(shitty_terminal terminal, shitty_i32 dx,
                                   shitty_i32 dy);
void shitty_handle_modifiers(shitty_terminal terminal, shitty_u8 modifiers);
void shitty_handle_mouse_leave(shitty_terminal terminal);
void shitty_handle_focus(shitty_terminal terminal, shitty_bool gained);

#ifdef SHITTY_FEATURE_KEYBOARD_SCANCODE
/**
 *\brief Handle a keyboard scancode input for the terminal.
 *
 *\param terminal A pointer to the terminal instance.
 *\param scancode The keyboard scancode.
 */
void shitty_handle_keyboard_scancode(shitty_terminal terminal,
                                     shitty_u8 scancode);
#endif

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //
#ifdef SHITTY_FEATURE_UNIFONT

/**
 *\brief Create a new Unifont instance.
 *
 *\return A pointer to the new Unifont instance.
 */
shitty_fontrenderer shitty_unifont_new();

/**
 *\brief Delete a Unifont instance.
 *
 *\param font A pointer to the Unifont instance to delete.
 */
void shitty_unifont_del(shitty_fontrenderer font);

#endif
// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //
#ifdef SHITTY_FEATURE_TRUETYPE

/**
 *\brief Create a new TrueType font instance.
 *\note Ownership of font_bytes is transferred to the shitty_truetypefont
 * instance.
 *
 *\param font_size The size of the font.
 *\param font_bytes A pointer to the font data.
 *\param font_bytes_len The length of the font data.
 *\return A pointer to the new TrueType font instance.
 */
shitty_fontrenderer shitty_truetypefont_new(shitty_i32 font_size,
                                            const void *font_bytes,
                                            shitty_usize font_bytes_len);

/**
 *\brief Delete a TrueType font instance.
 *
 *\param font A pointer to the TrueType font instance to delete.
 */
void shitty_truetypefont_del(shitty_fontrenderer font);

#endif
// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //
#ifdef SHITTY_FEATURE_BITMAP

/**
 *\brief Create a new Bitmap font instance.
 *
 *\return A pointer to the new Bitmap font instance.
 */
shitty_fontrenderer shitty_bitmapfont_new();

/**
 *\brief Delete a Bitmap font instance.
 *
 *\param font A pointer to the Bitmap font instance to delete.
 */
void shitty_bitmapfont_del(shitty_fontrenderer font);

#endif
