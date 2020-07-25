use super::*;
use crate::fs::*;
use core::slice::from_raw_parts_mut;
use core::slice::from_raw_parts;


pub(super) fn sys_read(fd: usize, buffer: *mut u8, size: usize) -> SyscallResult {
    let inode: Arc<dyn INode> = 
        if let Some(inode) = PROCESSOR.get().current_thread().inner().descriptors.get(fd) {
            inode.clone()
        } else {
            return SyscallResult::Proceed(-1);
        };
    let buffer = unsafe { from_raw_parts_mut(buffer, size) };
    if let Ok(ret) = inode.read_at(0, buffer) {
        let ret = ret as isize;
        if ret > 0 {
            return SyscallResult::Proceed(ret);
        }
        if ret == 0 {
            return SyscallResult::Park(ret);
        }
    }
    SyscallResult::Proceed(-1)
}

pub(super) fn sys_write(fd: usize, buffer: *mut u8, size: usize) -> SyscallResult {
    if let Some(inode) = PROCESSOR.get().current_thread().inner().descriptors.get(fd) {
        let buf = unsafe { from_raw_parts(buffer, size) };
        if let Ok(ret) = inode.write_at(0, buf) {
            let ret = ret as isize;
            if ret >= 0 {
                return SyscallResult::Proceed(ret);
            }
        }
    }
    SyscallResult::Proceed(-1)
}