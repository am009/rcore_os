pub mod address;
mod bitmap_vector_allocator;
pub mod config;
pub mod frame;
pub mod heap;
pub mod mapping;
pub mod range;

pub type MemoryResult<T> = Result<T, &'static str>;

pub fn init() {
    heap::init();
    println!("heap initialized.");
}
