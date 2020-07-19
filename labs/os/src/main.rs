#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(const_raw_ptr_to_usize_cast)]
#![warn(clippy::all)]
#![feature(slice_fill)]
#![allow(dead_code)]
#![feature(drain_filter)]
#[macro_use]

mod console;
mod interrupt;
mod memory;
mod panic;
mod process;
mod sbi;
mod unsafe_wrapper;
extern crate alloc;

global_asm!(include_str!("entry.asm"));

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // 初始化各种模块
    memory::init();
    interrupt::init();
    
    start_kernel_thread(test_kernel_thread as usize, Some(&[0usize]));

    process::PROCESSOR.get().run()

}

fn start_kernel_thread(entry_point: usize, arguments: Option<&[usize]>) {
    let process = process::Process::new_kernel().unwrap();
    let thread = process::Thread::new(process, entry_point, arguments).unwrap();
    process::PROCESSOR.get().add_thread(thread);
}

fn test_kernel_thread(id: usize) {
    println!("hello from kernel thread {}", id);

    println!("kernel end at: {:x?}", *memory::config::KERNEL_END_ADDRESS);

    for _ in 0..2 {
        let frame_0 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err),
        };
        let frame_1 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err),
        };
        println!(
            "alloc and get {:x?} and {:x?}",
            frame_0.address(),
            frame_1.address()
        );
    }

    // 动态内存分配测试
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    let v = Box::new(5);
    assert_eq!(*v, 5);
    core::mem::drop(v);
    println!("heap test started");
    let mut vec = Vec::new();
    for i in 0..10000 {
        vec.push(i);
    }
    assert_eq!(vec.len(), 10000);
    for (i, value) in vec.into_iter().enumerate() {
        assert_eq!(value, i);
    }
    println!("heap test passed");

    unsafe {
        llvm_asm!("ebreak"::::"volatile");
    };
    unreachable!();

}