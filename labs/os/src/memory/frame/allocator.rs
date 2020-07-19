use super::frame_tracker::FrameTracker;
use super::stacked_allocator::StackedAllocator;
use crate::memory::address::{PhysicalAddress, PhysicalPageNumber};
use crate::memory::config::{KERNEL_END_ADDRESS, MEMORY_END_ADDRESS};
use crate::memory::range::Range;
use crate::memory::MemoryResult;

use lazy_static::lazy_static;
use spin::Mutex;

/// 默认使用的分配器
pub type AllocImpl = StackedAllocator;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<FrameAllocator<AllocImpl>> =
        Mutex::new(FrameAllocator::new(Range::from(
            PhysicalPageNumber::ceil(PhysicalAddress::from(*KERNEL_END_ADDRESS))
                ..PhysicalPageNumber::floor(MEMORY_END_ADDRESS),
        )));
}

/// 分配器：固定容量，每次分配 / 回收一个元素
pub trait Allocator {
    /// 给定容量，创建分配器
    fn new(capacity: usize) -> Self;
    /// 分配一个元素，无法分配则返回 `None`
    fn alloc(&mut self) -> Option<usize>;
    /// 回收一个元素
    fn dealloc(&mut self, index: usize);
}

pub struct FrameAllocator<T: Allocator> {
    /// 起始页号
    start_ppn: PhysicalPageNumber,
    /// 分配器
    allocator: T,
}

impl<T: Allocator> FrameAllocator<T> {
    pub fn new(range: impl Into<Range<PhysicalPageNumber>> + Copy) -> Self {
        FrameAllocator {
            start_ppn: range.into().start,
            allocator: T::new(range.into().len()),
        }
    }

    pub fn alloc(&mut self) -> MemoryResult<FrameTracker> {
        self.allocator
            .alloc()
            .ok_or("no available frame to alloc")
            .map(|offset| FrameTracker(self.start_ppn + offset))
    }

    /// 只由frame_tracker调用
    pub(super) fn dealloc(&mut self, frame: &FrameTracker) {
        self.allocator.dealloc(frame.page_number() - self.start_ppn);
    }
}
