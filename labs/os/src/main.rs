
#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(const_raw_ptr_to_usize_cast)]
#[macro_use]


mod console;
mod panic;
mod sbi;
mod interrupt;
mod memory;
extern crate alloc;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // 初始化各种模块
    interrupt::init();
    memory::init();

    println!("kernel end at: {:x?}", *memory::config::KERNEL_END_ADDRESS);

    for _ in 0..2 {
        let frame_0 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err)
        };
        let frame_1 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err)
        };
        println!("alloc and get {:x?} and {:x?}", frame_0.address(), frame_1.address());
    }

    unsafe {
        llvm_asm!("ebreak"::::"volatile");
    };
    unreachable!();
}