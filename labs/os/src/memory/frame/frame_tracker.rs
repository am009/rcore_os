/// `Tracker` 就是指向物理页面的智能指针, 
/// 对接物理内存管理器, 自动析构

use crate::memory::address::{PhysicalPageNumber, PhysicalAddress};
use super::allocator::FRAME_ALLOCATOR;

pub struct FrameTracker(pub(super) PhysicalPageNumber);

impl FrameTracker {
    /// 物理地址
    pub fn address(&self) -> PhysicalAddress {
        self.0.into()
    }
    /// 物理页号
    pub fn page_number(&self) -> PhysicalPageNumber {
        self.0
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().dealloc(self);
    }
}