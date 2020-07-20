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
mod drivers;
mod fs;
extern crate alloc;

use process::{Process, PROCESSOR, Thread};
use spin::RwLock;
use alloc::sync::Arc;
use memory::address::PhysicalAddress;

global_asm!(include_str!("entry.asm"));

extern "C" {
    fn boot_page_table();
}

#[no_mangle]
pub extern "C" fn rust_main(hart_id: usize, dtb_pa: PhysicalAddress) -> ! {
    println!("hart {} initializing...", hart_id);
    println!("dtb at {:x?}.", dtb_pa);
    println!("kernel end at: {:x?}", *memory::config::KERNEL_END_ADDRESS);

    
    unsafe {
        let sscratch: usize = process::KERNEL_STACK.get_top();
        llvm_asm!("csrw sscratch, $0" :: "r"(sscratch) :: "volatile");
    }    

    // 初始化各种模块
    memory::init();
    interrupt::init();
    drivers::init(dtb_pa);
    fs::init();

    {
        let mut processor = PROCESSOR.get();
        let kernel_process = Process::new_kernel().unwrap();
        // for i in 1..33usize {
        //     processor.add_thread(create_kernel_thread(
        //         kernel_process.clone(),
        //         test_kernel_thread as usize,
        //         Some(&[i]),
        //     ));
        // }
        processor.add_thread(create_kernel_thread(kernel_process.clone(), test_kernel as usize, Some(&[0usize])));
    }

    unsafe { PROCESSOR.unsafe_get().run() }

}


fn test_kernel_thread(id: usize) {
    println!("hello from kernel thread {}", id);
}

fn kernel_thread_exit() {
    // 当前线程标记为结束
    PROCESSOR.get().current_thread().as_ref().inner().dead = true;
    // 制造一个中断来交给操作系统处理
    unsafe { llvm_asm!("ebreak" :::: "volatile") };
}

pub fn create_kernel_thread(
    process: Arc<RwLock<Process>>,
    entry_point: usize,
    arguments: Option<&[usize]>,
) -> Arc<Thread> {
    // 创建线程
    let thread = Thread::new(process, entry_point, arguments).unwrap();
    // 设置线程的返回地址为 kernel_thread_exit
    thread
        .as_ref()
        .inner()
        .context
        .as_mut()
        .unwrap()
        .set_ra(kernel_thread_exit as usize);

    thread
}

fn test_kernel(id: usize) {
    println!("hello from kernel thread {}", id);


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

    fs::ROOT_INODE
        .create("tmp", rcore_fs::vfs::FileType::Dir, 0o666)
        .expect("failed to mkdir /tmp");
    // 输出根文件目录内容
    fs::ls("/");
    println!("fs test passed");

    loop {}
}