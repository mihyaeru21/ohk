mod key_event;
mod keys;
mod mapping;
mod state;

use key_event::KeyEvent;
use state::State;
use std::os::raw::c_int;
use std::ptr;
use winapi::{
    shared::minwindef::{LPARAM, LRESULT, WPARAM},
    shared::windef::HHOOK,
    um::{
        errhandlingapi::GetLastError,
        libloaderapi::GetModuleHandleW,
        winuser::{
            CallNextHookEx, SendInput, SetWindowsHookExW, UnhookWindowsHookEx, INPUT,
            INPUT_KEYBOARD, KBDLLHOOKSTRUCT, KEYEVENTF_KEYUP, WH_KEYBOARD_LL,
        },
    },
};

// 全体的には以下を参考にして作った
// http://steavevaivai.hatenablog.com/entry/2018/07/24/220350

static mut H_HOOK: HHOOK = ptr::null_mut();
static mut STATE: State = State {
    last_event: None,
    sending_same_key: false,
    ohk_meta_pressed: false,
    left_ctrl_pressed: false,
    right_ctrl_pressed: false,
};

// SetWindowsHookExW については以下を参照
// https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw
pub fn register_hook() {
    unsafe {
        let instance = GetModuleHandleW(ptr::null());
        H_HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(handler), instance, 0);
        if H_HOOK == ptr::null_mut() {
            panic!("failed to register hook!");
        }
    }
}

// UnhookWindowsHookEx
// https://docs.microsoft.com/ja-jp/windows/win32/api/winuser/nf-winuser-unhookwindowshookex
pub fn unregister_hook() {
    unsafe {
        if H_HOOK != ptr::null_mut() {
            let ret = UnhookWindowsHookEx(H_HOOK);
            if ret == 0 {
                let code = GetLastError();
                panic!(format!("failed to unregister hook! code: {:?}", code));
            }
        }
    }
}

// INPUT については以下を参照
// https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-input
// https://docs.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput
unsafe fn create_input(code: u16, is_up: bool) -> INPUT {
    let mut input = INPUT::default();
    input.type_ = INPUT_KEYBOARD;
    let mut ki = input.u.ki_mut();
    ki.wVk = code;
    ki.dwFlags = if is_up { KEYEVENTF_KEYUP } else { 0 };
    input
}

// SendInput については以下を参照
// https://docs.microsoft.com/ja-jp/windows/win32/api/winuser/nf-winuser-sendinput
unsafe fn send_inputs(mut inputs: Vec<INPUT>) {
    SendInput(
        inputs.len() as u32,
        inputs.as_mut_ptr(),
        std::mem::size_of::<INPUT>() as c_int,
    );
    // TODO: check result
}

// この関数のインタフェースについては以下を参照
// https://docs.microsoft.com/ja-jp/windows/win32/api/winuser/nc-winuser-hookproc
// https://docs.microsoft.com/ja-jp/windows/win32/api/winuser/ns-winuser-kbdllhookstruct
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

        log::info!("{:?}", event);

        STATE.update(event);
    }

    CallNextHookEx(H_HOOK, code, wp, lp)
}
