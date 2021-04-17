mod key_event;

use crate::config;
use key_event::KeyEvent;
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
static mut LAST_EVENT: Option<KeyEvent> = None;
static mut SENDING_SAME_KEY: bool = false;
static mut OHK_META_PRESSED: bool = false;
static mut LEFT_CTRL_PRESSED: bool = false;
static mut RIGHT_CTRL_PRESSED: bool = false;

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
            if let Some(mapped_code) = config::simple_map(event.vk_code()) {
                let inputs = vec![create_input(mapped_code, event.is_up())];
                send_inputs(inputs);
                return -1;
            }
        }

        // OHK_META キーとの組み合わせによる挙動
        if !SENDING_SAME_KEY && OHK_META_PRESSED {
            if let Some(mapped_code) = config::simple_map_with_meta(event.vk_code()) {
                let inputs = vec![create_input(mapped_code, event.is_up())];
                send_inputs(inputs);
                return -1;
            }
        }

        // up に合わせた挙動
        if !SENDING_SAME_KEY && event.is_up() {
            let state = config::State {
                left_ctrl: LEFT_CTRL_PRESSED,
                right_ctrl: LEFT_CTRL_PRESSED,
                ohk_meta: OHK_META_PRESSED,
                just_down_up: LAST_EVENT.is_some()
                    && LAST_EVENT.as_ref().unwrap().vk_code() == event.vk_code(),
            };

            if let Some(mapped_events) = config::map_on_up(event.vk_code(), &state) {
                let has_same_key = mapped_events.iter().any(|me| me.code == event.vk_code());
                if has_same_key {
                    SENDING_SAME_KEY = true
                }
                let inputs = mapped_events
                    .iter()
                    .map(|me| create_input(me.code, me.state == config::UD::UP))
                    .collect();
                send_inputs(inputs);
                if has_same_key {
                    SENDING_SAME_KEY = false
                }
                return -1;
            }
        }

        let thread_id = std::thread::current().id();
        println!(
            "debug({:?}): hoge: {:?}, {:?}",
            thread_id, SENDING_SAME_KEY, event
        );

        // 装飾キーの状態を保持しておく
        // キーが複数ある場合は完全に正しく状態を保てない場合があるけどとりあえずこれで実用上問題ない
        match event.vk_code() {
            config::KEY_LEFT_CTRL => LEFT_CTRL_PRESSED = !event.is_up(),
            config::KEY_RIGHT_CTRL => RIGHT_CTRL_PRESSED = !event.is_up(),
            config::KEY_OHK_META => OHK_META_PRESSED = !event.is_up(),
            _ => {}
        }

        LAST_EVENT = Some(event);
    }

    CallNextHookEx(H_HOOK, code, wp, lp)
}
