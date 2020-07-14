
mod heap;
mod config;

pub fn init () {
    heap::init();
    println!("heap initialized.");
}