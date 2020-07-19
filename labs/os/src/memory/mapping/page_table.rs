//! 单一页表页面（4K） [`PageTable`]，以及相应封装 [`FrameTracker`] 的 [`PageTableTracker`]
//!

use super::super::address::PhysicalPageNumber;
use super::super::{config::PAGE_SIZE, frame::FrameTracker};
use super::page_table_entry::PageTableEntry;

/// 创建时申请物理页将指向页的指针转为PageTable类型
#[repr(C)]
pub struct PageTable {
    pub entries: [PageTableEntry; PAGE_SIZE / 8],
}

impl PageTable {
    pub fn zero_init(&mut self) {
        self.entries = [Default::default(); PAGE_SIZE / 8];
    }
}

pub struct PageTableTracker(pub FrameTracker);

impl PageTableTracker {
    pub fn new(frame: FrameTracker) -> Self {
        let mut page_table = Self(frame);
        page_table.zero_init(); // 转为了PageTable
        page_table
    }
    pub fn page_number(&self) -> PhysicalPageNumber {
        self.0.page_number()
    }
}

impl core::ops::Deref for PageTableTracker {
    type Target = PageTable;
    fn deref(&self) -> &Self::Target {
        self.0.address().deref_kernel()
    }
}

impl core::ops::DerefMut for PageTableTracker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.address().deref_kernel()
    }
}

impl PageTableEntry {
    pub fn get_next_table(&self) -> &'static mut PageTable {
        self.address().deref_kernel()
    }
}
