
use super::*;

pub(super) fn sys_exit(code: usize) -> SyscallResult {
    println!(
        "thread {} exit with code {}.",
        PROCESSOR.get().current_thread().id,
        code
    );
    SyscallResult::Kill
}