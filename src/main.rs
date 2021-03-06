use std::os::raw::c_int;
use std::ptr;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CallNextHookEx, INPUT_u, SendInput, SetWindowsHookExW, INPUT, INPUT_KEYBOARD, KBDLLHOOKSTRUCT,
    KEYBDINPUT, KEYEVENTF_KEYUP, LLKHF_INJECTED, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

static mut h_hook: HHOOK = ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn handler(code: c_int, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if let Some(mut k) = ptr::NonNull::new(lp as *mut KBDLLHOOKSTRUCT) {
        let kbd = k.as_mut();
        let is_injected = (kbd.flags & LLKHF_INJECTED) > 0;
        if !is_injected && kbd.vkCode == 0x38 {
            let mut input = INPUT::default();
            input.type_ = INPUT_KEYBOARD;
            let mut ki = input.u.ki_mut();
            ki.wVk = 0x39;
            ki.dwFlags = match (wp as u32) {
                WM_KEYDOWN | WM_SYSKEYDOWN => 0,
                _ => KEYEVENTF_KEYUP,
            };

            SendInput(1, &mut input, std::mem::size_of::<INPUT>() as c_int);
            return -1;
        }
        println!(
            "code: {:?}, w: {:?}, vkCode: {:?}, scanCode: {:?}, flags: {:?}",
            code, wp, kbd.vkCode, kbd.scanCode, kbd.flags
        );
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
