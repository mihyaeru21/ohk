mod key_event;
mod keys;
mod mapping;
mod state;

use key_event::KeyEvent;
use state::State;
use std::os::raw::c_int;
use std::ptr;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CallNextHookEx, SendInput, SetWindowsHookExW, INPUT, INPUT_KEYBOARD, KBDLLHOOKSTRUCT,
    KEYEVENTF_KEYUP, WH_KEYBOARD_LL,
};

static mut H_HOOK: HHOOK = ptr::null_mut();
static mut STATE: State = State {
    last_event: None,
    sending_same_key: false,
    ohk_meta_pressed: false,
    left_ctrl_pressed: false,
    right_ctrl_pressed: false,
};

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

unsafe fn create_input(code: u16, is_up: bool) -> INPUT {
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

        // 単一キーを別のキーにマッピングする挙動
        // このときだけは injected なイベントはスルーする
        if !event.is_injected() {
            if let Some(mapped_code) = mapping::map_simply(event.vk_code()) {
                let inputs = vec![create_input(mapped_code, event.is_up())];
                send_inputs(inputs);
                return -1;
            }
        }

        if !STATE.sending_same_key {
            let state = STATE.as_mapping_state(&event);

            if let Some(mapped_code) = mapping::map(event.vk_code(), &state) {
                let inputs = vec![create_input(mapped_code, event.is_up())];
                send_inputs(inputs);
                return -1;
            }

            // up に合わせた挙動
            if event.is_up() {
                if let Some(mapped_events) = mapping::map_on_up(event.vk_code(), &state) {
                    let inputs = mapped_events
                        .iter()
                        .map(|me| create_input(me.code, me.state == mapping::UD::UP))
                        .collect();

                    if mapped_events.iter().any(|me| me.code == event.vk_code()) {
                        STATE.sending_same_key = true;
                        send_inputs(inputs);
                        STATE.sending_same_key = false;
                    } else {
                        send_inputs(inputs);
                    }
                    return -1;
                }
            }
        }

        println!("debug: {:?}", event);

        STATE.update(event);
    }

    CallNextHookEx(H_HOOK, code, wp, lp)
}
