//! 页表的构建 [`Mapping`]
//!
//!

use crate::memory::{
    address::*,
    config::PAGE_SIZE,
    frame::{FrameTracker, FRAME_ALLOCATOR},
    mapping::{Flags, MapType, PageTable, PageTableEntry, PageTableTracker, Segment},
    MemoryResult,
};
use alloc::{vec, vec::Vec};
use core::cmp::min;
use core::ptr::slice_from_raw_parts_mut;

#[derive(Default)]
pub struct Mapping {
    page_tables: Vec<PageTableTracker>,
    root_ppn: PhysicalPageNumber,
}

impl Mapping {
    pub fn activate(&self) {
        println!("mapping root page table at: {:x?}", self.root_ppn);
        let new_satp = self.root_ppn.0 | (8 << 60);
        unsafe {
            llvm_asm!("csrw satp, $0" :: "r"(new_satp) :: "volatile");
            llvm_asm!("sfence.vma" :::: "volatile");
        }
    }
    /// 创建新映射, 分配根节点
    pub fn new() -> MemoryResult<Mapping> {
        let root_table = PageTableTracker::new(FRAME_ALLOCATOR.lock().alloc()?);
        let root_ppn = root_table.page_number();
        Ok(Mapping {
            page_tables: vec![root_table],
            root_ppn,
        })
    }
    /// 离散映射的时候会返回分配的物理页(和虚拟地址的tuple)的Vec
    pub fn map(
        &mut self,
        segment: &Segment,
        init_data: Option<&[u8]>,
    ) -> MemoryResult<Vec<(VirtualPageNumber, FrameTracker)>> {
        match segment.map_type {
            MapType::Linear => {
                for vpn in segment.page_range().iter() {
                    self.map_one(vpn, vpn.into(), segment.flags | Flags::VALID)?;
                }
                if let Some(data) = init_data {
                    unsafe {
                        (&mut *slice_from_raw_parts_mut(segment.range.start.deref(), data.len()))
                            .copy_from_slice(data);
                    }
                }
                Ok(Vec::new())
            }
            MapType::Framed => {
                let mut allocated_pairs = Vec::new();
                for vpn in segment.page_range().iter() {
                    let mut frame = FRAME_ALLOCATOR.lock().alloc()?;
                    self.map_one(vpn, frame.page_number(), segment.flags | Flags::VALID)?;
                    frame.fill(0);
                    allocated_pairs.push((vpn, frame));
                }

                if let Some(data) = init_data {
                    if !data.is_empty() {
                        for (vpn, frame) in allocated_pairs.iter_mut() {
                            let page_address = VirtualAddress::from(*vpn);
                            let start = if segment.range.start > page_address {
                                segment.range.start - page_address
                            } else {
                                0
                            };
                            let stop = min(PAGE_SIZE, segment.range.end - page_address);
                            let dst_slice = &mut frame[start..stop];
                            let src_slice = &data[(page_address + start - segment.range.start)
                                ..(page_address + stop - segment.range.start)];
                            dst_slice.copy_from_slice(src_slice);
                        }
                    }
                }
                Ok(allocated_pairs)
            }
        }
    }
    pub fn unmap(&mut self, segment: &Segment) {
        for vpn in segment.page_range().iter() {
            let entry = self.find_entry(vpn).unwrap();
            assert!(!entry.is_empty());
            // 从页表中清除项
            entry.clear();
        }
    }
    /// 从根页表不断查找, 创建中间的页表, 最终找到对应的最低级页表项
    pub fn find_entry(&mut self, vpn: VirtualPageNumber) -> MemoryResult<&mut PageTableEntry> {
        // 这里不用 self.page_tables[0] 避免后面产生 borrow-check 冲突（我太菜了）
        let root_table: &mut PageTable = PhysicalAddress::from(self.root_ppn).deref_kernel();
        let mut entry = &mut root_table.entries[vpn.levels()[0]];
        for vpn_slice in &vpn.levels()[1..] {
            if entry.is_empty() {
                // 如果页表不存在，则需要分配一个新的页表
                let new_table = PageTableTracker::new(FRAME_ALLOCATOR.lock().alloc()?);
                let new_ppn = new_table.page_number();
                *entry = PageTableEntry::new(new_ppn, Flags::VALID);
                self.page_tables.push(new_table);
            }
            entry = &mut entry.get_next_table().entries[*vpn_slice];
        }
        Ok(entry)
    }
    pub fn lookup(va: VirtualAddress) -> Option<PhysicalAddress> {
        let mut current_ppn;
        unsafe {
            llvm_asm!("csrr $0, satp": "=r"(current_ppn) ::: "volatile");
            current_ppn ^= 8 << 60;
        }
        let root_table: &PageTable =
            PhysicalAddress::from(PhysicalPageNumber(current_ppn)).deref_kernel();
        let vpn = VirtualPageNumber::floor(va);
        let mut entry = &root_table.entries[vpn.levels()[0]];
        let mut length = 12 + 2 * 9;
        for vpn_slice in &vpn.levels()[1..] {
            if entry.is_empty() {
                return None;
            }
            if entry.has_next_level() {
                length -= 9;
                entry = &mut entry.get_next_table().entries[*vpn_slice];
            } else {
                break;
            }
        }
        let base = PhysicalAddress::from(entry.page_number()).0;
        let offset = va.0 & ((1 << length) - 1);
        Some(PhysicalAddress(base + offset))
    }
    pub fn print_page_table(va: VirtualAddress) {
        println!("print page table for {:x?}", va);
        let mut current_ppn;
        unsafe {
            llvm_asm!("csrr $0, satp": "=r"(current_ppn) ::: "volatile");
            current_ppn ^= 8 << 60;
        }
        let root_table: &PageTable =
            PhysicalAddress::from(PhysicalPageNumber(current_ppn)).deref_kernel();
        let vpn = VirtualPageNumber::floor(va);
        let mut entry = &root_table.entries[vpn.levels()[0]];
        println!("vpn levels: {:x?}", vpn.levels());
        let mut length = 12 + 2 * 9;
        for vpn_slice in &vpn.levels()[1..] {
            if entry.is_empty() {
                println!("entry empty at len: {}", length);
                return;
            }
            if entry.has_next_level() {
                println!("got entry at {:p}, {:x?}", entry, entry);
                length -= 9;
                entry = &mut entry.get_next_table().entries[*vpn_slice];
            } else {
                break;
            }
        }
        let base = PhysicalAddress::from(entry.page_number()).0;
        let offset = va.0 & ((1 << length) - 1);
        println!("find entry at {:p}, {:x?}", entry, entry);
        println!("find physical addr: {:x?}", PhysicalAddress(base + offset));
    }
    pub fn map_one(
        &mut self,
        vpn: VirtualPageNumber,
        ppn: PhysicalPageNumber,
        flags: Flags,
    ) -> MemoryResult<()> {
        let entry = self.find_entry(vpn)?;
        assert!(entry.is_empty(), "virtual address is already mapped!");
        *entry = PageTableEntry::new(ppn, flags);
        Ok(())
    }
}
