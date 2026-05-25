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

// $$$$$ ===== ===== ===== ===== ===== ===== ===== ===== ===== ===== $$$$$ //

/**
 *\brief Flush the terminal buffer to an output buffer.
 *
 *\param terminal A pointer to the terminal instance.
 *\param width    The width of the output buffer.
 *\param height   The height of the output buffer.
 *\param pitch    The pitch (stride) of the output buffer.
 *\param buf      A pointer to the output buffer.
 */
void shitty_flush(shitty_terminal terminal, shitty_usize width,
                  shitty_usize height, shitty_usize pitch, void *buf,
                  shitty_u64 time_ms);

void shitty_process(shitty_terminal terminal, const shitty_u8 *input,
                    shitty_usize len);

void shitty_input(shitty_terminal terminal, const shitty_u8 *input,
                  shitty_usize len);

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
