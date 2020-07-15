//! 表示页面区间 [`Range`]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Range<T: From<usize> + Into<usize> + Copy> {
    pub start: T,
    pub end: T
}

/// 兼容core::ops::Range
impl<T: From<usize> + Into<usize> + Copy, U: Into<T>> From<core::ops::Range<U>> for Range<T> {
    fn from(range: core::ops::Range<U>) -> Self{
        Self {
            start: range.start.into(),
            end: range.end.into()
        }
    }
}

/// 这个T说的正是PhysicalPageNumber
impl<T: From<usize> + Into<usize> + Copy> Range<T> {
    /// 检测重合
    pub fn overlap_with(&self, other: &Range<T>) -> bool {
        self.start.into() < other.end.into() && self.end.into() > other.start.into()
    }
    /// 迭代每个页
    pub fn iter(&self) -> impl Iterator<Item = T> {
        (self.start.into()..self.end.into()).map(T::from)
    }

    pub fn len(&self) -> usize {
        self.end.into() - self.start.into()
    }

    /// 物理/虚拟页面相互转换
    pub fn into<U: From<usize> + Into<usize> + Copy + From<T>>(self) -> Range<U> {
        Range::<U> {
            start: U::from(self.start),
            end: U::from(self.end)
        }
    }

    /// 用下标取元素
    pub fn get(&self, index: usize) -> T {
        assert!(index < self.len());
        T::from(self.start.into() + index)
    }

    /// 区间是否包含指定的值
    pub fn contains(&self, value: T) -> bool {
        self.start.into() <= value.into() && value.into() < self.end.into()
    }
}