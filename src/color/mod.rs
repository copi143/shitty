//! 颜色空间和像素格式。支持多种颜色类型和通道顺序，通过 feature 切换默认格式。
//!
//! 核心类型：
//! - [`Color`]：8-bit sRGB 32 位真彩色（通道顺序由 feature 决定）
//! - [`AnsiColor`] / [`AnsiNamedColor`]：ANSI 颜色索引/命名颜色
//! - [`ColorScheme`]：512 色调色板（16 ANSI + 216 立方体 + 24 灰度 + 特殊色）
//! - [`WrappedDrawable`] / [`WrappedScreen`]：运行时多态颜色类型包装
//!
//! ---
//!
//! Color spaces and pixel formats. Supports multiple color types and channel orders.
//!
//! Core types:
//! - [`Color`]: 8-bit sRGB 32-bit true color (channel order determined by feature)
//! - [`AnsiColor`] / [`AnsiNamedColor`]: ANSI color index / named color
//! - [`ColorScheme`]: 512-color palette (16 ANSI + 216 cube + 24 grayscale + special)
//! - [`WrappedDrawable`] / [`WrappedScreen`]: Runtime polymorphic color type wrappers

#![allow(dead_code)]
#![allow(unused_imports)]

use core::fmt::Debug;
use core::hash::Hash;
use core::ops::{Add, Div, Mul, Rem, Shl, Shr, Sub};

/// 实现与 `Color` 互相转换的颜色类型。
///
/// 必须固定大小且实现 `Debug`、`Default`、`Copy`。
///
/// 比较和哈希不是必须的。
///
/// ---
///
/// Color types that can be converted to and from `Color`.
///
/// Must be of fixed size and implement `Debug`, `Default`, and `Copy`.
///
/// comparison and hashing are not required.
pub trait IColor: Sized + Debug + const Default + Copy + const From<Color> + const Into<Color> {}

impl<T: Sized + Debug + const Default + Copy + const From<Color> + const Into<Color>> IColor for T {}

pub mod ansi;
pub use ansi::*;

pub mod format;
pub use format::*;

pub mod scheme;
pub use scheme::ColorScheme;

macro_rules! wrapped {
    (# $variant:ident) => {
        $variant
    };
    (# $variant:ident : $ty:ty) => {
        $ty
    };
    ($($variant:ident $(: $ty:ty)?),+ $(,)?) => {
        #[allow(non_camel_case_types)]
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum ColorFormat {
            #[default]
            $($variant,)+
        }
        impl TryFrom<usize> for ColorFormat {
            type Error = ();
            fn try_from(value: usize) -> Result<Self, Self::Error> {
                match value {
                    $(x if x == Self::$variant as usize => Ok(Self::$variant),)+
                    _ => Err(()),
                }
            }
        }
        #[expect(unsafe_code)]
        impl ColorFormat {
            /// ### Safety
            ///
            /// The caller must ensure that the `buf` pointer is valid and points to a buffer of the correct size and format for the given `ColorFormat`.
            pub unsafe fn dispatch_flush(
                self,
                terminal: &mut crate::Terminal,
                buf: *mut core::ffi::c_void,
                width: usize,
                height: usize,
                pitch: usize,
                time: core::time::Duration,
            ) {
                match self {
                    $(Self::$variant => {
                        let mut d = unsafe {
                            crate::buffer::Drawable::from_raw_parts(
                                buf as *mut wrapped!(# $variant $(: $ty)?),
                                width,
                                height,
                                pitch,
                            )
                        };
                        terminal.flush(&mut d, time);
                    })+
                }
            }
        }
        #[allow(non_camel_case_types)]
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Debug)]
        pub enum WrappedDrawable<'buf> {
            $($variant(&'buf mut crate::buffer::Drawable<'buf, wrapped!(# $variant $(: $ty )?)>),)+
        }
        $(
            impl<'buf> From<&'buf mut crate::Drawable<'buf, wrapped!(# $variant $(: $ty )?)>> for WrappedDrawable<'buf> {
                fn from(drawable: &'buf mut crate::Drawable<'buf, wrapped!(# $variant $(: $ty )?)>) -> Self {
                    WrappedDrawable::$variant(drawable)
                }
            }
        )+
        #[allow(non_camel_case_types)]
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Debug, Default)]
        pub enum WrappedScreen {
            #[default]
            None,
            $($variant(crate::buffer::Screen<wrapped!(# $variant $(: $ty )?)>),)+
        }
        $(
            impl From<crate::buffer::Screen<wrapped!(# $variant $(: $ty )?)>> for WrappedScreen {
                fn from(screen: crate::buffer::Screen<wrapped!(# $variant $(: $ty )?)>) -> Self {
                    WrappedScreen::$variant(screen)
                }
            }
        )+
        impl WrappedScreen {
            /// 判断当前屏幕和给定的 Drawable 是否属于同一变体（即颜色类型相同）。
            ///
            /// This method checks if the current screen and the given Drawable belong to the same variant (i.e., have the same color type).
            pub fn is_same_variant(&self, drawable: &WrappedDrawable) -> bool {
                matches!(
                    (self, drawable),
                    $((WrappedScreen::$variant(_), WrappedDrawable::$variant(_)))|+
                )
            }
        }
        macro_rules! screen_dispatch {
            ($scr:expr, $cb:expr) => {
                screen_dispatch!($scr, $cb, ())
            };
            ($scr:expr, $cb:expr, $default:expr) => {{
                match &mut $scr.wrapped {
                    WrappedScreen::None => $default,
                    $(WrappedScreen::$variant(screen) => ($cb)(screen),)+
                }
            }};
        }
        macro_rules! screen_drawable_dispatch {
            ($scr:expr, $dr:expr, $cb:expr, $default:expr) => {{
                match (&mut $scr.wrapped, $dr) {
                    $(
                        (WrappedScreen::$variant(screen), WrappedDrawable::$variant(drawable)) => ($cb)(screen, drawable),
                    )+
                    _ => $default,
                }
            }};
        }
        macro_rules! drawable_dispatch_new_screen {
            ($dr:expr, $width:expr, $height:expr, $bufmode:expr) => {{
                match $dr {
                    $(WrappedDrawable::$variant(_) => WrappedScreen::$variant(Screen::new($width, $height, $bufmode)),)+
                }
            }};
        }
    };
}

#[cfg(not(feature = "color-formats"))]
wrapped! { Default: Color }

// INFO 添加一种新的颜色类型只需要在此处添加一个新的变体
#[cfg(feature = "color-formats")]
wrapped! {
    Default: Color,
    RGBA,
    BGRA,
    ARGB,
    ABGR,
    RGBA16,
    BGRA16,
    ARGB16,
    ABGR16,
    RGB,
    BGR,
    RGBA_F32,
    BGRA_F32,
    ARGB_F32,
    ABGR_F32,
    RGB_F32,
    BGR_F32,
    RGB565,
    BGR565,
}
