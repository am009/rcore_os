//! 控制台输出 [`Stdout`]
//! 

use super::*;
use core::any::Any;
use crate::memory::mapping::Mapping;
use crate::memory::address::VirtualAddress;

lazy_static! {
    pub static ref STDOUT: Arc<Stdout> = Default::default();
}

#[derive(Default)]
pub struct Stdout;

impl INode for Stdout {
    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize> {
        println!("write buffer: {:p}", buf);
        Mapping::print_page_table(VirtualAddress::from(buf.as_ptr()));
        unsafe {
            let sscratch: usize = crate::process::KERNEL_STACK.get_top();
            llvm_asm!("csrw sscratch, $0" :: "r"(sscratch) :: "volatile");
        } // DEBUG
        if offset != 0 {
            Err(FsError::NotSupported)
        } else if let Ok(string) = core::str::from_utf8(buf) {
            print!("{}", string);
            Ok(buf.len())
        } else {
            Err(FsError::InvalidParam)
        }
    }
    fn read_at(&self, _offset: usize, _buf: &mut [u8]) -> Result<usize>{
        Err(FsError::NotSupported)
    }
    fn poll(&self) -> Result<PollStatus> {
        Err(FsError::NotSupported)
    }
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
}

