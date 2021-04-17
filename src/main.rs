mod config;
mod hook;

fn main() {
    nwg::init().expect("oops!");
    hook::register_hook();
    println!("waiting...");
    nwg::dispatch_thread_events();
}
