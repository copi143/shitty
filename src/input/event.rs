use alloc::vec::Vec;

use super::PointerButton;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Char(char),
    Bytes(Vec<u8>),
    String(&'static str),
    Copy,
    Paste,
    SetColorScheme(usize),
    KbdScroll {
        up: bool,
        page: bool,
    },
    #[cfg(feature = "winit")]
    WinitKeyEvent(WinitKeyEvent),
    #[cfg(feature = "winit")]
    WinitModifiers(WinitModifiers),

    PointerMove(i32, i32),
    Scroll(i32),
    PointerPress(PointerButton),
    PointerRelease(PointerButton),
    PointerEnter,
    PointerLeave,
}

#[cfg(feature = "winit")]
use winit::event::{
    ElementState as WinitElementState, KeyEvent as WinitKeyEvent, Modifiers as WinitModifiers,
    MouseButton as WinitMouseButton,
};

#[cfg(feature = "winit")]
impl TryFrom<(WinitElementState, WinitMouseButton)> for Event {
    type Error = ();

    fn try_from(pair: (WinitElementState, WinitMouseButton)) -> Result<Self, Self::Error> {
        let (state, button) = pair;
        let button = PointerButton::try_from(button)?;
        match state {
            WinitElementState::Pressed => Ok(Event::PointerPress(button)),
            WinitElementState::Released => Ok(Event::PointerRelease(button)),
        }
    }
}

#[cfg(feature = "winit")]
impl From<WinitKeyEvent> for Event {
    fn from(event: WinitKeyEvent) -> Self {
        Event::WinitKeyEvent(event)
    }
}

#[cfg(feature = "winit")]
impl From<WinitModifiers> for Event {
    fn from(modifiers: WinitModifiers) -> Self {
        Event::WinitModifiers(modifiers)
    }
}
