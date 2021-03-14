mod key_event;

use key_event::KeyEvent;
use std::os::raw::c_int;
use std::ptr;
use winapi::shared::minwindef::{LPARAM, LRESULT, WORD, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CallNextHookEx, SendInput, SetWindowsHookExW, INPUT, INPUT_KEYBOARD, KBDLLHOOKSTRUCT,
    KEYEVENTF_KEYUP, WH_KEYBOARD_LL,
};

static mut H_HOOK: HHOOK = ptr::null_mut();
static mut LAST_EVENT: Option<KeyEvent> = None;
static mut SENDING_SAME_KEY: bool = false;

pub fn register_hook() {
    unsafe {
        let instance = GetModuleHandleW(ptr::null());
        H_HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(handler), instance, 0);
        if H_HOOK == ptr::null_mut() {
            panic!("failed to set hook!");
        }
    }
}

// TODO: unset

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

    CallNextHookEx(H_HOOK, code, wp, lp)
}
