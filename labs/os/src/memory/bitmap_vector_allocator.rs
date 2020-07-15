

extern crate bit_field;
use bit_field::BitArray;

pub type VectorAllocatorImpl = BitmapVectorAllocator;

/// 向量分配器：固定容量，每次分配 / 回收一个带有对齐要求的连续向量
///
/// 参数和返回值中的 usize 表示第 n 个字节，不需要考虑起始地址
pub trait VectorAllocator {
    fn new(capacity: usize) -> Self;
    fn alloc(&mut self, size: usize, align: usize) -> Option<usize>;
    fn dealloc(&mut self, start: usize, size: usize, align: usize);
}

const BITMAP_SIZE: usize = 4096;

pub struct BitmapVectorAllocator {
    capacity: usize,
    bitmap: [u8; BITMAP_SIZE / 8]
}

impl VectorAllocator for BitmapVectorAllocator {
    fn new(capacity: usize) -> Self {
        Self {
            capacity: core::cmp::min(BITMAP_SIZE, capacity),
            bitmap: [0u8; BITMAP_SIZE / 8]
        }
    }
    fn alloc(&mut self, size: usize, align: usize) -> Option<usize> {
        for start in (0..self.capacity - size).step_by(align) {
            if (start..start + size).all(|i| !self.bitmap.get_bit(i)) {
                (start..start + size).for_each(|i| self.bitmap.set_bit(i, true));
                return Some(start);
            }
        }
        None
    }
    fn dealloc(&mut self, start: usize, size: usize, _align: usize) {
        // ???👇↓
        assert!(self.bitmap.get_bit(start));
        (start..start + size).for_each(|i| self.bitmap.set_bit(i, false));
    }
}