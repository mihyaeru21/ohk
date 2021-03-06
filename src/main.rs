fn main() {
    nwg::init().expect("oops!");
    println!("waiting...");
    nwg::dispatch_thread_events();
}
