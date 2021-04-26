mod hook;
mod ui;

fn main() {
    hook::register_hook();
    ui::run();
    hook::unregister_hook();
}
