
pub mod heap;
pub mod frame;
pub mod config;
pub mod address;
pub mod range;

pub type MemoryResult<T> = Result<T, &'static str>;


pub fn init () {
    heap::init();
    println!("heap initialized.");
}