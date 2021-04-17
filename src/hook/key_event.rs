use std::fmt;
use winapi::um::winuser::{
    KBDLLHOOKSTRUCT, LLKHF_ALTDOWN, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_LOWER_IL_INJECTED,
    LLKHF_UP,
};

pub struct KeyEvent {
    raw: KBDLLHOOKSTRUCT,
}

impl KeyEvent {
    pub fn new(raw: KBDLLHOOKSTRUCT) -> Self {
        Self { raw }
    }

    pub fn vk_code(&self) -> u16 {
        self.raw.vkCode as u16
    }

    pub fn scan_code(&self) -> u16 {
        self.raw.scanCode as u16
    }

    pub fn is_extended(&self) -> bool {
        self.raw.flags & LLKHF_EXTENDED > 0
    }

    pub fn is_lower_il_injected(&self) -> bool {
        self.raw.flags & LLKHF_LOWER_IL_INJECTED > 0
    }

    pub fn is_injected(&self) -> bool {
        self.raw.flags & LLKHF_INJECTED > 0
    }

    pub fn is_altdown(&self) -> bool {
        self.raw.flags & LLKHF_ALTDOWN > 0
    }

    pub fn is_up(&self) -> bool {
        self.raw.flags & LLKHF_UP > 0
    }
}

impl fmt::Debug for KeyEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyEvent")
            .field("vk_code", &self.vk_code())
            .field("scan_code", &self.scan_code())
            .field("extended", &self.is_extended())
            .field("lower_il_injected", &self.is_lower_il_injected())
            .field("injected", &self.is_injected())
            .field("altdown", &self.is_altdown())
            .field("up", &self.is_up())
            .finish()
    }
}
