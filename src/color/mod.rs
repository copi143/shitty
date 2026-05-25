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

/// 用于内部存储颜色的类型，值恒为 0-255 范围内的整数，即使使用了更大范围的类型也是如此。
macro_rules! color {
    ($(#[$doc:meta])* Color, $type:ty, $align:literal) => {
        color!(*inner1 $(#[$doc])* Color, $type, $align);
    };
    ($(#[$doc:meta])* $name:ident, $type:ty, $align:literal) => {
        color!(*inner2 $(#[$doc])* $name, $type, $align);
    };
    (*inner1 $(#[$doc:meta])* $name:ident, $type:ty, $align:literal) => {
        $(#[$doc])*
        #[rustfmt::skip]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(C, align($align))]
        pub struct $name {
            #[cfg(any(feature = "default-argb", feature = "default-abgr"))] pub a: $type,
            #[cfg(any(feature = "default-rgba", feature = "default-argb"))] pub r: $type,
            #[cfg(any(feature = "default-bgra", feature = "default-abgr"))] pub b: $type,
                                                                            pub g: $type,
            #[cfg(any(feature = "default-bgra", feature = "default-abgr"))] pub r: $type,
            #[cfg(any(feature = "default-rgba", feature = "default-argb"))] pub b: $type,
            #[cfg(any(feature = "default-rgba", feature = "default-bgra"))] pub a: $type,
        }
        impl const Default for $name {
            fn default() -> Self {
                Self { r: 0, g: 0, b: 0, a: 255 }
            }
        }
        static_assert!(core::mem::size_of::<$name>() == core::mem::size_of::<$type>() * 4);
        impl $name {
            pub const BLACK: Self = Self::rgb(0, 0, 0);
            pub const WHITE: Self = Self::rgb(255, 255, 255);
            pub const fn default() -> Self {
                Self { r: 0, g: 0, b: 0, a: 255 }
            }
            pub const fn rgb(r: $type, g: $type, b: $type) -> Self {
                Self { r, g, b, a: 255 }
            }
            pub const fn rgba(r: $type, g: $type, b: $type, a: $type) -> Self {
                Self { r, g, b, a }
            }
            pub const fn gray(gray: $type) -> Self {
                Self { r: gray, g: gray, b: gray, a: 255 }
            }
            pub const fn gray_alpha(gray: $type, alpha: $type) -> Self {
                Self { r: gray, g: gray, b: gray, a: alpha }
            }
            /// Create a color from a 32-bit unsigned integer in 0xRRGGBB format.
            /// - No alpha component, alpha is set to 255.
            pub const fn from_readable_u32(c: u32) -> Self {
                Self::rgb((c >> 16) as _, (c >> 8) as _, c as _)
            }
            pub const fn as_readable_u32(&self) -> u32 {
                ((self.r as u8 as u32) << 16) | ((self.g as u8 as u32) << 8) | (self.b as u8 as u32)
            }
            pub const fn as_u32(&self) -> u32 {
                #[cfg(feature = "default-rgba")]
                return ((self.r as u8 as u32) << 0) | ((self.g as u8 as u32) << 8) | ((self.b as u8 as u32) << 16) | ((self.a as u8 as u32) << 24);
                #[cfg(feature = "default-bgra")]
                return ((self.b as u8 as u32) << 0) | ((self.g as u8 as u32) << 8) | ((self.r as u8 as u32) << 16) | ((self.a as u8 as u32) << 24);
                #[cfg(feature = "default-argb")]
                return ((self.a as u8 as u32) << 0) | ((self.r as u8 as u32) << 8) | ((self.g as u8 as u32) << 16) | ((self.b as u8 as u32) << 24);
                #[cfg(feature = "default-abgr")]
                return ((self.a as u8 as u32) << 0) | ((self.b as u8 as u32) << 8) | ((self.g as u8 as u32) << 16) | ((self.r as u8 as u32) << 24);
            }
        }
        impl const From<crate::color::AnsiRgb> for $name {
            fn from(rgb: crate::color::AnsiRgb) -> Self {
                Self::rgb(rgb.r as _, rgb.g as _, rgb.b as _)
            }
        }
    };
    (*inner2 $(#[$doc:meta])* $name:ident, $type:ty, $align:literal) => {
        color!(*inner1 $(#[$doc])* $name, $type, $align);
        impl From<Color> for $name {
            fn from(color: Color) -> Self {
                Self {
                    r: color.r as $type,
                    g: color.g as $type,
                    b: color.b as $type,
                    a: color.a as $type,
                }
            }
        }
        impl From<$name> for Color {
            fn from(color: $name) -> Self {
                Color {
                    r: (color.r % 256) as u8,
                    g: (color.g % 256) as u8,
                    b: (color.b % 256) as u8,
                    a: (color.a % 256) as u8,
                }
            }
        }
        impl Add<$type> for $name {
            type Output = Self;
            fn add(self, other: $type) -> Self {
                Self {
                    r: self.r.wrapping_add(other),
                    g: self.g.wrapping_add(other),
                    b: self.b.wrapping_add(other),
                    a: self.a.wrapping_add(other),
                }
            }
        }
        impl Add for $name {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                Self {
                    r: self.r.wrapping_add(other.r),
                    g: self.g.wrapping_add(other.g),
                    b: self.b.wrapping_add(other.b),
                    a: self.a.wrapping_add(other.a),
                }
            }
        }
        impl Sub<$type> for $name {
            type Output = Self;
            fn sub(self, other: $type) -> Self {
                Self {
                    r: self.r.wrapping_sub(other),
                    g: self.g.wrapping_sub(other),
                    b: self.b.wrapping_sub(other),
                    a: self.a.wrapping_sub(other),
                }
            }
        }
        impl Sub for $name {
            type Output = Self;
            fn sub(self, other: Self) -> Self {
                Self {
                    r: self.r.wrapping_sub(other.r),
                    g: self.g.wrapping_sub(other.g),
                    b: self.b.wrapping_sub(other.b),
                    a: self.a.wrapping_sub(other.a),
                }
            }
        }
        impl Mul<$type> for $name {
            type Output = Self;
            fn mul(self, other: $type) -> Self {
                Self {
                    r: self.r.wrapping_mul(other),
                    g: self.g.wrapping_mul(other),
                    b: self.b.wrapping_mul(other),
                    a: self.a.wrapping_mul(other),
                }
            }
        }
        impl Mul for $name {
            type Output = Self;
            fn mul(self, other: Self) -> Self {
                Self {
                    r: self.r.wrapping_mul(other.r),
                    g: self.g.wrapping_mul(other.g),
                    b: self.b.wrapping_mul(other.b),
                    a: self.a.wrapping_mul(other.a),
                }
            }
        }
        impl Shl<$type> for $name {
            type Output = Self;
            fn shl(self, shift: $type) -> Self {
                Self {
                    r: self.r.wrapping_shl(shift as u32),
                    g: self.g.wrapping_shl(shift as u32),
                    b: self.b.wrapping_shl(shift as u32),
                    a: self.a.wrapping_shl(shift as u32),
                }
            }
        }
        impl Shr<$type> for $name {
            type Output = Self;
            fn shr(self, shift: $type) -> Self {
                Self {
                    r: self.r.wrapping_shr(shift as u32),
                    g: self.g.wrapping_shr(shift as u32),
                    b: self.b.wrapping_shr(shift as u32),
                    a: self.a.wrapping_shr(shift as u32),
                }
            }
        }
        impl Div<$type> for $name {
            type Output = Self;
            fn div(self, other: $type) -> Self {
                Self {
                    r: self.r / other,
                    g: self.g / other,
                    b: self.b / other,
                    a: self.a / other,
                }
            }
        }
        impl Rem<$type> for $name {
            type Output = Self;
            fn rem(self, other: $type) -> Self {
                Self {
                    r: self.r % other,
                    g: self.g % other,
                    b: self.b % other,
                    a: self.a % other,
                }
            }
        }
    };
}

macro_rules! color4 {
    (
        #[derive($($derive_args:ident),*)]
        $(#[$doc:meta])* $name:ident, $type:ty, $align:literal;
        $min: literal, $max: literal, $map_fwd_args:tt => $map_fwd_body:expr, $map_bwd_args:tt => $map_bwd_body:expr;
        $($field:ident),*
    ) => {
        $(#[$doc])*
        #[allow(clippy::upper_case_acronyms)]
        #[derive($($derive_args),*)]
        #[repr(C, align($align))]
        pub struct $name {
            $(pub $field: $type,)*
        }
        static_assert!(core::mem::size_of::<$name>() == core::mem::size_of::<$type>() * 4);
        impl const Default for $name {
            fn default() -> Self {
                Self { r: $min, g: $min, b: $min, a: $max }
            }
        }
        #[allow(clippy::identity_op)]
        impl $name {
            const fn _map_fwd $map_fwd_args -> $type { $map_fwd_body }
            const fn _map_bwd $map_bwd_args -> u8 { $map_bwd_body }
        }
        #[allow(clippy::self_named_constructors)]
        impl $name {
            pub const fn default() -> Self {
                Self { r: $min, g: $min, b: $min, a: $max }
            }
            pub const fn rgb(r: $type, g: $type, b: $type) -> Self {
                Self { r, g, b, a: $max }
            }
            pub const fn rgba(r: $type, g: $type, b: $type, a: $type) -> Self {
                Self { r, g, b, a }
            }
            pub const fn gray(gray: $type) -> Self {
                Self { r: gray, g: gray, b: gray, a: $max }
            }
            pub const fn gray_alpha(gray: $type, alpha: $type) -> Self {
                Self { r: gray, g: gray, b: gray, a: alpha }
            }
        }
        impl const From<Color> for $name {
            fn from(color: Color) -> Self {
                Self::rgba(Self::_map_fwd(color.r), Self::_map_fwd(color.g), Self::_map_fwd(color.b), Self::_map_fwd(color.a))
            }
        }
        impl const From<$name> for Color {
            fn from(color: $name) -> Color {
                Color::rgba($name::_map_bwd(color.r), $name::_map_bwd(color.g), $name::_map_bwd(color.b), $name::_map_bwd(color.a))
            }
        }
    };
    (
        $(#[$doc:meta])* $name:ident, $type:ty, $align:literal;
        $min: literal, $max: literal, $map_fwd_args:tt => $map_fwd_body:expr, $map_bwd_args:tt => $map_bwd_body:expr;
        $($field:ident),*
    ) => {
        color4!(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            $(#[$doc])* $name, $type, $align;
            $min, $max, $map_fwd_args => $map_fwd_body, $map_bwd_args => $map_bwd_body;
            $($field),*
        );
    };
}

macro_rules! color3 {
    (
        #[derive($($derive_args:ident),*)]
        $(#[$doc:meta])* $name:ident, $type:ty, $align:literal;
        $min: literal, $max: literal, $map_fwd_args:tt => $map_fwd_body:expr, $map_bwd_args:tt => $map_bwd_body:expr;
        $($field:ident),*
    ) => {
        $(#[$doc])*
        #[allow(clippy::upper_case_acronyms)]
        #[derive($($derive_args),*)]
        #[repr(C, align($align))]
        pub struct $name {
            $(pub $field: $type,)*
        }
        static_assert!(core::mem::size_of::<$name>() == core::mem::size_of::<$type>() * 3);
        impl const Default for $name {
            fn default() -> Self {
                Self { r: $min, g: $min, b: $min }
            }
        }
        #[allow(clippy::identity_op)]
        impl $name {
            const fn _map_fwd $map_fwd_args -> $type { $map_fwd_body }
            const fn _map_bwd $map_bwd_args -> u8 { $map_bwd_body }
        }
        #[allow(clippy::self_named_constructors)]
        impl $name {
            pub const fn default() -> Self {
                Self { r: $min, g: $min, b: $min }
            }
            pub const fn rgb(r: $type, g: $type, b: $type) -> Self {
                Self { r, g, b }
            }
            pub const fn gray(gray: $type) -> Self {
                Self { r: gray, g: gray, b: gray }
            }
        }
        impl const From<Color> for $name {
            fn from(color: Color) -> Self {
                Self::rgb(Self::_map_fwd(color.r), Self::_map_fwd(color.g), Self::_map_fwd(color.b))
            }
        }
        impl const From<$name> for Color {
            fn from(color: $name) -> Color {
                Color::rgb($name::_map_bwd(color.r), $name::_map_bwd(color.g), $name::_map_bwd(color.b))
            }
        }
    };
    (
        $(#[$doc:meta])* $name:ident, $type:ty, $align:literal;
        $min: literal, $max: literal, $map_fwd_args:tt => $map_fwd_body:expr, $map_bwd_args:tt => $map_bwd_body:expr;
        $($field:ident),*
    ) => {
        color3!(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            $(#[$doc])* $name, $type, $align;
            $min, $max, $map_fwd_args => $map_fwd_body, $map_bwd_args => $map_bwd_body;
            $($field),*
        );
    };
}

macro_rules! color0 {
    (
        $(#[$doc:meta])* $name:ident, $type:ty;
        $default: literal;
        fn _from_color $from_color_args: tt -> $from_color_ret: ty $from_color_body: block
        fn _to_color $to_color_args: tt -> $to_color_ret: ty $to_color_body: block
    ) => {
        $(#[$doc])*
        #[allow(clippy::upper_case_acronyms)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        pub struct $name {
            pub value: $type,
        }
        impl const Default for $name {
            fn default() -> Self {
                Self { value: $default }
            }
        }
        #[allow(clippy::identity_op)]
        impl $name {
            const fn _from_color $from_color_args -> $from_color_ret $from_color_body
            const fn _to_color $to_color_args -> $to_color_ret $to_color_body
        }
        #[allow(clippy::self_named_constructors)]
        impl $name {
            pub const fn default() -> Self {
                Self { value: $default }
            }
            pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
                Self::_from_color(Color::rgb(r, g, b))
            }
            pub const fn gray(gray: u8) -> Self {
                Self::_from_color(Color::gray(gray))
            }
        }
        impl const From<Color> for $name {
            fn from(color: Color) -> Self {
                Self::rgb(color.r, color.g, color.b)
            }
        }
        impl const From<$name> for Color {
            fn from(color: $name) -> Color {
                $name::_to_color(color)
            }
        }
    };
}

mod ansi;
pub use ansi::*;

mod scheme;
pub use scheme::ColorScheme;

color4!(RGBA, u8, 4; 0, 255, (x: u8) => x, (x: u8) => x; r, g, b, a);
color4!(BGRA, u8, 4; 0, 255, (x: u8) => x, (x: u8) => x; b, g, r, a);
color4!(ARGB, u8, 4; 0, 255, (x: u8) => x, (x: u8) => x; a, r, g, b);
color4!(ABGR, u8, 4; 0, 255, (x: u8) => x, (x: u8) => x; a, b, g, r);

color3!(RGB, u8, 1; 0, 255, (x: u8) => x, (x: u8) => x; r, g, b);
color3!(BGR, u8, 1; 0, 255, (x: u8) => x, (x: u8) => x; b, g, r);

color4!(RGBA16, u16, 8; 0, 65535, (x: u8) => (x as u16) << 8 | (x as u16), (x: u16) => (x >> 8) as u8; r, g, b, a);
color4!(BGRA16, u16, 8; 0, 65535, (x: u8) => (x as u16) << 8 | (x as u16), (x: u16) => (x >> 8) as u8; b, g, r, a);
color4!(ARGB16, u16, 8; 0, 65535, (x: u8) => (x as u16) << 8 | (x as u16), (x: u16) => (x >> 8) as u8; a, r, g, b);
color4!(ABGR16, u16, 8; 0, 65535, (x: u8) => (x as u16) << 8 | (x as u16), (x: u16) => (x >> 8) as u8; a, b, g, r);

color4!(#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)] RGBA_F32, f32, 16; 0.0, 1.0, (x: u8) => x as f32 / 255.0, (x: f32) => (x * 255.0) as u8; r, g, b, a);
color4!(#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)] BGRA_F32, f32, 16; 0.0, 1.0, (x: u8) => x as f32 / 255.0, (x: f32) => (x * 255.0) as u8; b, g, r, a);
color4!(#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)] ARGB_F32, f32, 16; 0.0, 1.0, (x: u8) => x as f32 / 255.0, (x: f32) => (x * 255.0) as u8; a, r, g, b);
color4!(#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)] ABGR_F32, f32, 16; 0.0, 1.0, (x: u8) => x as f32 / 255.0, (x: f32) => (x * 255.0) as u8; a, b, g, r);

color3!(#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)] RGB_F32, f32, 4; 0.0, 1.0, (x: u8) => x as f32 / 255.0, (x: f32) => (x * 255.0) as u8; r, g, b);
color3!(#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)] BGR_F32, f32, 4; 0.0, 1.0, (x: u8) => x as f32 / 255.0, (x: f32) => (x * 255.0) as u8; b, g, r);

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

// INFO 添加一种新的颜色类型只需要在此处添加一个新的变体
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

color! {
    /// 8 位色深的 sRGB 颜色空间中的颜色，32 位真彩色。
    /// - 通道顺序由 feature 决定，默认为 RGBA。
    ///
    /// 8-bit color depth in the sRGB color space, 32-bit true color.
    /// - The channel order is determined by the feature, default is RGBA.
    Color, u8, 4
}

color! {
    /// 8 位色深的 sRGB 颜色空间中的颜色，32 位真彩色。
    /// - 通道顺序由 feature 决定，默认为 RGBA。
    /// - 使用 u16 存储每个通道，值范围仍为 0-255。
    ///
    /// 8-bit color depth in the sRGB color space, 32-bit true color.
    /// - The channel order is determined by the feature, default is RGBA.
    /// - Each channel is stored in a u16, but the value range is still 0-255.
    ColorI16, i16, 8
}

color! {
    /// 8 位色深的 sRGB 颜色空间中的颜色，32 位真彩色。
    /// - 通道顺序由 feature 决定，默认为 RGBA。
    /// - 使用 i32 存储每个通道，值范围仍为 0-255。
    ///
    /// 8-bit color depth in the sRGB color space, 32-bit true color.
    /// - The channel order is determined by the feature, default is RGBA.
    /// - Each channel is stored in an i32, but the value range is still 0-255.
    ColorI32, i32, 16
}

color0!(RGB565, u16; 0;
    fn _from_color(color: Color) -> Self {
        let r = (color.r as u16 >> 3) << 11;
        let g = (color.g as u16 >> 2) << 5;
        let b = (color.b as u16 >> 3) << 0;
        Self { value: r | g | b }
    }
    fn _to_color(value: Self) -> Color {
        let r = ((value.value >> 11) & 0x1F) as u8 * 255 / 31;
        let g = ((value.value >> 5) & 0x3F) as u8 * 255 / 63;
        let b = ((value.value >> 0) & 0x1F) as u8 * 255 / 31;
        Color::rgb(r, g, b)
    }
);

color0!(BGR565, u16; 0;
    fn _from_color(color: Color) -> Self {
        let r = (color.r as u16 >> 3) << 0;
        let g = (color.g as u16 >> 2) << 5;
        let b = (color.b as u16 >> 3) << 11;
        Self { value: r | g | b }
    }
    fn _to_color(value: Self) -> Color {
        let r = ((value.value >> 0) & 0x1F) as u8 * 255 / 31;
        let g = ((value.value >> 5) & 0x3F) as u8 * 255 / 63;
        let b = ((value.value >> 11) & 0x1F) as u8 * 255 / 31;
        Color::rgb(r, g, b)
    }
);

macro_rules! from_slice {
    (try $type:ident * $mul:literal ; $($name:ident),*) => {
        $(
            static_assert!(core::mem::size_of::<$name>() == core::mem::size_of::<$type>() * $mul);
            #[expect(unsafe_code)]
            impl $name {
                pub fn ${ concat(try_from_, $type, _slice) } (c: &[$type]) -> Option<&[Self]> {
                    if c.len() % $mul != 0 { return None; }
                    if c.as_ptr() as usize % core::mem::align_of::<Self>() != 0 { return None; }
                    Some(unsafe { core::slice::from_raw_parts(c.as_ptr() as *const Self, c.len() / $mul) })
                }
                pub fn ${ concat(try_from_, $type, _mut_slice) } (c: &mut [$type]) -> Option<&mut [Self]> {
                    if c.len() % $mul != 0 { return None; }
                    if c.as_mut_ptr() as usize % core::mem::align_of::<Self>() != 0 { return None; }
                    Some(unsafe { core::slice::from_raw_parts_mut(c.as_mut_ptr() as *mut Self, c.len() / $mul) })
                }
            }
        )*
    };
    ($type:ident; $($name:ident),*) => {
        $(
            static_assert!(core::mem::size_of::<$name>() == core::mem::size_of::<$type>());
            static_assert!(core::mem::align_of::<$name>() <= core::mem::align_of::<$type>());
            #[expect(unsafe_code)]
            impl $name {
                pub const fn ${ concat(from_, $type, _slice) } (c: &[$type]) -> &[Self] {
                    unsafe { core::slice::from_raw_parts(c.as_ptr() as *const Self, c.len()) }
                }
                pub const fn ${ concat(from_, $type, _mut_slice) } (c: &mut [$type]) -> &mut [Self] {
                    unsafe { core::slice::from_raw_parts_mut(c.as_mut_ptr() as *mut Self, c.len()) }
                }
                #[deprecated(since = "0.0.0", note = "Since this conversion will not fail, use the non-try version instead.")]
                pub const fn ${ concat(try_from_, $type, _slice) } (c: &[$type]) -> Option<&[Self]> {
                    Some(Self::${ concat(from_, $type, _slice) }(c))
                }
                #[deprecated(since = "0.0.0", note = "Since this conversion will not fail, use the non-try version instead.")]
                pub const fn ${ concat(try_from_, $type, _mut_slice) } (c: &mut [$type]) -> Option<&mut [Self]> {
                    Some(Self::${ concat(from_, $type, _mut_slice) }(c))
                }
            }
        )*
    };
}

from_slice!(try u8 * 4; Color, RGBA, BGRA, ARGB, ABGR);
from_slice!(try u8 * 3; RGB, BGR);
from_slice!(try u16 * 4; RGBA16, BGRA16, ARGB16, ABGR16);
from_slice!(try f32 * 4; RGBA_F32, BGRA_F32, ARGB_F32, ABGR_F32);
from_slice!(try f32 * 3; RGB_F32, BGR_F32);

from_slice!(u16; RGB565, BGR565);
from_slice!(u32; Color, RGBA, BGRA, ARGB, ABGR);
from_slice!(u64; RGBA16, BGRA16, ARGB16, ABGR16);
