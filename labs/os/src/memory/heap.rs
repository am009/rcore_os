
use super::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;
extern crate alloc;

static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

pub fn init() {
    let start = unsafe { HEAP_SPACE.as_ptr() as usize };
    println!("heap initializing at 0x{:x} - 0x{:x}.", start, start + KERNEL_HEAP_SIZE);
    unsafe {
        HEAP.lock().init(
            start, KERNEL_HEAP_SIZE
        )
    }
}

#[alloc_error_handler]
fn alloc_error_handler(_: alloc::alloc::Layout) -> ! {
    panic!("alloc error")
}