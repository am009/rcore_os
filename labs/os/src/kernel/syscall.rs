
use super::*;

pub const SYS_READ: usize = 63;
pub const SYS_WRITE: usize = 64;
pub const SYS_EXIT: usize = 93;

pub(super) enum SyscallResult {
    Proceed(isize),
    Park(isize),
    Kill
}

pub fn syscall_handler(context: &mut Context) -> *mut Context {
    // skip ecall
    context.sepc += 4;

    let syscall_id = context.x[17];
    let args = [context.x[10], context.x[11], context.x[12]];

    let result = match syscall_id {
        SYS_READ => sys_read(args[0], args[1] as *mut u8, args[2]),
        SYS_WRITE => sys_write(args[0], args[1] as *mut u8, args[2]),
        SYS_EXIT => sys_exit(args[0]),
        _ => {
            println!("unimplemented syscall: {}", syscall_id);
            SyscallResult::Kill
        }
    };

    match result {
        SyscallResult::Proceed(ret) => {
            context.x[0] = ret as usize;
            context
        }
        SyscallResult::Park(ret) => {
            context.x[10] = ret as usize;
            PROCESSOR.get().park_current_thread(context);
            PROCESSOR.get().prepare_next_thread()
        }
        SyscallResult::Kill => {
            PROCESSOR.get().kill_current_thread();
            PROCESSOR.get().prepare_next_thread()
        }
    }

}