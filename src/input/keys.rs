use xkeysym::{Keysym, key};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Backspace,
    Tab,
    Enter,
    Escape,
    Space,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowLeft,
    ArrowUp,
    ArrowRight,
    ArrowDown,
    Clear,
    Pause,
    PrintScreen,
    ScrollLock,
    NumLock,
    CapsLock,
    Menu,
    Help,
    Undo,
    Redo,
    Find,
    Cancel,
    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,
    SuperLeft,
    SuperRight,
    MetaLeft,
    MetaRight,
    Function(u8),
    Numpad(u8),
    NumpadAdd,
    NumpadSubtract,
    NumpadMultiply,
    NumpadDivide,
    NumpadDecimal,
    NumpadEqual,
    Character(char),
    Unknown(u32),
}

impl Key {
    pub fn from_keysym(raw: u32) -> Self {
        match raw {
            key::BackSpace => Self::Backspace,
            key::Tab | key::KP_Tab => Self::Tab,
            key::Return | key::KP_Enter => Self::Enter,
            key::Escape => Self::Escape,
            key::space | key::KP_Space => Self::Space,
            key::Delete | key::KP_Delete => Self::Delete,
            key::Insert | key::KP_Insert => Self::Insert,
            key::Home | key::KP_Home => Self::Home,
            key::End | key::KP_End => Self::End,
            key::Page_Up | key::KP_Page_Up => Self::PageUp,
            key::Page_Down | key::KP_Page_Down => Self::PageDown,
            key::Left | key::KP_Left => Self::ArrowLeft,
            key::Up | key::KP_Up => Self::ArrowUp,
            key::Right | key::KP_Right => Self::ArrowRight,
            key::Down | key::KP_Down => Self::ArrowDown,
            key::Clear => Self::Clear,
            key::Pause => Self::Pause,
            key::Print => Self::PrintScreen,
            key::Scroll_Lock => Self::ScrollLock,
            key::Num_Lock => Self::NumLock,
            key::Caps_Lock => Self::CapsLock,
            key::Menu => Self::Menu,
            key::Help => Self::Help,
            key::Undo => Self::Undo,
            key::Redo => Self::Redo,
            key::Find => Self::Find,
            key::Cancel => Self::Cancel,
            key::Shift_L => Self::ShiftLeft,
            key::Shift_R => Self::ShiftRight,
            key::Control_L => Self::ControlLeft,
            key::Control_R => Self::ControlRight,
            key::Alt_L => Self::AltLeft,
            key::Alt_R => Self::AltRight,
            key::Super_L => Self::SuperLeft,
            key::Super_R => Self::SuperRight,
            key::Meta_L => Self::MetaLeft,
            key::Meta_R => Self::MetaRight,
            key::KP_Add => Self::NumpadAdd,
            key::KP_Subtract => Self::NumpadSubtract,
            key::KP_Multiply => Self::NumpadMultiply,
            key::KP_Divide => Self::NumpadDivide,
            key::KP_Decimal | key::KP_Separator => Self::NumpadDecimal,
            key::KP_Equal => Self::NumpadEqual,
            key::KP_0..=key::KP_9 => Self::Numpad((raw - key::KP_0) as u8),
            key::F1..=key::F35 => Self::Function((raw - key::F1 + 1) as u8),
            _ => Keysym::new(raw)
                .key_char()
                .map(Self::Character)
                .unwrap_or(Self::Unknown(raw)),
        }
    }

    pub const fn is_modifier(self) -> bool {
        matches!(
            self,
            Self::ShiftLeft
                | Self::ShiftRight
                | Self::ControlLeft
                | Self::ControlRight
                | Self::AltLeft
                | Self::AltRight
                | Self::SuperLeft
                | Self::SuperRight
                | Self::MetaLeft
                | Self::MetaRight
        )
    }
}

impl From<u32> for Key {
    fn from(keysym: u32) -> Self {
        Self::from_keysym(keysym)
    }
}
