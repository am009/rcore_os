

use crate::syscall::*;
use alloc::string::String;
use core::fmt::{self, Write};

struct Stdout;

impl Write for Stdout {
    /// 打印一个字符串
    fn write_str(&mut self, s: &str) -> fmt::Result {
        sys_write(STDOUT, s.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, ($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt, $(, $($arg)+)?));
    };
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

pub fn getchar() -> u8 {
    let mut c = [0u8; 1];
    sys_read(STDIN, &mut c);
    c[0]
}

pub fn getchars() -> String {
    let mut buffer = [0u8; 64];
    loop {
        let size = sys_read(STDIN, &mut buffer);
        if let Ok(string) = String::from_utf8(buffer.iter().copied().take(size as usize).collect()) {
            return string;
        }
    }
}