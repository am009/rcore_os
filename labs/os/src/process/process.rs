use super::*;
use xmas_elf::ElfFile;

pub struct Process {
    pub is_user: bool,
    pub memory_set: MemorySet,
}

impl Process {
    /// 创建内核进程
    pub fn new_kernel() -> MemoryResult<Arc<RwLock<Self>>> {
        Ok(Arc::new(RwLock::new(Self {
            is_user: false,
            memory_set: MemorySet::new_kernel()?,
        })))
    }
    pub fn from_elf(file: &ElfFile, is_user: bool) -> MemoryResult<Arc<RwLock<Self>>> {
        Ok(Arc::new(RwLock::new(Self {
            is_user,
            memory_set: MemorySet::from_elf(file, is_user)?,
        })))
    }
    pub fn alloc_page_range(
        &mut self,
        size: usize,
        flags: Flags,
    ) -> MemoryResult<Range<VirtualAddress>> {
        let alloc_size = (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let mut range = Range::<VirtualAddress>::from(0x100_0000..0x100_0000 + alloc_size);
        while self.memory_set.overlap_with(range.into()) {
            range.start += alloc_size;
            range.end += alloc_size;
        }

        self.memory_set.add_segment(
            Segment {
                map_type: MapType::Framed,
                range,
                flags: flags | Flags::user(self.is_user),
            },
            None,
        )?;
        // 这里还是size没有向上取整
        Ok(Range::from(range.start..(range.start + size)))
    }
}
