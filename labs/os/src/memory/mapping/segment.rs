//! 映射类型 [`MapType`] 和映射片段 [`Segment`]

use crate::memory::range::Range;
use crate::memory::mapping::Flags;
use crate::memory::address::{VirtualAddress, VirtualPageNumber, PhysicalPageNumber};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MapType {
    Linear,
    Framed
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Segment {
    pub map_type: MapType,
    pub range: Range<VirtualAddress>,
    pub flags: Flags
}

impl Segment {
    /// 线性映射的时候按页遍历
    pub fn iter_mapped(&self) -> Option<impl Iterator<Item = PhysicalPageNumber>> {
        match self.map_type {
            MapType::Linear => Some(self.page_range().into().iter()),
            MapType::Framed => None
        }
    }
    pub fn page_range(&self) -> Range<VirtualPageNumber> {
        Range::from(
            VirtualPageNumber::floor(self.range.start)..VirtualPageNumber::ceil(self.range.end)
        )
    }
}