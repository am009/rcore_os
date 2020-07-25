//! 一个线程中关于内存空间的所有信息 [`MemorySet`]
//!
//! 一组映射

use crate::memory::{
    address::*,
    config::*,
    frame::FrameTracker,
    mapping::{Flags, MapType, Mapping, Segment},
    range::Range,
    MemoryResult,
};
use alloc::{vec, vec::Vec};
use xmas_elf::{
    program::{SegmentData, Type},
    ElfFile,
};

pub struct MemorySet {
    pub mapping: Mapping,
    pub segments: Vec<Segment>,
    pub allocated_pairs: Vec<(VirtualPageNumber, FrameTracker)>,
}

impl MemorySet {
    pub fn overlap_with(&self, range: Range<VirtualPageNumber>) -> bool {
        for seg in self.segments.iter() {
            if range.overlap_with(&seg.page_range()) {
                return true;
            }
        }
        false
    }
    pub fn remove_segment(&mut self, segment: &Segment) -> MemoryResult<()> {
        let segment_index = self
            .segments
            .iter()
            .position(|s| s == segment)
            .expect("segment to remove is not found !");
        self.segments.remove(segment_index);
        self.mapping.unmap(segment);
        self.allocated_pairs
            .retain(|(vpn, _frame)| !segment.page_range().contains(*vpn));
        Ok(())
    }
    pub fn add_segment(&mut self, segment: Segment, init_data: Option<&[u8]>) -> MemoryResult<()> {
        assert!(!self.overlap_with(segment.page_range()));
        self.allocated_pairs
            .extend(self.mapping.map(&segment, init_data)?);
        self.segments.push(segment);
        Ok(())
    }
    /// 如果当前页表就是自身，则不会替换，但仍然会刷新 TLB。
    pub fn activate(&self) {
        println!("activate memory set: {:x?}", self.segments);
        self.mapping.activate()
    }
    pub fn from_elf(file: &ElfFile, is_user: bool) -> MemoryResult<MemorySet> {
        let mut memory_set = MemorySet::new_kernel()?;
        for program_header in file.program_iter() {
            if program_header.get_type() != Ok(Type::Load) {
                continue;
            }
            let start = VirtualAddress(program_header.virtual_addr() as usize);
            let size = program_header.mem_size() as usize;
            let data: &[u8] = 
                if let SegmentData::Undefined(data) = program_header.get_data(file).unwrap() {
                    data
                } else {
                    return Err("unsupported elf format.");
                };
            let segment = Segment {
                map_type: MapType::Framed,
                range: Range::from(start..(start + size)),
                flags: Flags::user(is_user)
                    | Flags::readable(program_header.flags().is_read())
                    | Flags::writable(program_header.flags().is_write())
                    | Flags::executable(program_header.flags().is_execute()),
            };
            memory_set.add_segment(segment, Some(data))?;
        }
        Ok(memory_set)
    }
    /// 建立内核映射
    /// 给main函数调用
    pub fn new_kernel() -> MemoryResult<MemorySet> {
        extern "C" {
            fn text_start();
            fn rodata_start();
            fn data_start();
            fn bss_start();
        }
        let segments = vec![
            // DEVICE 段，rw-
            Segment {
                map_type: MapType::Linear,
                range: Range::from(DEVICE_START_ADDRESS..DEVICE_END_ADDRESS),
                flags: Flags::READABLE | Flags::WRITABLE,
            },
            // .text 段，r-x
            Segment {
                map_type: MapType::Linear,
                range: Range::from((text_start as usize)..(rodata_start as usize)),
                flags: Flags::READABLE | Flags::EXECUTABLE,
            },
            // .rodata 段，r--
            Segment {
                map_type: MapType::Linear,
                range: Range::from((rodata_start as usize)..(data_start as usize)),
                flags: Flags::READABLE,
            },
            // .data 段，rw-
            Segment {
                map_type: MapType::Linear,
                range: Range::from((data_start as usize)..(bss_start as usize)),
                flags: Flags::READABLE | Flags::WRITABLE,
            },
            // .bss 段，rw-
            Segment {
                map_type: MapType::Linear,
                range: Range::<VirtualAddress>::from(
                    VirtualAddress::from(bss_start as usize)..*KERNEL_END_ADDRESS,
                ),
                flags: Flags::READABLE | Flags::WRITABLE,
            },
            // 剩余内存空间，rw-
            Segment {
                map_type: MapType::Linear,
                range: Range::from(*KERNEL_END_ADDRESS..VirtualAddress::from(MEMORY_END_ADDRESS)),
                flags: Flags::READABLE | Flags::WRITABLE,
            },
        ];
        let mut mapping = Mapping::new()?;
        let mut allocated_pairs = Vec::new();

        for segment in segments.iter() {
            allocated_pairs.extend(mapping.map(segment, None)?);
        }
        Ok(MemorySet {
            mapping,
            segments,
            allocated_pairs,
        })
    }
}
