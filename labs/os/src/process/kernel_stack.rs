use super::*;
use core::mem::size_of;

#[repr(align(16))]
#[repr(C)]
pub struct KernelStack([u8; KERNEL_STACK_SIZE]);

/// 共用的内核栈
pub static mut KERNEL_STACK: KernelStack = KernelStack([0; KERNEL_STACK_SIZE]);

impl KernelStack {
    pub fn push_context(&mut self, context: Context) -> *mut Context {
        let stack_top = &self.0 as *const _ as usize + size_of::<Self>();
        let push_address = (stack_top - size_of::<Context>()) as *mut Context;
        unsafe {
            *push_address = context;
        }
        push_address
    }
}
