//! 系统调用


pub const STDIN: usize = 0;
pub const STDOUT: usize = 1;

const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;

fn syscall(id: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    let mut ret;
    unsafe {
        llvm_asm!("ecall"
            : "={x10}" (ret)
            : "{x10}" (arg0), "{x11}" (arg1), "{x12}" (arg2), "{x17}" (id)
            : "memory"
            : "volatile");
    }
    ret
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    loop {
        let ret = syscall(
            SYSCALL_READ,
            fd,
            buffer as *const [u8] as *const u8 as usize,
            buffer.len()
        );
        if ret > 0 {
            return ret;
        }
    }
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(
        SYSCALL_WRITE,
        fd,
        buffer as *const [u8] as *const u8 as usize,
        buffer.len()
    )
}

pub fn sys_exit(code: isize) -> ! {
    syscall(SYSCALL_EXIT, code as usize, 0, 0);
    unreachable!()
}