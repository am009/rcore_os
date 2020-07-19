//! 中断模块
//!
//!

pub mod context;
mod handler;
mod timer;

use riscv::register::{sie, sstatus};

/// 初始化中断相关的子模块
///
/// - [`handler::init`]
/// - [`timer::init`]
pub fn init() {
    handler::init();
    timer::init();
    println!("mod interrupt initialized.");
}

pub fn wait_for_interrupt() {
    unsafe {
        sie::clear_stimer();
        sstatus::set_sie();
        llvm_asm!("wfi" :::: "volatile");
        sstatus::clear_sie();
        sie::set_stimer();
    }
}