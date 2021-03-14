use std::fmt;
use std::os::raw::c_int;
use std::ptr;
use winapi::shared::minwindef::{LPARAM, LRESULT, WORD, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CallNextHookEx, SendInput, SetWindowsHookExW, INPUT, INPUT_KEYBOARD, KBDLLHOOKSTRUCT,
    KEYEVENTF_KEYUP, LLKHF_ALTDOWN, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_LOWER_IL_INJECTED,
    LLKHF_UP, WH_KEYBOARD_LL,
};

static mut H_HOOK: HHOOK = ptr::null_mut();
static mut LAST_EVENT: Option<KeyEvent> = None;
static mut SENDING_SAME_KEY: bool = false;

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

unsafe fn create_input(code: WORD, is_up: bool) -> INPUT {
    let mut input = INPUT::default();
    input.type_ = INPUT_KEYBOARD;
    let mut ki = input.u.ki_mut();
    ki.wVk = code;
    ki.dwFlags = if is_up { KEYEVENTF_KEYUP } else { 0 };
    input
}

unsafe fn send_inputs(mut inputs: Vec<INPUT>) {
    SendInput(
        inputs.len() as u32,
        inputs.as_mut_ptr(),
        std::mem::size_of::<INPUT>() as c_int,
    );
    // TODO: check result
}

#[no_mangle]
pub unsafe extern "system" fn handler(code: c_int, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if let Some(mut k) = ptr::NonNull::new(lp as *mut KBDLLHOOKSTRUCT) {
        let event = KeyEvent::new(*k.as_mut());

        if !event.is_injected() {
            match event.vk_code() {
                0xa4 => {
                    let inputs = vec![create_input(0xa4, event.is_up())];
                    send_inputs(inputs);
                    return -1;
                }
                0xa5 => {
                    let inputs = vec![create_input(0xa5, event.is_up())];
                    send_inputs(inputs);
                    return -1;
                }
                _ => {}
            }
        }

        if let Some(last) = &LAST_EVENT {
            if !SENDING_SAME_KEY && event.is_up() && event.vk_code() == last.vk_code() {
                match event.vk_code() {
                    // left alt
                    0xa4 => {
                        let inputs = vec![
                            create_input(0x07, false),
                            create_input(0x07, true),
                            create_input(0xa4, true),
                            create_input(0x1d, false),
                            create_input(0x1d, true),
                        ];
                        SENDING_SAME_KEY = true;
                        send_inputs(inputs);
                        SENDING_SAME_KEY = false;
                        return -1;
                    }
                    // right alt
                    0xa5 => {
                        let inputs = vec![
                            create_input(0x07, false),
                            create_input(0x07, true),
                            create_input(0xa5, true),
                            create_input(0x1c, false),
                            create_input(0x1c, true),
                        ];
                        SENDING_SAME_KEY = true;
                        send_inputs(inputs);
                        SENDING_SAME_KEY = false;
                        return -1;
                    }
                    _ => {}
                }
            }
        }

        let thread_id = std::thread::current().id();
        println!(
            "debug({:?}): hoge: {:?}, {:?}",
            thread_id, SENDING_SAME_KEY, event
        );

        LAST_EVENT = Some(event);
    }

    // 握りつぶすなら -1 を返す

    CallNextHookEx(H_HOOK, code, wp, lp)
}

fn main() {
    nwg::init().expect("oops!");

    unsafe {
        let instance = GetModuleHandleW(ptr::null());
        H_HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(handler), instance, 0);
        if H_HOOK == ptr::null_mut() {
            panic!("failed to set hook!");
        }
    }

    println!("waiting...");
    nwg::dispatch_thread_events();
}
