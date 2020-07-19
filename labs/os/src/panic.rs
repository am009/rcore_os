//! 代替 std 库，实现 panic 和 abort 的功能

use crate::sbi::shutdown;
use core::panic::PanicInfo;

/// 打印 panic 的信息并 [`shutdown`]
///
/// ### `#[panic_handler]` 属性
/// 声明此函数是 panic 的回调
#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    // 需要全局开启 feature(panic_info_message) 才可以调用 .message() 函数
    println!("\x1b[1;31mpanic: '{}'\x1b[0m", info.message().unwrap());

    if let Some(location) = info.location() {
        println!(
            "panic occurred at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    } else {
        println!("panic occurred but can't get location information...");
    }

    shutdown()
}

/// 终止程序
///
/// 调用 [`panic_handler`]
#[no_mangle]
extern "C" fn abort() -> ! {
    panic!("abort()")
}
