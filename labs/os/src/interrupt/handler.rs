
use riscv::register::scause::Exception;
use riscv::register::scause::Interrupt;
use riscv::register::scause::Scause;
use riscv::register::scause::Trap;
use riscv::register::stvec;
use alloc::{format, string::String};

use super::context::Context;
use crate::interrupt::timer;
use crate::process::PROCESSOR;

global_asm!(include_str!("./interrupt.asm"));

pub fn init() {
    unsafe {
        extern "C" {
            /// 声明__interrupt作为函数
            fn __interrupt();
        }
        // 使用 Direct 模式，将中断入口设置为 `__interrupt`
        stvec::write(__interrupt as usize, stvec::TrapMode::Direct);
        // // 开启外部中断使能
        // sie::set_sext();

        // // 在 OpenSBI 中开启外部中断
        // *PhysicalAddress(0x0c00_2080).deref_kernel() = 1u32 << 10;
        // // 在 OpenSBI 中开启串口
        // *PhysicalAddress(0x1000_0004).deref_kernel() = 0x0bu8;
        // *PhysicalAddress(0x1000_0001).deref_kernel() = 0x01u8;
        // // 其他一些外部中断相关魔数
        // *PhysicalAddress(0x0C00_0028).deref_kernel() = 0x07u32;
        // *PhysicalAddress(0x0C20_1000).deref_kernel() = 0u32;

    }
}

/// 中断的处理入口
///
/// `interrupt.asm` 首先保存寄存器至 Context，其作为参数和 scause 以及 stval 一并传入此函数
/// 具体的中断类型需要根据 scause 来推断，然后分别处理
#[no_mangle]
pub fn handle_interrupt(context: &mut Context, scause: Scause, stval: usize) -> *mut Context {
    // println!("{:x?}, {:x?}", scause.cause(), stval);
    {
        let mut processor = PROCESSOR.get();
        let current_thread = processor.current_thread();
        if current_thread.as_ref().inner().dead {
            println!("thread {} exit", current_thread.id);
            processor.kill_current_thread();
            return processor.prepare_next_thread();
        }
    }
    match scause.cause() {
        // 断点中断（ebreak）
        Trap::Exception(Exception::Breakpoint) => breakpoint(context),
        // 时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => supervisor_timer(context),
        // 其他情况，终止当前线程
        _ => Err(format!(
            "unimplemented interrupt type: {:x?}",
            scause.cause()
        )),
    }.unwrap_or_else(|msg| fault(msg, scause, stval))
}

/// 处理 ebreak 断点
///
/// 继续执行，其中 `sepc` 增加 2 字节，以跳过当前这条 `ebreak` 指令
fn breakpoint(context: &mut Context) -> Result<*mut Context, String> {
    println!("Breakpoint at 0x{:x}", context.sepc);
    context.sepc += 2;
    Ok(context)
}

/// 处理时钟中断
///
/// 目前只会在 [`timer`] 模块中进行计数
fn supervisor_timer(context: &mut Context) -> Result<*mut Context, String> {
    timer::tick();
    // PROCESSOR.get().park_current_thread(context);
    // Ok(PROCESSOR.get().prepare_next_thread())
    Ok(PROCESSOR.get().tick(context))
}

/// 出现未能解决的异常
fn fault(msg: String, scause: Scause, stval: usize) -> *mut Context {
    // panic!(
    //     "Unresolved interrupt: {:?}\n{:x?}\nstval: {:x}",
    //     scause.cause(),
    //     context,
    //     stval
    // );
    println!(
        "{:#x?} terminated: {}",
        PROCESSOR.get().current_thread(),
        msg
    );
    println!("cause: {:?}, stval: {:x}", scause.cause(), stval);

    PROCESSOR.get().kill_current_thread();
    // 跳转到 PROCESSOR 调度的下一个线程
    PROCESSOR.get().prepare_next_thread()
}
