use super::address::PhysicalAddress;
use lazy_static::lazy_static;

pub const KERNEL_HEAP_SIZE: usize = 0x80_0000;

extern "C" {
    fn kernel_end();
}

pub const PAGE_SIZE: usize = 4096;

pub const MEMORY_START_ADDRESS: PhysicalAddress = PhysicalAddress(0xffff_ffff_8000_0000);
/// 可以访问的内存区域结束地址
pub const MEMORY_END_ADDRESS: PhysicalAddress = PhysicalAddress(0xffff_ffff_8800_0000);

pub const KERNEL_MAP_OFFSET: usize = 0xffff_ffff_0000_0000;

lazy_static! {
    pub static ref KERNEL_END_ADDRESS: PhysicalAddress = PhysicalAddress(kernel_end as usize);
}


