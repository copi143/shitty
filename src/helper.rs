#![allow(unused_macros)]
#![allow(dead_code)]

#[cfg(feature = "rustc-likely-unlikely")]
pub use core::hint::{likely, unlikely};

#[cfg(not(feature = "rustc-likely-unlikely"))]
#[inline(always)]
pub const fn likely(b: bool) -> bool {
    b
}

#[cfg(not(feature = "rustc-likely-unlikely"))]
#[inline(always)]
pub const fn unlikely(b: bool) -> bool {
    b
}

macro_rules! static_assert {
    ($cond:expr, $msg:expr) => {
        const _: () = {
            if !$cond {
                panic!($msg);
            }
        };
    };
    ($cond:expr) => {
        const _: () = {
            if !$cond {
                panic!("Assertion failed");
            }
        };
    };
}

pub const fn assert_send_sync_helper<T: Send + Sync>() {}

/// Assert that a type is `Send` and `Sync` at compile time.
macro_rules! assert_send_sync {
    ($ty:ty) => {
        const _: () = crate::helper::assert_send_sync_helper::<$ty>();
    };
}

/// Define a global constant with a specific type and value.
macro_rules! ffi_const {
    ($name:ident, $type:ty, $value:expr) => {
        #[unsafe(no_mangle)]
        pub static $name: $type = $value;
    };
}

macro_rules! wasm_const {
    ($name:ident, $type:ty, $value:expr) => {
        #[wasm_bindgen]
        #[allow(nonstandard_style)]
        pub fn ${ concat(__get_, $name) }() -> $type {
            $value
        }
    };
}

macro_rules! call {
    ($self:ident, $callback:ident) => {
        if let Some(handler) = $self.callbacks.lock().$callback.as_ref() {
            handler();
        }
    };
    ($self:ident, $callback:ident, $($args:expr),*) => {
        if let Some(handler) = $self.callbacks.lock().$callback.as_ref() {
            handler($($args),*);
        }
    };
}

macro_rules! enum_map {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident : $ty:ty {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $value:expr
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant = $value,
            )*
        }
        const impl From<$name> for $ty {
            fn from(value: $name) -> Self {
                match value {
                    $(
                        $name::$variant => $value,
                    )*
                }
            }
        }
        const impl TryFrom<$ty> for $name {
            type Error = ();
            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                match value {
                    $(
                        $value => Ok(Self::$variant),
                    )*
                    _ => Err(()),
                }
            }
        }
    };
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident : $ty:ty {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $value:expr
            ),* $(,)?
        }
        --- SAME AS ---
        $(
            $(#[$same_meta:meta])*
            $other:path
        ),+ $(,)?
    ) => {
        enum_map! {
            $(#[$enum_meta])*
            $vis enum $name : $ty {
                $(
                    $(#[$variant_meta])*
                    $variant = $value,
                )*
            }
        }
        bidirectional_from!($name = [$($(#[$same_meta])* $other),*] { $($variant),* });
    };
}

macro_rules! bidirectional_from {
    ($(#[$meta:meta])* $a:path = $b:path { $($variant:ident),* $(,)? }) => {
        $(#[$meta])*
        const impl From<$a> for $b {
            fn from(value: $a) -> Self {
                match value {
                    $( <$a>::$variant => <$b>::$variant, )*
                }
            }
        }
        $(#[$meta])*
        const impl From<$b> for $a {
            fn from(value: $b) -> Self {
                match value {
                    $( <$b>::$variant => <$a>::$variant, )*
                }
            }
        }
    };
    ($self:path = [$($(#[$other_meta:meta])* $other:path),*] $variants:tt ) => {
        $(
            bidirectional_from! {
                $(#[$other_meta])* $self = $other $variants
            }
        )*
    };
}

pub struct BufWriter<'buf> {
    buf: &'buf mut [u8],
    pos: usize,
}

impl<'buf> BufWriter<'buf> {
    pub const fn new(buf: &'buf mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub const fn len(&self) -> usize {
        self.pos
    }

    pub fn into_slice(self) -> &'buf [u8] {
        &self.buf[..self.pos]
    }
}

impl core::fmt::Write for BufWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();

        if self.pos + bytes.len() > self.buf.len() {
            return Err(core::fmt::Error);
        }

        self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
        Ok(())
    }
}

pub enum TerminalProcessInput<'input> {
    Inner([u8; 4], u8),
    Outer(&'input [u8]),
}

impl From<u8> for TerminalProcessInput<'_> {
    fn from(value: u8) -> Self {
        Self::Inner([value, 0, 0, 0], 1)
    }
}

impl From<char> for TerminalProcessInput<'_> {
    fn from(value: char) -> Self {
        let mut buf = [0; 4];
        let len = value.encode_utf8(&mut buf).len();
        Self::Inner(buf, len as u8)
    }
}

impl<'input> From<&'input str> for TerminalProcessInput<'input> {
    fn from(value: &'input str) -> Self {
        Self::Outer(value.as_bytes())
    }
}

impl<'input> From<&'input [u8]> for TerminalProcessInput<'input> {
    fn from(value: &'input [u8]) -> Self {
        Self::Outer(value)
    }
}

impl<'input, const N: usize> From<&'input [u8; N]> for TerminalProcessInput<'input> {
    fn from(arr: &'input [u8; N]) -> Self {
        Self::from(&arr[..])
    }
}

impl AsRef<[u8]> for TerminalProcessInput<'_> {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Inner(buf, len) => &buf[..*len as usize],
            Self::Outer(b) => b,
        }
    }
}
