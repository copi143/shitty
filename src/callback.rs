use alloc::boxed::Box;
use alloc::string::String;

// macro_rules! pty_write_enum {
//     (
//         $(#[$enum_meta:meta])*
//         $vis:vis enum $name:ident {
//             $(
//                 $(#[$variant_meta:meta])*
//                 $variant:ident($type:ty) => $call:expr,
//             )*
//         }
//     ) => {
//         $(#[$enum_meta])*
//         $vis enum $name {
//             $(
//                 $(#[$variant_meta])*
//                 $variant($type),
//             )*
//         }

//         $(
//             impl ::core::convert::From<$type> for $name {
//                 fn from(v: $type) -> Self {
//                     $name::$variant(v)
//                 }
//             }
//         )*

//         impl PtyWrite {
//             pub fn call(&self, data: &[u8]) {
//                 match self {
//                     $(
//                         Self::$variant(f) => ($call)(f, data),
//                     )*
//                 }
//             }
//         }
//     };
// }

// pty_write_enum! {
//     /// Type alias for a callback that writes data to the pseudo‑terminal (PTY).
//     pub enum PtyWrite {
//         Bytes(Box<dyn Fn(&[u8]) + Send + Sync>) => |f,data| f(data),
//         String(Box<dyn Fn(&str) + Send + Sync>) => {
//             if let Ok(s) = core::str::from_utf8(data) {
//                 f(s);
//             }
//         },
//         C(PtyWriteC) => f(data.as_ptr(), data.len()),
//     }
// }

/// Type alias for a callback that writes data to the pseudo-terminal (PTY).
pub type PtyWrite = Box<dyn Fn(&[u8]) + Send + Sync>;
/// Type alias for a callback that retrieves text from the clipboard.
pub type ClipboardGet = Box<dyn Fn() -> Option<String> + Send + Sync>;
/// Type alias for a callback that sets text to the clipboard.
pub type ClipboardSet = Box<dyn Fn(String) + Send + Sync>;
/// Type alias for a callback that triggers a bell sound or visual indication.
pub type Bell = Box<dyn Fn() + Send + Sync>;
/// Type alias for a callback that updates the terminal title.
pub type Title = Box<dyn Fn(Option<String>) + Send + Sync>;

/// C version of the `PtyWrite` callback.
///
/// See [`callback::PtyWrite`](crate::callback::PtyWrite) for details.
pub type PtyWriteC = extern "C" fn(*const u8, usize);
/// C version of the `ClipboardGet` callback.
///
/// See [`callback::ClipboardGet`](crate::callback::ClipboardGet) for details.
pub type ClipboardGetC = extern "C" fn() -> *mut u8;
/// C version of the `ClipboardSet` callback.
///
/// See [`callback::ClipboardSet`](crate::callback::ClipboardSet) for details.
pub type ClipboardSetC = extern "C" fn(*const u8, usize);
/// C version of the `Bell` callback.
///
/// See [`callback::Bell`](crate::callback::Bell) for details.
pub type BellC = extern "C" fn();
/// C version of the `Title` callback.
///
/// See [`callback::Title`](crate::callback::Title) for details.
pub type TitleC = extern "C" fn(*const u8, usize);

#[derive(Default)]
pub struct Callbacks {
    /// See [`callback::PtyWrite`](crate::callback::PtyWrite) for details.
    pub pty_write: Option<PtyWrite>,
    /// See [`callback::ClipboardGet`](crate::callback::ClipboardGet) for details.
    pub clipboard_get: Option<ClipboardGet>,
    /// See [`callback::ClipboardSet`](crate::callback::ClipboardSet) for details.
    pub clipboard_set: Option<ClipboardSet>,
    /// See [`callback::Bell`](crate::callback::Bell) for details.
    pub bell: Option<Bell>,
    /// See [`callback::Title`](crate::callback::Title) for details.
    pub title: Option<Title>,
}

assert_send_sync!(Callbacks);
