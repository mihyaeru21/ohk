#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod hook;
mod ui;

fn main() {
    env_logger::init();
    hook::register_hook();
    ui::run();
    hook::unregister_hook();
}
