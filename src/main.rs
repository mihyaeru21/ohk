use std::fmt;
use std::os::raw::c_int;
use std::ptr;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CallNextHookEx, SendInput, SetWindowsHookExW, INPUT, INPUT_KEYBOARD, KBDLLHOOKSTRUCT,
    KEYEVENTF_KEYUP, LLKHF_ALTDOWN, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_LOWER_IL_INJECTED,
    LLKHF_UP, WH_KEYBOARD_LL,
};

static mut h_hook: HHOOK = ptr::null_mut();

struct KeyEvent {
    raw: KBDLLHOOKSTRUCT,
}

impl KeyEvent {
    pub fn new(raw: KBDLLHOOKSTRUCT) -> Self {
        Self { raw }
    }

    pub fn vk_code(&self) -> u32 {
        self.raw.vkCode
    }

    pub fn scan_code(&self) -> u32 {
        self.raw.scanCode
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

#[no_mangle]
pub unsafe extern "system" fn handler(code: c_int, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if let Some(mut k) = ptr::NonNull::new(lp as *mut KBDLLHOOKSTRUCT) {
        let event = KeyEvent::new(*k.as_mut());
        if !event.is_injected() && event.vk_code() == 0x38 {
            let mut input = INPUT::default();
            input.type_ = INPUT_KEYBOARD;
            let mut ki = input.u.ki_mut();
            ki.wVk = 0x39;
            ki.dwFlags = if event.is_up() { KEYEVENTF_KEYUP } else { 0 };

            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as c_int);
            return -1;
        }
        println!("debug: {:?}", event);
    }

    // 握りつぶすなら -1 を返す

    CallNextHookEx(h_hook, code, wp, lp)
}

fn main() {
    nwg::init().expect("oops!");

    unsafe {
        let instance = GetModuleHandleW(ptr::null());
        h_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(handler), instance, 0);
        if h_hook == ptr::null_mut() {
            panic!("failed to set hook!");
        }
    }

    println!("waiting...");
    nwg::dispatch_thread_events();
}
