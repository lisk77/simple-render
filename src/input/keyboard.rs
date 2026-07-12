use crate::wayland::SurfaceId;

use super::{Key, WidgetId};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeyboardModifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub caps_lock: bool,
    pub logo: bool,
    pub num_lock: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyboardEventKind {
    Enter {
        pressed_keycodes: Vec<u32>,
        pressed_keysyms: Vec<u32>,
    },
    Leave,
    Key {
        state: KeyState,
        raw_code: u32,
        keysym: u32,
        key: Key,
        utf8: Option<String>,
    },
    Modifiers {
        modifiers: KeyboardModifiers,
        layout: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyboardEvent {
    pub surface: Option<SurfaceId>,
    pub time: Option<u32>,
    pub serial: Option<u32>,
    pub kind: KeyboardEventKind,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyboardState {
    pub focused: Option<WidgetId>,
    pub surface: Option<SurfaceId>,
    pub modifiers: KeyboardModifiers,
    pub events: Vec<KeyboardEvent>,
}
