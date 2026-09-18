use alloc::vec::Vec;
use core::fmt::Debug;
use core::panic;

use crate::buffer::{Cell, CharFlags};
use crate::color::{Color, ColorI16, IColor};
use crate::font::{FontBuffer, RenderResultRef};
use crate::helper::{likely, unlikely};

/// 一个表示可绘制表面的结构，用于渲染终端输出。
/// - 当缓冲开启时，相同地址和大小的 buf 会被认为数据没有被外部修改，会被跳过渲染以提高性能。
///
/// ---
///
/// A structure representing a drawable surface for rendering terminal output.
/// - When buffering is enabled, the same address and size of the buffer will be considered unchanged by external modifications and will be skipped for rendering to improve performance.
///
/// ---
///
/// Example usage, from softbuffer surface:
///
/// ```rust,ignore
/// let mut buffer = surface.buffer_mut().unwrap();
/// let mut drawable = Drawable::from_softbuffer(&mut buffer);
/// terminal.flush(&mut drawable);
/// buffer.present().unwrap();
/// ```
///
/// Example usage, from slice:
///
/// ```rust,ignore
/// let mut buffer = vec![0u32; surface_width * surface_height];
/// let mut drawable = Drawable::new(
///     Color::from_u32_mut_slice(&mut buffer),
///     surface_width,
///     surface_height,
///     0,
/// );
/// terminal.flush(&mut drawable);
/// ```
///
/// Example usage, from raw parts:
///
/// ```rust,ignore
/// let mut buffer = vec![0u32; surface_width * surface_height];
/// let mut drawable = unsafe {
///    Drawable::from_raw_parts(
///       buffer.as_mut_ptr() as *mut Color,
///       surface_width,
///       surface_height,
///       0,
///    )
/// };
/// terminal.flush(&mut drawable);
/// ```
pub struct Drawable<'buf, T: IColor = Color> {
    pub buf: &'buf mut [T],
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
}

impl<'buf, T: IColor> Debug for Drawable<'buf, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Drawable")
            .field("addr", &(self.buf.as_ptr() as usize))
            .field("len", &self.buf.len())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pitch", &self.pitch)
            .finish()
    }
}

#[allow(dead_code)]
impl<'buf> Drawable<'buf> {
    const ALPHA_MAX: u16 = 255;
    const BLEND_ROUNDING_OFFSET: u16 = 127;

    #[inline(always)]
    fn blend_pixel(src: Color, dst: Color) -> Color {
        let sa = src.a as u16;
        if sa == 0 {
            return dst;
        }
        if sa == Self::ALPHA_MAX {
            return src;
        }
        let inv = Self::ALPHA_MAX - sa;
        let r = ((src.r as u16 * sa + dst.r as u16 * inv + Self::BLEND_ROUNDING_OFFSET) / Self::ALPHA_MAX) as u8;
        let g = ((src.g as u16 * sa + dst.g as u16 * inv + Self::BLEND_ROUNDING_OFFSET) / Self::ALPHA_MAX) as u8;
        let b = ((src.b as u16 * sa + dst.b as u16 * inv + Self::BLEND_ROUNDING_OFFSET) / Self::ALPHA_MAX) as u8;
        let a = (sa + (dst.a as u16 * inv + Self::BLEND_ROUNDING_OFFSET) / Self::ALPHA_MAX).min(Self::ALPHA_MAX) as u8;
        Color::rgba(r, g, b, a)
    }

    #[inline(always)]
    #[expect(unsafe_code)]
    fn set_index_blend(&mut self, index: usize, color: Color) {
        debug_assert!(index < self.buf.len(), "Index out of bounds");
        if likely(color.a == 255) {
            unsafe { *self.buf.get_unchecked_mut(index) = color };
            return;
        }
        unsafe {
            let old = *self.buf.get_unchecked(index);
            *self.buf.get_unchecked_mut(index) = Self::blend_pixel(color, old);
        }
    }
}

#[cfg(feature = "softbuffer")]
impl<'buf> Drawable<'buf, crate::BGRA> {
    pub fn from_softbuffer<D: raw_window_handle::HasDisplayHandle, W: raw_window_handle::HasWindowHandle>(
        buffer: &'buf mut softbuffer::Buffer<D, W>,
    ) -> Self {
        let width = buffer.width().get() as usize;
        let height = buffer.height().get() as usize;
        let buf = crate::BGRA::from_u32_mut_slice(buffer.as_mut());
        Self::new(buf, width, height, 0)
    }
}

#[expect(unsafe_code)]
impl<'buf, T: IColor> Drawable<'buf, T> {
    /// Create a new Drawable with the given buffer, width, height, and pitch.
    /// - `buf` should be a mutable slice of colors representing the pixel data.
    /// - `width` is the width of the drawable area in pixels.
    /// - `height` is the height of the drawable area in pixels.
    /// - `pitch` is the number of pixels per row in the buffer (may be greater than `width`).
    /// - If `pitch` is set to 0, it will be treated as equal to `width`.
    /// - The buffer is expected to be in row-major order.
    /// - The caller is responsible for ensuring the buffer lives long enough.
    pub fn new(buf: &'buf mut [T], width: usize, height: usize, pitch: usize) -> Self {
        assert!(pitch == 0 || pitch >= width, "Pitch must be greater than or equal to width");
        assert!(width > 0 && height > 0, "Width and height must be greater than 0");
        assert!(buf.len() >= height * pitch, "Buffer size is smaller than height * pitch");
        let pitch = if pitch == 0 { width } else { pitch };
        Self {
            buf,
            width,
            height,
            pitch,
        }
    }

    /// Create a Drawable from raw parts.
    /// - `buf` is a raw pointer to the pixel data.
    /// - `width` is the width of the drawable area in pixels.
    /// - `height` is the height of the drawable area in pixels.
    /// - `pitch` is the number of pixels per row in the buffer (may be greater than `width`).
    /// - If `pitch` is set to 0, it will be treated as equal to `width`.
    /// - The buffer is expected to be in row-major order.
    /// - The caller is responsible for ensuring the buffer lives long enough.
    ///
    /// #### Safety
    /// - The caller must ensure that `buf` is valid for reads and writes for `height * pitch` elements of type `T`.
    /// - The caller must ensure that the buffer is properly aligned for type `T`.
    /// - The caller must ensure that the buffer lives long enough and is not mutated by other code while the Drawable is in use.
    pub unsafe fn from_raw_parts(buf: *mut T, width: usize, height: usize, pitch: usize) -> Self {
        assert!(pitch == 0 || pitch >= width, "Pitch must be greater than or equal to width");
        assert!(width > 0 && height > 0, "Width and height must be greater than 0");
        let pitch = if pitch == 0 { width } else { pitch };
        let len = height * pitch;
        let buf = unsafe { core::slice::from_raw_parts_mut(buf, len) };
        Self {
            buf,
            width,
            height,
            pitch,
        }
    }

    /// Get the color at the specified (x, y) position.
    ///
    /// ***Warning**: Check for overflows only in debug mode!!!*
    #[inline(always)]
    pub fn get(&self, x: usize, y: usize) -> T {
        debug_assert!(x < self.width, "X coordinate out of bounds");
        debug_assert!(y < self.height, "Y coordinate out of bounds");
        unsafe { *self.buf.get_unchecked(y * self.pitch + x) }
    }

    /// Set the color at the specified (x, y) position.
    ///
    /// ***Warning**: Check for overflows only in debug mode!!!*
    #[inline(always)]
    pub fn set(&mut self, x: usize, y: usize, color: T) {
        debug_assert!(x < self.width, "X coordinate out of bounds");
        debug_assert!(y < self.height, "Y coordinate out of bounds");
        unsafe { *self.buf.get_unchecked_mut(y * self.pitch + x) = color };
    }

    /// Set the color at the specified index in the buffer.
    ///
    /// ***Warning**: Check for overflows only in debug mode!!!*
    #[inline(always)]
    pub fn set_index(&mut self, index: usize, color: T) {
        debug_assert!(index < self.buf.len(), "Index out of bounds");
        unsafe { *self.buf.get_unchecked_mut(index) = color };
    }

    /// Clear the entire drawable surface with the specified color.
    #[inline(always)]
    pub fn clear(&mut self, color: T) {
        self.buf.fill(color);
    }

    pub(crate) fn write(&mut self, font: &FontBuffer, x: u32, y: u32, cell: &Cell<T>) {
        let (drawable_width, drawable_height) = (self.width, self.height);
        let (font_width, font_height) = (font.width as usize, font.height as usize);
        let (x_start, y_start) = (x as usize * font_width, y as usize * font_height);
        debug_assert!(font_width > 0 && font_height > 0, "Font width and height must be greater than 0");
        debug_assert!(x_start + font_width <= drawable_width, "X coordinate out of bounds");
        debug_assert!(y_start + font_height <= drawable_height, "Y coordinate out of bounds");

        match cell.rend_ref() {
            RenderResultRef::Empty => panic!("Cannot render empty cell"),
            RenderResultRef::Blank => {
                let mut idx = y_start * self.pitch + x_start;
                let max = idx + font_height * self.pitch;
                while likely(idx < max) {
                    for dx in 0..font_width {
                        self.set_index(idx + dx, cell.bg);
                    }
                    idx += self.pitch;
                }
            }
            RenderResultRef::Subpix(glyph) => {
                let fg = ColorI16::from(cell.fg.into());
                let bg = ColorI16::from(cell.bg.into());
                let a = fg - bg;
                let b = bg * 255 + 255;
                let mut idx = y_start * self.pitch + x_start;
                let max = idx + glyph.height as usize * self.pitch;
                let mut i = 0;
                while likely(idx < max) {
                    for dx in 0..glyph.width as usize {
                        let k = ColorI16::from(glyph.at_index(i));
                        self.set_index(idx + dx, T::from(((k * a + b) >> 8).into()));
                        i += 1;
                    }
                    idx += self.pitch;
                }
            }
            RenderResultRef::Colored(glyph) => {
                let mut idx = y_start * self.pitch + x_start;
                let max = idx + glyph.height as usize * self.pitch;
                let mut i = 0;
                while likely(idx < max) {
                    for dx in 0..glyph.width as usize {
                        self.set_index(idx + dx, T::from(glyph.buf[i]));
                        i += 1;
                    }
                    idx += self.pitch;
                }
            }
        };

        if likely(cell.flags.is_empty()) {
            return;
        }

        if cell.flags.contains(CharFlags::UNDERLINE) {
            let y_base = y_start + font_height - 1;
            (0..font_width).for_each(|x| self.set(x_start + x, y_base, cell.fg));
        }

        if cell.flags.contains(CharFlags::STRIKETHROUGH) {
            let y_base = y_start + font_height / 2;
            (0..font_width).for_each(|x| self.set(x_start + x, y_base, cell.fg));
        }
    }

    #[inline(always)]
    pub fn map(&mut self, mut f: impl FnMut(T) -> T) {
        for pixel in self.buf.iter_mut() {
            *pixel = f(*pixel);
        }
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, &T)> {
        self.buf.iter().enumerate().map(|(i, color)| (i % self.width, i / self.width, color))
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, usize, &mut T)> {
        self.buf.iter_mut().enumerate().map(|(i, color)| (i % self.width, i / self.width, color))
    }
}

/// A drawable that owns its pixel buffer.
///
/// 用于内部分配并拥有像素缓冲区的结构，接口与 `Drawable` 相似，方便在需要自管理内存时使用。
pub struct OwnedDrawable {
    pub buf: Vec<Color>,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
}

impl<'buf> From<&'buf mut OwnedDrawable> for Drawable<'buf> {
    fn from(owned: &'buf mut OwnedDrawable) -> Self {
        Drawable::new(owned.buf.as_mut_slice(), owned.width, owned.height, owned.pitch)
    }
}

impl OwnedDrawable {
    /// Create a new OwnedDrawable with internally allocated buffer.
    pub fn with_capacity(width: usize, height: usize, pitch: usize) -> Self {
        assert!(pitch == 0 || pitch >= width, "Pitch must be greater than or equal to width");
        assert!(width > 0 && height > 0, "Width and height must be greater than 0");
        let pitch = if pitch == 0 { width } else { pitch };
        let len = height * pitch;
        Self {
            buf: vec![Color::default(); len],
            width,
            height,
            pitch,
        }
    }

    /// Create from an existing Vec buffer. Ensures buffer is large enough.
    pub fn from_vec(buf: Vec<Color>, width: usize, height: usize, pitch: usize) -> Self {
        let pitch = if pitch == 0 { width } else { pitch };
        assert!(pitch >= width, "Pitch must be greater than or equal to width");
        assert!(width > 0 && height > 0, "Width and height must be greater than 0");
        assert!(buf.len() >= height * pitch, "Buffer size is smaller than height * pitch");
        Self {
            buf,
            width,
            height,
            pitch,
        }
    }

    /// Consume and return the inner Vec buffer.
    pub fn into_vec(self) -> Vec<Color> {
        self.buf
    }

    #[inline(always)]
    #[expect(unsafe_code)]
    pub fn get(&self, x: usize, y: usize) -> Color {
        debug_assert!(x < self.width, "X coordinate out of bounds");
        debug_assert!(y < self.height, "Y coordinate out of bounds");
        unsafe { *self.buf.get_unchecked(y * self.pitch + x) }
    }

    #[inline(always)]
    #[expect(unsafe_code)]
    pub fn set(&mut self, x: usize, y: usize, color: Color) {
        debug_assert!(x < self.width, "X coordinate out of bounds");
        debug_assert!(y < self.height, "Y coordinate out of bounds");
        unsafe { *self.buf.get_unchecked_mut(y * self.pitch + x) = color };
    }

    #[inline(always)]
    #[expect(unsafe_code)]
    pub fn set_index(&mut self, index: usize, color: Color) {
        debug_assert!(index < self.buf.len(), "Index out of bounds");
        unsafe { *self.buf.get_unchecked_mut(index) = color };
    }

    #[inline(always)]
    pub fn clear(&mut self, color: Color) {
        self.buf.fill(color);
    }

    #[inline(always)]
    pub fn map(&mut self, f: impl FnMut(Color) -> Color) {
        Drawable::from(self).map(f);
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, &Color)> + use<'_> {
        self.buf.iter().enumerate().map(|(i, color)| (i % self.width, i / self.width, color))
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, usize, &mut Color)> + use<'_> {
        self.buf.iter_mut().enumerate().map(|(i, color)| (i % self.width, i / self.width, color))
    }
}
